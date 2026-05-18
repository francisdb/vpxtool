//! Read high scores from an EM table's plain-text score file.
//!
//! A number of EM (and early SS) tables in VPX share a common high-score
//! storage approach attributed in their VBS comments to "Black's Highscore
//! routines" (the historical author). Each table writes a per-table
//! plain-text file under
//! `UserDirectory/` (usually `<vpx-folder>/user/`, occasionally the table
//! folder itself), one value per line. Author copy-paste churn has produced
//! a lot of variation, so there is no single canonical line layout - but
//! every variant contains an inner block of **5 ranked score integers
//! followed by 5 short initials lines**, surrounded by a varying number of
//! header/footer bytes (credits, dip settings, bonus counters, ...).
//!
//! Examples from real tables in the wild (with the score block centered):
//!
//! ```text
//! Carnaval no Rio (12 lines)        8 Ball Williams (16 lines)
//! 0        marker                   1        marker
//! 0        credits                  12       credits
//! 95510    -.                       5        dip 1
//! 84000     |                       1        dip 2
//! 73000     |- scores               1        dip 3
//! 62000     |                       0        dip 4
//! 51000    -'                       5000     -.
//! AAA      -.                       4000      |
//! ZZZ       |                       3500      |- scores
//! XXX       |- initials             3000      |
//! ABC       |                       2500     -'
//! BBB      -'                       AAA      -.
//!                                   ZZZ       |
//!                                   XXX       |- initials
//!                                   ABC       |
//!                                   BBB      -'
//! ```
//!
//! Older EM tables (typically pre-1970, e.g. "2 in 1 (Bally 1964)",
//! "4 Queens (Bally 1970)") have a simpler variant: a single high score
//! and **no initials** at all. The on-disk file is a sequence of plain
//! integers with no string lines anywhere. We handle this as a second
//! strategy: when the 5+5 scan fails, fall back to an "all-integer file
//! whose max value is the high score" rule. The all-integer anchor cleanly
//! separates these from the 5+5 format (which always has 5 string lines).
//!
//! Filename is also non-canonical: some tables use `<cGameName>.txt`, some
//! use `<TableName>.txt` (a separate constant), some hard-code an unrelated
//! literal in the VBS. That makes auto-detection a glob + parse rather than
//! a name lookup - the dispatcher tries each `*.txt` in the candidate
//! folders and keeps the first one that yields a valid score block.
//!
//! Tables that fit neither shape are reported as
//! [`LookupError::PatternNotFound`] so the caller can keep probing.

use std::path::Path;

use super::Section;

const SCORE_BLOCK_SIZE: usize = 5;
/// Upper bound on the "initials" half of a 5+5 block. Pinball tables
/// historically use 1-3 char initials, but original tables (Cuphead,
/// some Stern-tribute mods) sometimes record full character names like
/// `CUPHEAD` / `MUGMAN` / `KINGDICE` (longest seen: 8 chars). Cap at 12
/// for headroom while still rejecting prose / sentence lines.
const MAX_INITIALS_LEN: usize = 12;

#[derive(Debug, PartialEq, Eq)]
pub enum LookupError {
    /// Neither the 5-ints-then-5-initials Black's block nor the
    /// all-integer single-hisc shape was found. Either it's not an EM
    /// score file or it follows a variant we don't recognize.
    PatternNotFound,
    /// I/O failure reading the file.
    ReadFailed(String),
}

/// Read a Black's-style score file and return a single ranked HIGH SCORES
/// section. Entries with a score of `0` (default-zero, never-played slots)
/// are dropped to mirror PinMAME / VPReg behavior.
///
/// Non-UTF-8 bytes (commonly CP1252 smart-quotes / en-dashes in
/// human-authored README files that happen to share the same folder) are
/// replaced with U+FFFD via [`String::from_utf8_lossy`]. The replacement
/// chars never match the integer-or-short-initials test in the block
/// scanner, so non-score files fall through as [`LookupError::PatternNotFound`]
/// instead of bombing with a fatal read error. Real score files are pure
/// ASCII (digits + 3-char initials) so this is a no-op for them.
pub fn read_sections(path: &Path) -> Result<Vec<Section>, LookupError> {
    let bytes = std::fs::read(path).map_err(|e| LookupError::ReadFailed(e.to_string()))?;
    let raw = String::from_utf8_lossy(&bytes);
    extract_sections_from_text(&raw)
}

/// Minimum line count for the all-integer single-hisc fallback. Filters out
/// 1- or 2-integer config-y files that happen to be all numbers.
const MIN_SINGLE_HISC_LINES: usize = 4;

