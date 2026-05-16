//! Extract human-readable score rows from a resolved pinmame-nvram JSON value.
//!
//! The pinmame-nvram crate produces a `serde_json::Value` with score-bearing
//! arrays (`high_scores`, `mode_champions`, `more_mode_champions`). This
//! module turns those into a flat list of `(label, initials, score, units)`
//! rows, applying the same "drop empty slots" filter PINemHi does so output
//! line counts match.
//!
//! Scores come out **raw** (no thousands grouping, no units-aware formatting)
//! so callers emitting machine-friendly formats (TSV, JSON) can use them as-is.
//! Use [`pretty_score_column`] to apply comma grouping and unit-based
//! formatting for the aligned-table human view.

use num_format::{Format, ToFormattedString};
use serde_json::Value;

pub mod vpreg;

/// Numeric value keys we accept on a score entry, in priority order. Maps for
/// regular high-scores use `score`; mode-champion maps use `counter` and
/// occasionally `nth time`.
const NUMERIC_KEYS: &[&str] = &["score", "counter", "nth time"];

/// Column indices for rows produced by [`extract_rows`].
pub const COL_LABEL: usize = 0;
pub const COL_INITIALS: usize = 1;
pub const COL_SCORE: usize = 2;
pub const COL_UNITS: usize = 3;

/// Header labels matching the columns above. TSV output emits all four; the
/// aligned table emits the first three.
pub const HEADERS: [&str; 4] = ["LABEL", "INITIALS", "SCORE", "UNITS"];

/// Extract score rows as a flat list.
///
/// Each row has four columns: `LABEL`, `INITIALS`, `SCORE` (raw, no thousands
/// grouping, no unit-based formatting), and `UNITS` (empty string when the
/// map has no `units` annotation). Entries whose numeric value is zero
/// (cleared slots, e.g. Medieval Madness's empty King-of-the-Realm
/// positions) are dropped to match PINemHi's "no entry recorded" behavior.
///
/// This is the flat view; [`extract_sections`] produces the same data
/// grouped into PINemHi-style sections.
pub fn extract_rows(resolved: &Value) -> Vec<Vec<String>> {
    extract_sections(resolved)
        .into_iter()
        .flat_map(|s| s.rows)
        .collect()
}

/// A logical group of score rows that share a section header (e.g. a single
/// "HIGH SCORES" block, or one mode-champion's mini-block).
#[derive(Debug, PartialEq, Eq)]
pub struct Section {
    /// Uppercased header text to print above the rows (e.g. "GRAND CHAMPION",
    /// "HIGH SCORES", or the label of a single mode champion).
    pub header: String,
    /// Rows in this section, in the same 4-column shape as [`extract_rows`].
    pub rows: Vec<Vec<String>>,
    /// True when the section is a ranked list (multiple entries that should
    /// be rendered with `1.`, `2.`, ... prefixes). False for single-entry
    /// sections (champion records, top scorer pulled out of a list).
    pub ranked: bool,
}

/// Extract score rows grouped into PINemHi-style sections.
///
/// Sections are produced in this order:
///   1. If `high_scores` is present:
///      * single entry: one section, header = entry label uppercase
///      * multiple entries, first label distinct from rest: split into
///        a one-entry top section (e.g. `GRAND CHAMPION`) plus a ranked
///        `HIGH SCORES` section for the rest
///      * multiple entries, all rank-like: one ranked `HIGH SCORES` section
///   2. Each `mode_champions` / `more_mode_champions` row becomes its own
///      one-entry section.
pub fn extract_sections(resolved: &Value) -> Vec<Section> {
    let mut sections = Vec::new();

    if let Some(entries) = resolved.get("high_scores").and_then(|v| v.as_array()) {
        let rows: Vec<Vec<String>> = entries.iter().filter_map(score_row_from_entry).collect();
        sections.extend(split_high_scores(rows));
    }

    for key in ["mode_champions", "more_mode_champions"] {
        let Some(entries) = resolved.get(key).and_then(|v| v.as_array()) else {
            continue;
        };
        for entry in entries {
            if let Some(row) = score_row_from_entry(entry) {
                sections.push(Section {
                    header: row[COL_LABEL].to_uppercase(),
                    rows: vec![row],
                    ranked: false,
                });
            }
        }
    }

    sections
}

