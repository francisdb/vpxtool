//! Read high scores from a GLF (Game Logic Framework) `_glf.ini` file.
//!
//! GLF is a VBScript framework for original (rom-less) VPX tables. It writes
//! per-table state to `<cGameName>_glf.ini` in the table folder, with a
//! `[HighScores]` section whose keys follow a fixed shape:
//!
//! ```ini
//! [HighScores]
//! score_1_label=GRAND CHAMPION
//! score_1_name=DAN
//! score_1_value=9000000
//! score_2_label=HIGH SCORE 1
//! score_2_name=MPC
//! score_2_value=7000000
//! ...
//! [MachineVars]
//! ...
//! ```
//!
//! The library writer (vpx-glf's `ReadHighScores`/`WriteHighScores`) groups
//! keys as `<category>_<position>_<attr>` where `attr` is one of `label`,
//! `name`, or `value`. Most tables use a single `score` category; tables with
//! mode champions can register additional categories that follow the same
//! shape. We treat each category as its own logical block and, within a block,
//! reuse the PinMAME-style "lift the distinct first label into its own
//! section" logic so output matches `--format pinemhi` for ROM tables.

use std::collections::BTreeMap;
use std::path::Path;

use ini::Ini;

use super::{Section, split_high_scores};

#[derive(Debug, PartialEq, Eq)]
pub enum LookupError {
    /// The `.ini` file has no `[HighScores]` section at all.
    NoHighScoresSection,
    /// The `[HighScores]` section is present but has no parsable
    /// `<category>_<N>_value` entries. Used when a stub file exists
    /// (initialized but never written to) so the dispatcher can fall through.
    EmptyHighScores,
    /// The `.ini` file is unreadable or malformed.
    ParseFailed(String),
}

/// Read all high-score sections from `glf_path`. Returns one or more sections
/// per category, in the same shape as the PinMAME backend.
pub fn read_sections(glf_path: &Path) -> Result<Vec<Section>, LookupError> {
    let ini = Ini::load_from_file(glf_path).map_err(|e| LookupError::ParseFailed(e.to_string()))?;
    extract_sections(&ini)
}

/// In-memory variant for tests; split out so we can drive the parser from
/// string fixtures without writing temp files.
fn extract_sections(ini: &Ini) -> Result<Vec<Section>, LookupError> {
    let section = ini
        .section(Some("HighScores"))
        .ok_or(LookupError::NoHighScoresSection)?;

    // Bucket entries by category and position. Use BTreeMaps so we get a
    // stable ordering: categories alphabetically, positions numerically.
    // Each leaf is `(label, name, value)` populated as keys are encountered.
    let mut buckets: BTreeMap<String, BTreeMap<u32, GlfEntry>> = BTreeMap::new();
    for (key, value) in section.iter() {
        let Some((category, position, attr)) = parse_glf_key(key) else {
            continue;
        };
        let entry = buckets
            .entry(category)
            .or_default()
            .entry(position)
            .or_default();
        match attr {
            GlfAttr::Label => entry.label = Some(value.trim().to_string()),
            GlfAttr::Name => entry.name = Some(value.trim().to_string()),
            GlfAttr::Value => entry.value = Some(value.trim().to_string()),
        }
    }

    // Build rows per category and run them through split_high_scores so the
    // first distinct label (typically GRAND CHAMPION) gets its own section
    // ahead of the ranked rest. Entries without a `value` are silently
    // dropped - they represent unfilled slots, mirroring PinMAME behavior.
    let mut sections = Vec::new();
    for positions in buckets.into_values() {
        let rows: Vec<Vec<String>> = positions
            .into_values()
            .filter_map(|entry| {
                let value = entry.value?;
                let label = entry.label.unwrap_or_default();
                let name = entry.name.unwrap_or_default();
                Some(vec![label, name, value, String::new()])
            })
            .collect();
        if rows.is_empty() {
            continue;
        }
        sections.extend(split_high_scores(rows));
    }

    if sections.is_empty() {
        return Err(LookupError::EmptyHighScores);
    }
    Ok(sections)
}

/// Per-position scratch state while parsing `[HighScores]` keys.
#[derive(Default)]
struct GlfEntry {
    label: Option<String>,
    name: Option<String>,
    value: Option<String>,
}

enum GlfAttr {
    Label,
    Name,
    Value,
}