/// In-memory variant for tests. Split out so we can drive the parser from
/// string fixtures without writing temp files.
fn extract_sections_from_text(text: &str) -> Result<Vec<Section>, LookupError> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if let Some(sections) = try_score_block(&lines) {
        return Ok(sections);
    }
    if let Some(sections) = try_single_hisc(&lines) {
        return Ok(sections);
    }
    Err(LookupError::PatternNotFound)
}

/// First strategy: locate the canonical 5-scores-then-5-initials Black's
/// block. Returns `None` when no such block exists in the file.
fn try_score_block(lines: &[&str]) -> Option<Vec<Section>> {
    let (scores, names) = find_score_block(lines)?;
    let rows: Vec<Vec<String>> = scores
        .iter()
        .zip(names.iter())
        .enumerate()
        // Drop zero-scored slots: every Black's variant initializes empty
        // slots to 0/empty initials, matching the PinMAME convention.
        .filter(|(_, (score, _))| **score != 0)
        .map(|(i, (score, name))| {
            vec![
                format!("#{}", i + 1),
                (*name).to_string(),
                score.to_string(),
                String::new(),
            ]
        })
        .collect();
    if rows.is_empty() {
        return None;
    }
    let ranked = rows.len() > 1;
    Some(vec![Section {
        header: "HIGH SCORES".to_string(),
        rows,
        ranked,
    }])
}

/// Second strategy: single-hisc EM tables. Older EM tables (typically
/// pre-1970) store one high score with no initials. The on-disk file is
/// a small sequence of integers (credits, current-game scores, the single
/// high score, dip settings, ...) with no labels and no initials.
///
/// Anchor: the file is **all integer lines**, no string lines anywhere
/// (Black's 5+5 files always have 5 string lines, so this filter cleanly
/// separates the two formats). We additionally require [`MIN_SINGLE_HISC_LINES`]
/// or more lines so a 1-2 line config file can't trigger the fallback.
///
/// The high score itself is the **maximum integer** in the file. For these
/// tables, `hisc` dwarfs every other field (credits, current-game scores
/// during a play that hasn't finished, dip indices) - we surveyed real
/// played files (2 in 1: max=1000, 4 Queens: max=50000) and the heuristic
/// holds. Returns `None` when the file doesn't fit the all-integer shape.
fn try_single_hisc(lines: &[&str]) -> Option<Vec<Section>> {
    if lines.len() < MIN_SINGLE_HISC_LINES {
        return None;
    }
    let ints: Vec<u64> = lines
        .iter()
        .map(|l| l.parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;
    let max = *ints.iter().max()?;
    if max == 0 {
        // All zeros - unplayed slots; treat as "no high score yet" so the
        // dispatcher keeps probing other backends/files.
        return None;
    }
    Some(vec![Section {
        header: "HIGH SCORE".to_string(),
        rows: vec![vec![
            "HIGH SCORE".to_string(),
            String::new(),
            max.to_string(),
            String::new(),
        ]],
        ranked: false,
    }])
}

/// Return `([5 score ints], [5 initials strings])` for the first window where
/// 5 consecutive lines parse as non-negative integers and the next 5 are
/// short non-numeric strings (typical 3-char initials).
fn find_score_block<'a>(lines: &[&'a str]) -> Option<(Vec<u64>, Vec<&'a str>)> {
    if lines.len() < SCORE_BLOCK_SIZE * 2 {
        return None;
    }
    let max_start = lines.len() - SCORE_BLOCK_SIZE * 2;
    for start in 0..=max_start {
        let score_window = &lines[start..start + SCORE_BLOCK_SIZE];
        let names_window = &lines[start + SCORE_BLOCK_SIZE..start + SCORE_BLOCK_SIZE * 2];
        let Some(scores) = score_window
            .iter()
            .map(|l| l.parse::<u64>().ok())
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        if !names_window.iter().all(|n| looks_like_initials(n)) {
            continue;
        }
        let names: Vec<&str> = names_window.to_vec();
        return Some((scores, names));
    }
    None
}

