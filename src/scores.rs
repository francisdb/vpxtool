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

use serde_json::Value;

const SCORE_ARRAYS: &[&str] = &["high_scores", "mode_champions", "more_mode_champions"];

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

/// Extract score rows.
///
/// Each row has four columns: `LABEL`, `INITIALS`, `SCORE` (raw, no thousands
/// grouping, no unit-based formatting), and `UNITS` (empty string when the
/// map has no `units` annotation). Entries whose numeric value is zero
/// (cleared slots, e.g. Medieval Madness's empty King-of-the-Realm
/// positions) are dropped to match PINemHi's "no entry recorded" behavior.
pub fn extract_rows(resolved: &Value) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    for key in SCORE_ARRAYS {
        let Some(entries) = resolved.get(*key).and_then(|v| v.as_array()) else {
            continue;
        };
        for entry in entries {
            if let Some(row) = score_row_from_entry(entry) {
                rows.push(row);
            }
        }
    }
    rows
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
/// * everything else -> thousands grouping for integers, pass-through for
///   non-integer strings (already-formatted times, etc.)
pub fn pretty_score_column(rows: &mut [Vec<String>]) {
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
                    *score = group_thousands_u64(n);
                } else if let Ok(n) = score.parse::<i64>() {
                    let abs = group_thousands_u64(n.unsigned_abs());
                    *score = if n < 0 { format!("-{abs}") } else { abs };
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

fn group_thousands_u64(mut n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut out = String::new();
    let mut group_digits = 0;
    while n > 0 {
        if group_digits == 3 {
            out.push(',');
            group_digits = 0;
        }
        out.push(char::from_digit((n % 10) as u32, 10).unwrap());
        n /= 10;
        group_digits += 1;
    }
    out.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

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
        pretty_score_column(&mut rows);
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
        pretty_score_column(&mut rows);
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
        pretty_score_column(&mut rows);
        assert_eq!(rows[0][COL_SCORE], "3:45:07");
    }

    #[test]
    fn pretty_score_column_leaves_string_scores_alone() {
        // Pre-formatted scores (timer strings written into the map as a
        // string value, etc.) must pass through unchanged.
        let mut rows = vec![vec!["A".into(), "AAA".into(), "10:00.00".into(), "".into()]];
        pretty_score_column(&mut rows);
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
    fn group_thousands_handles_zero_and_small() {
        assert_eq!(group_thousands_u64(0), "0");
        assert_eq!(group_thousands_u64(7), "7");
        assert_eq!(group_thousands_u64(999), "999");
        assert_eq!(group_thousands_u64(1000), "1,000");
    }
}