/// Split the rows of a `high_scores` array into one or two sections per the
/// rules documented on [`extract_sections`].
fn split_high_scores(rows: Vec<Vec<String>>) -> Vec<Section> {
    match rows.len() {
        0 => Vec::new(),
        1 => vec![Section {
            header: rows[0][COL_LABEL].to_uppercase(),
            rows,
            ranked: false,
        }],
        _ => {
            let first_label = &rows[0][COL_LABEL];
            // If the first label is already rank-shaped (e.g. "First Place",
            // "#1", "High Score #1") we don't pull it out; the whole array
            // belongs under one ranked HIGH SCORES section. Otherwise the
            // first entry is a distinct top label like "Grand Champion" /
            // "World Record" and we lift it into its own one-entry section.
            if is_ranked_label(first_label) {
                vec![Section {
                    header: "HIGH SCORES".to_string(),
                    rows,
                    ranked: true,
                }]
            } else {
                let mut iter = rows.into_iter();
                let top = iter.next().unwrap();
                let rest: Vec<_> = iter.collect();
                let top_header = top[COL_LABEL].to_uppercase();
                let mut sections = vec![Section {
                    header: top_header,
                    rows: vec![top],
                    ranked: false,
                }];
                if !rest.is_empty() {
                    sections.push(Section {
                        header: "HIGH SCORES".to_string(),
                        rows: rest,
                        ranked: true,
                    });
                }
                sections
            }
        }
    }
}

/// Does this label look like a positional ranking ("First Place", "1st",
/// "#1", "High Score #2") rather than a distinct top label ("Grand
/// Champion", "World Record", ...)? Used to decide whether to lift the
/// first high-score entry into its own section.
fn is_ranked_label(label: &str) -> bool {
    let l = label.trim().to_ascii_lowercase();
    if l.starts_with('#') {
        return true;
    }
    const RANKED_PREFIXES: &[&str] = &[
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
        "tenth", "1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th", "10th",
    ];
    if RANKED_PREFIXES.iter().any(|p| l.starts_with(p)) {
        return true;
    }
    // Labels like "High Score #1", "Standings #1" - has a # somewhere.
    l.contains('#')
}

/// Render sections in PINemHi-style: each section's uppercase header on its
/// own line, then rows below (ranked sections prefix each row with `N.`,
/// unranked single-entry sections drop the rank). Empty initials collapse to
/// score-only. Section bodies use the [`pretty_score_column`] formatting
/// (thousands grouping per `locale`, seconds-to-time).
///
/// Columns are padded per-section so the rank, initials, and score columns
/// line up: rank is left-padded to the largest rank's width, initials are
/// left-padded to the widest initials in the section, scores are
/// right-padded so digits align. Widths use char count (not byte count) so
/// locales whose thousands separator is multi-byte (e.g. fr_FR's U+202F)
/// still align.
///
/// `locale` is generic over `num_format::Format` rather than `&dyn Format`
/// because the underlying `ToFormattedString::to_formatted_string` requires
/// `Sized`. Callers that want runtime locale selection should branch on the
/// chosen locale type and dispatch into a monomorphized copy per arm.
pub fn render_pinemhi<F: Format>(sections: &[Section], locale: &F) -> String {
    let mut out = String::new();
    for (idx, section) in sections.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        out.push_str(&section.header);
        out.push('\n');
        // Apply table-style score formatting (locale-aware grouping / mm:ss)
        // without touching the raw row data.
        let mut formatted = section.rows.clone();
        pretty_score_column(&mut formatted, locale);

        let initials_w = formatted
            .iter()
            .map(|r| r.get(COL_INITIALS).map_or(0, |s| s.chars().count()))
            .max()
            .unwrap_or(0);
        let score_w = formatted
            .iter()
            .map(|r| r.get(COL_SCORE).map_or(0, |s| s.chars().count()))
            .max()
            .unwrap_or(0);
        // Width of the largest rank label ("10." -> 3) so two-digit ranks
        // don't push the initials column out of alignment.
        let rank_w = if section.ranked {
            format!("{}.", formatted.len()).chars().count()
        } else {
            0
        };

        for (i, row) in formatted.iter().enumerate() {
            let initials = row.get(COL_INITIALS).map(String::as_str).unwrap_or("");
            let score = row.get(COL_SCORE).map(String::as_str).unwrap_or("");
            let mut line = String::new();
            if section.ranked {
                let rank = format!("{}.", i + 1);
                line.push_str(&format!("{rank:<rank_w$} "));
            }
            if initials_w > 0 {
                line.push_str(&format!("{initials:<initials_w$}"));
                if score_w > 0 {
                    line.push_str("  ");
                }
            }
            if score_w > 0 {
                line.push_str(&format!("{score:>score_w$}"));
            }
            // Drop trailing padding so rows missing a score/initials don't
            // emit stray whitespace at end-of-line.
            out.push_str(line.trim_end());
            out.push('\n');
        }
    }
    out
}