/// Looks-like-initials: short (1..=MAX_INITIALS_LEN ASCII chars) and not
/// parseable as an integer. The integer-check excludes name fields that
/// happen to be all-digit (e.g. `"0000"` would otherwise pass the length
/// test). We do not require alphabetic-only because some tables let players
/// enter `_`, `<`, or digits in initials.
fn looks_like_initials(s: &str) -> bool {
    if s.is_empty() || s.len() > MAX_INITIALS_LEN {
        return false;
    }
    if s.parse::<u64>().is_ok() {
        return false;
    }
    s.chars().all(|c| c.is_ascii_graphic() || c == ' ')
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parses_carnaval_12_line_shape() {
        // Real Carnaval no Rio carnavalnorio.txt.
        let text = "0\n0\n95510\n84000\n73000\n62000\n51000\nAAA\nZZZ\nXXX\nABC\nBBB\n";
        let sections = extract_sections_from_text(text).expect("section");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header, "HIGH SCORES");
        assert!(sections[0].ranked);
        assert_eq!(sections[0].rows.len(), 5);
        assert_eq!(sections[0].rows[0], vec!["#1", "AAA", "95510", ""]);
        assert_eq!(sections[0].rows[4], vec!["#5", "BBB", "51000", ""]);
    }

    #[test]
    fn parses_8_ball_16_line_shape_with_six_header_bytes() {
        // Real 8 Ball Williams8Ball_66VPX.txt: six bytes of header
        // (marker, credits, four dip settings) before the score block.
        let text = "1\n12\n5\n1\n1\n0\n5000\n4000\n3500\n3000\n2500\nAAA\nZZZ\nXXX\nABC\nBBB\n";
        let sections = extract_sections_from_text(text).expect("section");
        assert_eq!(sections[0].rows.len(), 5);
        assert_eq!(sections[0].rows[0], vec!["#1", "AAA", "5000", ""]);
        assert_eq!(sections[0].rows[4], vec!["#5", "BBB", "2500", ""]);
    }

    #[test]
    fn parses_roller_coaster_18_line_shape_with_trailing_bytes() {
        // Real RollerCoaster_71VPX.txt: five header bytes, then the score
        // block, then two trailing bytes after the initials.
        let text =
            "0\n1\n5\n2\n12\n7500\n7000\n6000\n5500\n5000\nAAA\nZZZ\nXXX\nABC\nBBB\n2199\n0\n";
        let sections = extract_sections_from_text(text).expect("section");
        assert_eq!(sections[0].rows.len(), 5);
        assert_eq!(sections[0].rows[0], vec!["#1", "AAA", "7500", ""]);
        assert_eq!(sections[0].rows[4], vec!["#5", "BBB", "5000", ""]);
    }

    #[test]
    fn drops_zero_scored_slots() {
        // Slot 4 and 5 are unfilled (score 0). Result still ranked since
        // 3 slots remain, but only those three rows appear.
        let text = "100\n90\n80\n0\n0\nAAA\nBBB\nCCC\nDDD\nEEE\n";
        let sections = extract_sections_from_text(text).expect("section");
        assert_eq!(sections[0].rows.len(), 3);
        let names: Vec<&str> = sections[0].rows.iter().map(|r| r[1].as_str()).collect();
        assert_eq!(names, vec!["AAA", "BBB", "CCC"]);
    }

    #[test]
    fn read_sections_treats_non_utf8_readme_as_pattern_not_found() {
        // Real-world: README files in the same folder as the .vpx get
        // picked up by the glob; many are Windows CP1252 (smart-quote
        // 0x92, en-dash 0x96). The lossy decoder converts those to
        // U+FFFD, which fails the integer/short-initials test, so the
        // file falls through with PatternNotFound rather than aborting
        // the whole scores show invocation.
        let dir = std::env::temp_dir().join(format!("vpxtool-emhs-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir tmp");
        let path = dir.join("readme_cp1252.txt");
        // 0x92 = curly apostrophe in CP1252; invalid as standalone UTF-8.
        let bytes: &[u8] = b"Welcome to the world\x92s most famous table.\nInstructions follow.\n";
        std::fs::write(&path, bytes).expect("write fixture");
        let err = read_sections(&path).expect_err("non-utf8 readme should not parse");
        assert_eq!(err, LookupError::PatternNotFound);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn returns_pattern_not_found_when_no_block() {
        // A README-style .txt; no 5+5 window exists.
        let text =
            "This is a readme file.\nNothing here looks like a score.\nLine three.\nLine four.\n";
        let err = extract_sections_from_text(text).expect_err("should not match");
        assert_eq!(err, LookupError::PatternNotFound);
    }

    #[test]
    fn returns_pattern_not_found_when_all_zero_scored() {
        // File matches the 5+5 shape but every score is 0; after the
        // zero-filter the rows are empty so we surface PatternNotFound
        // rather than an empty section.
        let text = "0\n0\n0\n0\n0\nAAA\nBBB\nCCC\nDDD\nEEE\n";
        let err = extract_sections_from_text(text).expect_err("should not match");
        assert_eq!(err, LookupError::PatternNotFound);
    }

    #[test]
    fn finds_first_matching_window_when_multiple_could_overlap() {
        // Constructed: a leading 5-int block followed by 5 ints again,
        // then proper initials. The first window after which 5 short
        // non-ints follow should be the one we pick.
        let text = "10\n20\n30\n40\n50\n60\n70\n80\n90\n100\nAAA\nBBB\nCCC\nDDD\nEEE\n";
        let sections = extract_sections_from_text(text).expect("section");
        // Lines 5..9 (60..100) are followed by 5 initials, so that window
        // wins. Lines 0..4 (10..50) are followed by ints, so it doesn't.
        assert_eq!(sections[0].rows[0][2], "60");
        assert_eq!(sections[0].rows[4][2], "100");
    }

    #[test]
    fn rejects_window_where_names_look_too_long() {
        // 5 ints followed by 5 long strings (prose, not character names).
        // The name-half cap (MAX_INITIALS_LEN) rejects anything past the
        // longest real character name we've seen in the wild.
        let text = "100\n90\n80\n70\n60\n\
                    OneSentenceLongerThanInitials\n\
                    TwoSentenceLongerThanInitials\n\
                    ThreeSentenceLongerThanInitials\n\
                    FourSentenceLongerThanInitials\n\
                    FiveSentenceLongerThanInitials\n";
        let err = extract_sections_from_text(text).expect_err("should not match");
        assert_eq!(err, LookupError::PatternNotFound);
    }

    #[test]
    fn parses_single_hisc_2_in_1_shape() {
        // Real 2 in 1 (Bally 1964) user/2in1.txt after one game:
        // credit / score(0) / score(1) / hisc / wv / mv / qs / extra.
        let text = "11\n0\n321\n1000\n8\n7\n1\n0\n";
        let sections = extract_sections_from_text(text).expect("section");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header, "HIGH SCORE");
        assert!(!sections[0].ranked);
        assert_eq!(sections[0].rows.len(), 1);
        assert_eq!(sections[0].rows[0], vec!["HIGH SCORE", "", "1000", ""]);
    }

    #[test]
    fn parses_single_hisc_4_queens_shape() {
        // Real 4 Queens (Bally 1970) user/4Queens70.txt: 6 integer lines
        // with the high score being 50000 (well above credits/dips).
        let text = "0\n0\n0\n50000\n12\n0\n";
        let sections = extract_sections_from_text(text).expect("section");
        assert_eq!(sections[0].rows[0][2], "50000");
    }

    #[test]
    fn single_hisc_rejects_short_files() {
        // 3-line all-integer files (typically dip-only config remnants)
        // don't qualify; we require at least MIN_SINGLE_HISC_LINES.
        let text = "12\n0\n1\n";
        let err = extract_sections_from_text(text).expect_err("too short");
        assert_eq!(err, LookupError::PatternNotFound);
    }

    #[test]
    fn single_hisc_rejects_all_zero_files() {
        // A freshly-initialized score file with every slot at 0 must NOT
        // claim a high score - return PatternNotFound so the user sees
        // "no high scores yet" rather than a bogus "HIGH SCORE 0".
        let text = "0\n0\n0\n0\n0\n";
        let err = extract_sections_from_text(text).expect_err("all zero");
        assert_eq!(err, LookupError::PatternNotFound);
    }

    #[test]
    fn single_hisc_rejects_files_with_any_string_line() {
        // Anything that isn't a plain integer (or empty line) disqualifies
        // the file. Catches README-style content cleanly without needing
        // a separate length check.
        let text = "12\n0\nSomeReadme\n1000\n";
        let err = extract_sections_from_text(text).expect_err("has prose");
        assert_eq!(err, LookupError::PatternNotFound);
    }

    #[test]
    fn rejects_window_where_names_are_all_digits() {
        // The Black's 5+5 scanner must reject a "5 ints + 5 ints" run
        // (initials-half all-digit) so we don't lift counters as names.
        // A trailing non-integer line keeps the single-hisc fallback out
        // of the picture too, so this stays a PatternNotFound end-to-end.
        let text = "100\n90\n80\n70\n60\n1\n2\n3\n4\n5\nGARBAGE\n";
        let err = extract_sections_from_text(text).expect_err("should not match");
        assert_eq!(err, LookupError::PatternNotFound);
    }
}
