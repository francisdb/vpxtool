//! Read high scores from a VPReg.ini file.
//!
//! `VPReg.ini` is the standalone-vpinball stand-in for the Windows registry
//! that the in-VBS `LoadValue` / `SaveValue` calls (from `core.vbs`) target.
//! Each table writes its data under a `[<cGameName>]` section. The modern
//! convention used by all rom-less tables we care about here is:
//!
//! ```ini
//! [TheMatrix]
//! HighScore1=1154150
//! HighScore1Name=SOM
//! HighScore2=100000
//! HighScore2Name=AAA
//! ...
//! Credits=5
//! TotalGamesPlayed=4
//! ```
//!
//! The exact entry count varies per table (1, 4, 5, 12, and 16 all observed
//! in the wild - Stern tables in particular keep 16 ranks). Some tables omit
//! the `HighScoreNName` keys entirely (score-only ranked lists like Volkan).
//! Non-score keys in the same section (`Credits`, `TotalGamesPlayed`,
//! `MasterVol`, `SETDIPS`, ...) are ignored.
//!
//! Older EM tables (e.g. Abra Ca Dabra, 4 Roses) use a different shape:
//! a single `hiscore=N` integer with optional `hsa1`/`hsa2`/`hsa3` indices
//! into a fixed alphabet (`"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_<"`,
//! 1-based via VBS `Mid()`) - `hsa1=4 hsa2=15 hsa3=7` decodes to "DOG".
//! Falls back to this when the modern HighScoreN keys are absent.
//!
//! The section name must be passed in explicitly - it is the script's
//! `cGameName` constant and the .ini key does not encode it any other way.

use std::collections::BTreeMap;
use std::path::Path;

use ini::Ini;

use super::Section;

/// Outcome of looking for a `[section]` in a VPReg.ini.
#[derive(Debug, PartialEq, Eq)]
pub enum LookupError {
    /// The section exists but has no `HighScoreN` keys. Common for tables
    /// that store only settings (e.g. PinMAME tables that use VPReg.ini
    /// for SETDIPS but keep scores in the nvram).
    SectionHasNoScores,
    /// The .ini file has no `[section]` matching the script's `cGameName`.
    SectionNotFound,
    /// The .ini file is unreadable or malformed.
    ParseFailed(String),
}

/// Read the score section for `game_name` from `vpreg_path`, returning a
/// single ranked `HIGH SCORES` section. The rows match the same column shape
/// as the rest of the scores pipeline: `[label, initials, score, units]`,
/// with `label` set to `"#N"` for traceability and `units` always empty
/// (VPReg.ini scores are unitless integers).
pub fn read_sections(vpreg_path: &Path, game_name: &str) -> Result<Vec<Section>, LookupError> {
    let ini =
        Ini::load_from_file(vpreg_path).map_err(|e| LookupError::ParseFailed(e.to_string()))?;
    extract_sections(&ini, game_name)
}

/// 1-indexed alphabet used by the legacy EM `hsa<N>` initials encoding.
/// `Mid()` in VBS is 1-based, so a value of 1 maps to `A`, 4 to `D`, etc.
/// This exact string was found in 100+ extracted scripts (4 Aces, Apollo,
/// Ace High, ... - it's a near-universal convention).
const LEGACY_EM_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_<";

/// Same as [`read_sections`] but operates on an already-parsed `Ini`. Split
/// out so tests can drive the parser from inline string fixtures without
/// writing temp files.
fn extract_sections(ini: &Ini, game_name: &str) -> Result<Vec<Section>, LookupError> {
    let section = ini
        .section(Some(game_name))
        .ok_or(LookupError::SectionNotFound)?;

    // Collect `HighScoreN` / `HighScoreNName` pairs keyed by N so we render
    // them in rank order regardless of the .ini's physical key ordering.
    // BTreeMap gives us a stable ascending iteration which is what we want.
    let mut scores: BTreeMap<u32, &str> = BTreeMap::new();
    let mut names: BTreeMap<u32, &str> = BTreeMap::new();
    for (key, value) in section.iter() {
        if let Some(rest) = key.strip_prefix("HighScore") {
            if let Some(n_str) = rest.strip_suffix("Name") {
                if let Ok(n) = n_str.parse::<u32>() {
                    names.insert(n, value);
                }
            } else if let Ok(n) = rest.parse::<u32>() {
                scores.insert(n, value);
            }
        }
    }

    if scores.is_empty() {
        // Fall back to the legacy EM pattern (`hiscore=N` + optional
        // `hsa1`/`hsa2`/`hsa3` for index-encoded initials) before giving
        // up. Tables like Abra Ca Dabra and 4 Roses use this older shape.
        if let Some(legacy) = try_legacy_em(section, game_name) {
            return Ok(vec![legacy]);
        }
        return Err(LookupError::SectionHasNoScores);
    }

    let rows: Vec<Vec<String>> = scores
        .into_iter()
        .map(|(n, score)| {
            let initials = names.get(&n).copied().unwrap_or("").trim().to_string();
            vec![
                format!("#{n}"),
                initials,
                score.trim().to_string(),
                String::new(),
            ]
        })
        .collect();

    let ranked = rows.len() > 1;
    let header = if ranked {
        "HIGH SCORES".to_string()
    } else {
        // Single-entry section: use the table's section name as the header
        // (uppercased to match the pinmame branch's convention) since the
        // row itself has a rank-shaped label and there is nothing else to
        // hang the header off.
        game_name.to_uppercase()
    };

    Ok(vec![Section {
        header,
        rows,
        ranked,
    }])
}