/// Decode a `<category>_<position>_<attr>` key, e.g. `score_2_value`.
/// Categories themselves can contain underscores (the GLF library splits on
/// `_` greedy-from-left for the first two segments and treats the rest as
/// the attribute), so we split from the **right** to keep the category name
/// intact: the last token is the attr, the second-to-last is the position,
/// and everything before is the category.
fn parse_glf_key(key: &str) -> Option<(String, u32, GlfAttr)> {
    let (rest, attr) = key.rsplit_once('_')?;
    let attr = match attr {
        "label" => GlfAttr::Label,
        "name" => GlfAttr::Name,
        "value" => GlfAttr::Value,
        _ => return None,
    };
    let (category, position) = rest.rsplit_once('_')?;
    let position = position.parse::<u32>().ok()?;
    if category.is_empty() {
        return None;
    }
    Some((category.to_string(), position, attr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn parse(content: &str) -> Ini {
        Ini::load_from_str(content).expect("ini parse")
    }

    #[test]
    fn drops_entries_missing_value() {
        // Slots without a `_value` line are treated as unfilled.
        let ini = parse(
            r"
[HighScores]
score_1_label=GRAND CHAMPION
score_1_name=DAN
score_1_value=9000000
score_2_label=HIGH SCORE 1
score_2_name=MPC
",
        );
        let sections = extract_sections(&ini).expect("sections");
        // Only the one complete entry survives, so we get a single
        // unranked GRAND CHAMPION section (no ranked HIGH SCORES block).
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header, "GRAND CHAMPION");
        assert_eq!(sections[0].rows.len(), 1);
    }

    #[test]
    fn orders_positions_numerically_not_lexically() {
        // Verify that position 10 sorts after 9, not after 1 (which a string
        // sort would do).
        let ini = parse(
            r"
[HighScores]
score_2_label=#2
score_2_name=BBB
score_2_value=200
score_10_label=#10
score_10_name=JJJ
score_10_value=100
score_1_label=#1
score_1_name=AAA
score_1_value=300
",
        );
        let sections = extract_sections(&ini).expect("sections");
        // All labels are rank-shaped (#N), so split_high_scores keeps the
        // whole list under one ranked HIGH SCORES section.
        assert_eq!(sections.len(), 1);
        let names: Vec<&str> = sections[0].rows.iter().map(|r| r[1].as_str()).collect();
        assert_eq!(names, vec!["AAA", "BBB", "JJJ"]);
    }

    #[test]
    fn groups_multiple_categories_into_separate_section_blocks() {
        // GLF tables with mode champions register additional categories
        // following the same key shape. Each category becomes its own
        // block; ordering across categories is alphabetic (BTreeMap).
        let ini = parse(
            r"
[HighScores]
score_1_label=GRAND CHAMPION
score_1_name=AAA
score_1_value=1000
loop_champ_1_label=LOOP CHAMPION
loop_champ_1_name=BBB
loop_champ_1_value=99
",
        );
        let sections = extract_sections(&ini).expect("sections");
        assert_eq!(sections.len(), 2);
        // BTreeMap orders categories alphabetically: "loop_champ" before
        // "score". Both are single-entry so each becomes an unranked
        // section using its row's label as the header.
        assert_eq!(sections[0].header, "LOOP CHAMPION");
        assert_eq!(sections[0].rows[0][1], "BBB");
        assert_eq!(sections[1].header, "GRAND CHAMPION");
        assert_eq!(sections[1].rows[0][1], "AAA");
    }

    #[test]
    fn category_with_underscores_in_name_round_trips() {
        // `loop_champ` is two-segment; ensure we split from the right so
        // the category keeps its underscore.
        let ini = parse(
            r"
[HighScores]
loop_champ_1_label=LOOP CHAMPION
loop_champ_1_name=ABC
loop_champ_1_value=500
",
        );
        let sections = extract_sections(&ini).expect("sections");
        assert_eq!(sections[0].header, "LOOP CHAMPION");
        assert_eq!(sections[0].rows[0][1], "ABC");
        assert_eq!(sections[0].rows[0][2], "500");
    }

    #[test]
    fn returns_no_high_scores_section_when_absent() {
        let ini = parse(
            r"
[MachineVars]
won_game=0
",
        );
        let err = extract_sections(&ini).expect_err("should miss");
        assert_eq!(err, LookupError::NoHighScoresSection);
    }

    #[test]
    fn returns_empty_when_section_has_no_valued_entries() {
        // Stub file shape: section header exists but no entry has a value
        // (labels/names only). Distinct from "section missing" so the
        // dispatcher can decide whether to keep probing other backends.
        let ini = parse(
            r"
[HighScores]
score_1_label=GRAND CHAMPION
score_1_name=
",
        );
        let err = extract_sections(&ini).expect_err("should be empty");
        assert_eq!(err, LookupError::EmptyHighScores);
    }

    #[test]
    fn ignores_unrecognized_keys_in_section() {
        // Non-triple keys (`HighScoreReset=1`, stray comments processed as
        // keys, etc.) must be skipped silently.
        let ini = parse(
            r"
[HighScores]
HighScoreReset=1
random_key=garbage
score_1_label=GRAND CHAMPION
score_1_name=DAN
score_1_value=9000000
",
        );
        let sections = extract_sections(&ini).expect("sections");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header, "GRAND CHAMPION");
    }
}