/// Convert one entry of a score array into a `[label, initials, score, units]`
/// row, or `None` if it is an empty / cleared slot. The `score` column is the
/// raw numeric value (or pre-formatted string from the map) without thousands
/// grouping; the `units` column is the map's `units` annotation (e.g.
/// `"seconds"`) or an empty string. Callers needing a human view should pass
/// the rows through [`pretty_score_column`].
fn score_row_from_entry(entry: &Value) -> Option<Vec<String>> {
    let label = entry.get("label").and_then(|v| v.as_str()).unwrap_or("");
    // Some maps (e.g. Stern Whitestar's LotR) reserve a 10-char initials
    // field for what's typically a 3-char value, padding the rest with
    // spaces. Trim that trailing padding so TSV output is clean and table
    // alignment is driven by the renderer, not by the map's raw width.
    let initials = entry
        .get("initials")
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_str())
        .map(str::trim_end)
        .unwrap_or("");
    // Locate the score-bearing object (e.g. {"value": 600, "units": "seconds"})
    // so we can pick both the numeric value and the units annotation from it.
    let score_obj = NUMERIC_KEYS.iter().find_map(|k| entry.get(*k));
    let numeric_value = score_obj.and_then(|o| o.get("value"));
    let units = score_obj
        .and_then(|o| o.get("units"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let score_str = match numeric_value {
        Some(Value::Number(n)) => {
            // Cleared / never-set slots get reported as zero by the maps;
            // PINemHi suppresses them, so we do too. Keep non-zero numbers,
            // including legitimately-low ones (a "world record" of 100 is
            // still a recorded score).
            if number_is_zero(n) {
                return None;
            }
            n.to_string()
        }
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    };

    // Drop entries that produced no displayable content at all.
    if label.is_empty() && initials.is_empty() && score_str.is_empty() {
        return None;
    }
    Some(vec![
        label.to_string(),
        initials.to_string(),
        score_str,
        units.to_string(),
    ])
}

/// Render the SCORE column (`COL_SCORE`) in place for the aligned-table human
/// view, using the UNITS column (`COL_UNITS`) as a formatting hint:
///
/// * `units: "seconds"` -> `mm:ss` (or `h:mm:ss` past an hour)
/// * everything else -> thousands grouping for integers (per `locale`),
///   pass-through for non-integer strings (already-formatted times, etc.)
///
/// `locale` controls the thousands separator: pass a `num_format::Locale`
/// (e.g. `Locale::en` for `1,234,567`) or a `SystemLocale` to follow the
/// user's `LC_ALL`/`LANG`/`LC_NUMERIC` env at runtime.
pub fn pretty_score_column<F: Format>(rows: &mut [Vec<String>], locale: &F) {
    for row in rows {
        if row.len() <= COL_SCORE {
            continue;
        }
        let units = row.get(COL_UNITS).cloned().unwrap_or_default();
        let score = &mut row[COL_SCORE];
        match units.as_str() {
            "seconds" => {
                if let Ok(n) = score.parse::<u64>() {
                    *score = format_seconds_as_time(n);
                }
            }
            _ => {
                if let Ok(n) = score.parse::<u64>() {
                    *score = n.to_formatted_string(locale);
                } else if let Ok(n) = score.parse::<i64>() {
                    *score = n.to_formatted_string(locale);
                }
                // Non-integer string (pre-formatted time etc.) - leave alone.
            }
        }
    }
}

/// Format an integer number of seconds as `mm:ss`, or `h:mm:ss` once it
/// reaches an hour. Used for `units: "seconds"` scores (LotR's Destroy Ring
/// Champion is the canonical example - 600 -> `10:00`).
fn format_seconds_as_time(total: u64) -> String {
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn number_is_zero(n: &serde_json::Number) -> bool {
    if let Some(u) = n.as_u64() {
        return u == 0;
    }
    if let Some(i) = n.as_i64() {
        return i == 0;
    }
    if let Some(f) = n.as_f64() {
        return f == 0.0;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_format::Locale;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    /// Pinned locale for tests so output is reproducible regardless of the
    /// developer's `LC_ALL` / `LC_NUMERIC` setting. Production callers
    /// use `SystemLocale::default()` to honor the user's env.
    const TEST_LOCALE: &Locale = &Locale::en;

    #[test]
    fn extracts_high_scores_in_array_order_with_raw_numeric_scores() {
        let resolved = json!({
            "high_scores": [
                {"label": "Grand Champion", "initials": {"value": "JBJ"}, "score": {"value": 180000000}},
                {"label": "First Place",    "initials": {"value": "DRF"}, "score": {"value": 165000000}},
            ]
        });
        // extract_rows returns raw scores (no commas) and an empty units
        // column when the map doesn't annotate one. Machine consumers can
        // use the values verbatim; pretty_score_column adds the grouping
        // for the aligned human view.
        assert_eq!(
            extract_rows(&resolved),
            vec![
                vec![
                    "Grand Champion".to_string(),
                    "JBJ".to_string(),
                    "180000000".to_string(),
                    "".to_string(),
                ],
                vec![
                    "First Place".to_string(),
                    "DRF".to_string(),
                    "165000000".to_string(),
                    "".to_string(),
                ],
            ]
        );
    }

    #[test]
    fn extracts_units_when_map_provides_them() {
        // LotR Destroy Ring Champion is annotated `units: "seconds"` by
        // pinmame-nvram >=0.4.9; the row must surface it so the table
        // renderer can format the score as a time.
        let resolved = json!({
            "mode_champions": [
                {"label": "Destroy Ring Champion", "initials": {"value": "EYE"},
                 "score": {"value": 600, "units": "seconds"}},
            ]
        });
        let rows = extract_rows(&resolved);
        assert_eq!(rows[0][COL_SCORE], "600");
        assert_eq!(rows[0][COL_UNITS], "seconds");
    }

    #[test]
    fn pretty_score_column_groups_integer_score_thousands() {
        let mut rows = vec![
            vec!["A".into(), "AAA".into(), "180000000".into(), "".into()],
            vec!["B".into(), "BBB".into(), "7".into(), "".into()],
        ];
        pretty_score_column(&mut rows, TEST_LOCALE);
        assert_eq!(rows[0][COL_SCORE], "180,000,000");
        assert_eq!(rows[1][COL_SCORE], "7");
    }

    #[test]
    fn pretty_score_column_renders_seconds_as_time() {
        // LotR Destroy Ring Champion: 600 seconds -> 10:00.
        let mut rows = vec![vec![
            "Destroy Ring Champion".into(),
            "EYE".into(),
            "600".into(),
            "seconds".into(),
        ]];
        pretty_score_column(&mut rows, TEST_LOCALE);
        assert_eq!(rows[0][COL_SCORE], "10:00");
    }

    #[test]
    fn pretty_score_column_renders_long_seconds_as_h_mm_ss() {
        // 3 hours 45 minutes 7 seconds.
        let mut rows = vec![vec![
            "Marathon".into(),
            "AAA".into(),
            "13507".into(),
            "seconds".into(),
        ]];
        pretty_score_column(&mut rows, TEST_LOCALE);
        assert_eq!(rows[0][COL_SCORE], "3:45:07");
    }

    #[test]
    fn pretty_score_column_leaves_string_scores_alone() {
        // Pre-formatted scores (timer strings written into the map as a
        // string value, etc.) must pass through unchanged.
        let mut rows = vec![vec!["A".into(), "AAA".into(), "10:00.00".into(), "".into()]];
        pretty_score_column(&mut rows, TEST_LOCALE);
        assert_eq!(rows[0][COL_SCORE], "10:00.00");
    }

    #[test]
    fn appends_mode_champions_after_high_scores() {
        let resolved = json!({
            "high_scores": [
                {"label": "GC", "initials": {"value": "AAA"}, "score": {"value": 1000}},
            ],
            "mode_champions": [
                {"label": "Castle Champion", "initials": {"value": "JCY"}, "score": {"value": 6}},
            ],
        });
        let rows = extract_rows(&resolved);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], "GC");
        assert_eq!(rows[1][0], "Castle Champion");
    }

    #[test]
    fn drops_entries_with_zero_numeric_value() {
        // Mirrors MM's unset King-of-the-Realm slots: initials still set
        // from a previous wipe, counter zero. PINemHi suppresses these; we do too.
        let resolved = json!({
            "mode_champions": [
                {"label": "King #1", "initials": {"value": "KOP"}, "counter": {"value": 1}},
                {"label": "King #2", "initials": {"value": "KOP"}, "counter": {"value": 0}},
                {"label": "King #3", "initials": {"value": "KOP"}, "counter": {"value": 0}},
            ]
        });
        let rows = extract_rows(&resolved);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "King #1");
    }

    #[test]
    fn falls_back_through_score_then_counter_then_nth_time() {
        // The `nth time` key (with a space) is real - shows up on
        // King-of-the-Realm-style entries alongside `counter`.
        let resolved = json!({
            "mode_champions": [
                {"label": "Old style", "initials": {"value": "ABC"}, "nth time": {"value": 7}},
            ]
        });
        let rows = extract_rows(&resolved);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][2], "7");
    }

    #[test]
    fn accepts_string_score_values() {
        // Some maps express counters as already-formatted strings (e.g.
        // "10:00.00" for a destroy-the-ring timer). Pass them through.
        let resolved = json!({
            "mode_champions": [
                {"label": "Destroy Ring Champion", "initials": {"value": "EYE"}, "score": {"value": "10:00.00"}},
            ]
        });
        let rows = extract_rows(&resolved);
        assert_eq!(rows[0][2], "10:00.00");
    }

    #[test]
    fn returns_empty_when_resolved_has_no_score_arrays() {
        let resolved = json!({"audits": [], "game_state": {}});
        assert!(extract_rows(&resolved).is_empty());
    }

    #[test]
    fn pretty_score_column_uses_passed_locale_for_grouping_separator() {
        // PINemHi honors LC_ALL/LANG for the thousands separator. We want
        // the same behavior, threaded through the locale parameter rather
        // than implicit global state.
        let mut rows_en = vec![vec!["A".into(), "AAA".into(), "1234567".into(), "".into()]];
        pretty_score_column(&mut rows_en, &Locale::en);
        assert_eq!(rows_en[0][COL_SCORE], "1,234,567");

        let mut rows_de = vec![vec!["A".into(), "AAA".into(), "1234567".into(), "".into()]];
        pretty_score_column(&mut rows_de, &Locale::de);
        assert_eq!(rows_de[0][COL_SCORE], "1.234.567");
    }

    #[test]
    fn extract_sections_splits_grand_champion_from_ranked_rest() {
        let resolved = json!({
            "high_scores": [
                {"label": "Grand Champion", "initials": {"value": "SLL"}, "score": {"value": 52000000}},
                {"label": "First Place",    "initials": {"value": "BRE"}, "score": {"value": 44000000}},
                {"label": "Second Place",   "initials": {"value": "LFS"}, "score": {"value": 40000000}},
            ]
        });
        let sections = extract_sections(&resolved);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].header, "GRAND CHAMPION");
        assert_eq!(sections[0].ranked, false);
        assert_eq!(sections[0].rows.len(), 1);
        assert_eq!(sections[1].header, "HIGH SCORES");
        assert_eq!(sections[1].ranked, true);
        assert_eq!(sections[1].rows.len(), 2);
    }

    #[test]
    fn extract_sections_keeps_ranked_list_together_when_no_distinct_top() {
        // bttf_a27-style "WORLD RECORD" pulled out is one case; here we
        // simulate a table whose entries are all rank-labeled with no
        // separate top score.
        let resolved = json!({
            "high_scores": [
                {"label": "First Place",  "initials": {"value": "AAA"}, "score": {"value": 1000}},
                {"label": "Second Place", "initials": {"value": "BBB"}, "score": {"value": 900}},
                {"label": "Third Place",  "initials": {"value": "CCC"}, "score": {"value": 800}},
            ]
        });
        let sections = extract_sections(&resolved);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header, "HIGH SCORES");
        assert_eq!(sections[0].ranked, true);
        assert_eq!(sections[0].rows.len(), 3);
    }

    #[test]
    fn extract_sections_keeps_ranked_list_for_hash_labels() {
        // LotR-style "#1", "#2", ... labels are rank-shaped; treat them
        // like any other ranked list (no Grand Champion split).
        let resolved = json!({
            "high_scores": [
                {"label": "#1", "initials": {"value": "AAA"}, "score": {"value": 1000}},
                {"label": "#2", "initials": {"value": "BBB"}, "score": {"value": 900}},
            ]
        });
        let sections = extract_sections(&resolved);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header, "HIGH SCORES");
        assert_eq!(sections[0].ranked, true);
    }

    #[test]
    fn extract_sections_single_high_score_uses_its_label_as_header() {
        // vector-style "High Score" single-entry table.
        let resolved = json!({
            "high_scores": [
                {"label": "High Score", "initials": {"value": ""}, "score": {"value": 1989660}},
            ]
        });
        let sections = extract_sections(&resolved);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header, "HIGH SCORE");
        assert_eq!(sections[0].ranked, false);
    }

    #[test]
    fn extract_sections_emits_one_section_per_mode_champion() {
        let resolved = json!({
            "high_scores": [
                {"label": "Grand Champion", "initials": {"value": "SLL"}, "score": {"value": 1000}},
            ],
            "mode_champions": [
                {"label": "Castle Champion", "initials": {"value": "JCY"}, "score": {"value": 6}},
                {"label": "Joust Champion",  "initials": {"value": "DWF"}, "score": {"value": 5}},
            ]
        });
        let sections = extract_sections(&resolved);
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].header, "GRAND CHAMPION");
        assert_eq!(sections[1].header, "CASTLE CHAMPION");
        assert_eq!(sections[2].header, "JOUST CHAMPION");
        assert!(sections.iter().all(|s| !s.ranked));
    }

    #[test]
    fn render_pinemhi_emits_section_headers_and_ranked_bodies() {
        let resolved = json!({
            "high_scores": [
                {"label": "Grand Champion", "initials": {"value": "SLL"}, "score": {"value": 52000000}},
                {"label": "First Place",    "initials": {"value": "BRE"}, "score": {"value": 44000000}},
            ]
        });
        let sections = extract_sections(&resolved);
        let rendered = render_pinemhi(&sections, TEST_LOCALE);
        assert_eq!(
            rendered,
            "GRAND CHAMPION\nSLL  52,000,000\n\nHIGH SCORES\n1. BRE  44,000,000\n"
        );
    }

    #[test]
    fn render_pinemhi_aligns_score_column_when_initials_widths_differ() {
        // Baywatch-style: most entries have 3-char initials but one is
        // shorter. Score column must stay aligned vertically.
        let resolved = json!({
            "high_scores": [
                {"label": "#1", "initials": {"value": "JEK"}, "score": {"value": 2_400_000_000_u64}},
                {"label": "#2", "initials": {"value": "LON"}, "score": {"value": 2_100_000_000_u64}},
                {"label": "#3", "initials": {"value": "NF"},  "score": {"value": 1_950_000_000_u64}},
            ]
        });
        let sections = extract_sections(&resolved);
        let rendered = render_pinemhi(&sections, TEST_LOCALE);
        assert_eq!(
            rendered,
            "HIGH SCORES\n\
             1. JEK  2,400,000,000\n\
             2. LON  2,100,000,000\n\
             3. NF   1,950,000,000\n"
        );
    }

    #[test]
    fn render_pinemhi_pads_rank_for_two_digit_ranks() {
        // Sections with >=10 rows must pad single-digit ranks so the
        // initials column doesn't jog left on rows 1-9.
        let rows: Vec<_> = (1..=10)
            .map(|i| {
                serde_json::json!({
                    "label": format!("#{i}"),
                    "initials": {"value": "AAA"},
                    "score": {"value": 1000 - i * 10},
                })
            })
            .collect();
        let resolved = serde_json::json!({ "high_scores": rows });
        let sections = extract_sections(&resolved);
        let rendered = render_pinemhi(&sections, TEST_LOCALE);
        // Lines 1-9 prefix is "1.  ".."9.  " (rank padded to width 3);
        // line 10 is "10. ".
        assert!(rendered.contains("\n1.  AAA  990\n"));
        assert!(rendered.contains("\n9.  AAA  910\n"));
        assert!(rendered.contains("\n10. AAA  900\n"));
    }

    #[test]
    fn render_pinemhi_drops_trailing_whitespace_when_score_is_empty() {
        // wd_12's THE ROOF CHAMPION case - initials present, no numeric score.
        let resolved = json!({
            "mode_champions": [
                {"label": "The Roof Champion", "initials": {"value": "XAQ"}}
            ]
        });
        let sections = extract_sections(&resolved);
        let rendered = render_pinemhi(&sections, TEST_LOCALE);
        assert_eq!(rendered, "THE ROOF CHAMPION\nXAQ\n");
    }

    #[test]
    fn render_pinemhi_omits_initials_line_when_no_initials() {
        // EM/early-SS-style table with just a score, no initials.
        let resolved = json!({
            "high_scores": [
                {"label": "High Score", "initials": {"value": ""}, "score": {"value": 1989660}},
            ]
        });
        let sections = extract_sections(&resolved);
        let rendered = render_pinemhi(&sections, TEST_LOCALE);
        assert_eq!(rendered, "HIGH SCORE\n1,989,660\n");
    }
}