/// Read the legacy EM `hiscore` / `hsa<N>` shape from an already-located
/// section. Returns `None` when there's no `hiscore` key (the strict signal
/// that this is the older form), or when the value isn't a positive integer.
/// Initials decode from `hsa1`/`hsa2`/`hsa3` as 1-indexed positions into
/// [`LEGACY_EM_ALPHABET`]; missing or out-of-range indices yield an empty
/// initial slot for that position.
fn try_legacy_em(section: &ini::Properties, game_name: &str) -> Option<Section> {
    let hiscore: u64 = section.get("hiscore")?.trim().parse().ok()?;
    if hiscore == 0 {
        return None;
    }
    let initials: String = ["hsa1", "hsa2", "hsa3"]
        .iter()
        .filter_map(|k| section.get(k))
        .filter_map(|v| v.trim().parse::<usize>().ok())
        .filter_map(decode_legacy_em_initial)
        .collect();
    Some(Section {
        header: game_name.to_uppercase(),
        rows: vec![vec![
            "HIGH SCORE".to_string(),
            initials,
            hiscore.to_string(),
            String::new(),
        ]],
        ranked: false,
    })
}

/// Map a 1-indexed `hsa<N>` value to its alphabet character. Returns `None`
/// for index 0 (treated as "unfilled slot" by the EM scripts) and for
/// out-of-range indices.
fn decode_legacy_em_initial(idx_1based: usize) -> Option<char> {
    if idx_1based == 0 {
        return None;
    }
    LEGACY_EM_ALPHABET.get(idx_1based - 1).map(|&b| b as char)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn parse(content: &str) -> Ini {
        Ini::load_from_str(content).expect("ini parse")
    }

    #[test]
    fn parses_modern_four_entry_section() {
        // Matrix (Original 2023) - real shape with names alongside scores.
        let ini = parse(
            r"
[TheMatrix]
HighScore1=1154150
HighScore1Name=SOM
HighScore2=100000
HighScore2Name=AAA
HighScore3=100000
HighScore3Name=BBB
HighScore4=100000
HighScore4Name=CCC
Credits=5
TotalGamesPlayed=4
",
        );
        let sections = extract_sections(&ini, "TheMatrix").expect("section");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header, "HIGH SCORES");
        assert!(sections[0].ranked);
        assert_eq!(sections[0].rows.len(), 4);
        assert_eq!(sections[0].rows[0], vec!["#1", "SOM", "1154150", ""]);
        assert_eq!(sections[0].rows[3], vec!["#4", "CCC", "100000", ""]);
    }

    #[test]
    fn parses_section_with_scores_but_no_names() {
        // Volkan Steel and Metal: 12 HighScoreN keys, no HighScoreNName.
        let ini = parse(
            r"
[volkan]
HighScore1=20000
HighScore2=30000
HighScore3=50000
",
        );
        let sections = extract_sections(&ini, "volkan").expect("section");
        assert_eq!(sections[0].rows[0], vec!["#1", "", "20000", ""]);
        assert_eq!(sections[0].rows[2], vec!["#3", "", "50000", ""]);
    }

    #[test]
    fn orders_by_rank_number_not_ini_order() {
        // 16-entry Stern-style table; verify N=10..16 sort correctly after
        // N=1..9 (string ordering of "HighScore10" < "HighScore2" would
        // break this if we forgot to parse N as an integer).
        let ini = parse(
            r"
[gameofthrones]
HighScore2=500000000
HighScore2Name=BBB
HighScore10=7000000
HighScore10Name=JJJ
HighScore1=750000000
HighScore1Name=AAA
",
        );
        let sections = extract_sections(&ini, "gameofthrones").expect("section");
        let labels: Vec<&str> = sections[0].rows.iter().map(|r| r[0].as_str()).collect();
        assert_eq!(labels, vec!["#1", "#2", "#10"]);
    }

    #[test]
    fn single_entry_section_marks_unranked_and_uses_section_header() {
        // Loch Ness Monster: just one HighScore1 entry, no name. Treat as
        // an unranked single-entry section so the renderer doesn't prefix
        // the lone row with "1.".
        let ini = parse(
            r"
[Lochness]
HighScore1=100000
Credits=0
TotalGamesPlayed=0
",
        );
        let sections = extract_sections(&ini, "Lochness").expect("section");
        assert_eq!(sections.len(), 1);
        assert!(!sections[0].ranked);
        assert_eq!(sections[0].header, "LOCHNESS");
        assert_eq!(sections[0].rows.len(), 1);
    }

    #[test]
    fn ignores_unrelated_keys_in_the_section() {
        // Credits/TotalGamesPlayed/MasterVol/SETDIPS commonly live alongside
        // the HighScore keys; the parser must skip them silently.
        let ini = parse(
            r"
[somegame]
SETDIPS=0
HighScore1=42
HighScore1Name=FOO
MasterVol=99
Credits=3
TotalGamesPlayed=7
",
        );
        let sections = extract_sections(&ini, "somegame").expect("section");
        assert_eq!(sections[0].rows[0], vec!["#1", "FOO", "42", ""]);
        assert_eq!(sections[0].rows.len(), 1);
    }

    #[test]
    fn returns_section_not_found_when_game_name_missing() {
        let ini = parse(
            r"
[OtherGame]
HighScore1=10
",
        );
        let err = extract_sections(&ini, "TheMatrix").expect_err("should miss");
        assert_eq!(err, LookupError::SectionNotFound);
    }

    #[test]
    fn returns_no_scores_when_section_has_only_settings() {
        // Haunted House's [hh] section only carries SETDIPS, the actual
        // scores live in the pinmame nvram. We surface this distinctly
        // from "section missing" so the caller can fall through cleanly.
        let ini = parse(
            r"
[hh]
SETDIPS=0
",
        );
        let err = extract_sections(&ini, "hh").expect_err("should be empty");
        assert_eq!(err, LookupError::SectionHasNoScores);
    }

    #[test]
    fn parses_legacy_em_abra_ca_dabra_shape() {
        // Real Abra Ca Dabra VPReg.ini after a real game: hiscore=10000,
        // hsa1=4/hsa2=15/hsa3=7 -> 'D','O','G' in the canonical alphabet.
        let ini = parse(
            r"
[Abra_Ca_Dabra]
credit=0
hiscore=10000
hsa1=4
hsa2=15
hsa3=7
score1=1910
",
        );
        let sections = extract_sections(&ini, "Abra_Ca_Dabra").expect("section");
        assert_eq!(sections.len(), 1);
        assert!(!sections[0].ranked);
        assert_eq!(sections[0].header, "ABRA_CA_DABRA");
        assert_eq!(sections[0].rows[0], vec!["HIGH SCORE", "DOG", "10000", ""]);
    }

    #[test]
    fn legacy_em_missing_hsa_yields_empty_initials() {
        // Some early EM tables don't write hsa keys at all; the single
        // hiscore value should still surface with an empty initials slot.
        let ini = parse(
            r"
[some_em_table]
credit=0
hiscore=5000
score1=
",
        );
        let sections = extract_sections(&ini, "some_em_table").expect("section");
        assert_eq!(sections[0].rows[0], vec!["HIGH SCORE", "", "5000", ""]);
    }

    #[test]
    fn legacy_em_zero_hiscore_falls_through() {
        // Default-zero `hiscore=0` (never reached) must not be surfaced as
        // a real high score; the parser returns SectionHasNoScores so the
        // dispatcher can keep probing.
        let ini = parse(
            r"
[some_em_table]
credit=0
hiscore=0
hsa1=0
hsa2=0
hsa3=0
",
        );
        let err = extract_sections(&ini, "some_em_table").expect_err("no real score");
        assert_eq!(err, LookupError::SectionHasNoScores);
    }

    #[test]
    fn legacy_em_decodes_extended_alphabet_chars() {
        // Positions 27-38 in the alphabet are 0-9 / _ / <; verify decode.
        // hsa1=27 = '0', hsa2=37 = '_' (space), hsa3=38 = '<' (backspace).
        let ini = parse(
            r"
[em_extended]
hiscore=42
hsa1=27
hsa2=37
hsa3=38
",
        );
        let sections = extract_sections(&ini, "em_extended").expect("section");
        assert_eq!(sections[0].rows[0][1], "0_<");
    }

    #[test]
    fn legacy_em_drops_out_of_range_hsa_index() {
        // Out-of-range index (or 0) decodes to no character so the
        // remaining initials still render.
        let ini = parse(
            r"
[em_oor]
hiscore=42
hsa1=4
hsa2=99
hsa3=7
",
        );
        let sections = extract_sections(&ini, "em_oor").expect("section");
        // Position 99 dropped, leaves 'D' + 'G'.
        assert_eq!(sections[0].rows[0][1], "DG");
    }
}
