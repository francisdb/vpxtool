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
//! Filename is also non-canonical: some tables use `<cGameName>.txt`, some
//! use `<TableName>.txt` (a separate constant), some hard-code an unrelated
//! literal in the VBS. That makes auto-detection a glob + parse rather than
//! a name lookup - the dispatcher tries each `*.txt` in the candidate
//! folders and keeps the first one that yields a valid score block.
//!
//! The parser is deliberately strict on the *block shape* (5+5 contiguous,
//! integers then non-integer strings) but lenient on surrounding noise.
//! Tables that diverge from the 5+5 convention are reported as
//! [`LookupError::PatternNotFound`] so the caller can keep probing.

use std::path::Path;

use super::Section;

const SCORE_BLOCK_SIZE: usize = 5;
const MAX_INITIALS_LEN: usize = 4;

#[derive(Debug, PartialEq, Eq)]
pub enum LookupError {
    /// No `<5 ints, 5 short non-int strings>` window found anywhere in the
    /// file. Either it's not a Black's-style score file or it follows a
    /// variant we don't recognize.
    PatternNotFound,
    /// I/O failure reading the file.
    ReadFailed(String),
}

/// Read a Black's-style score file and return a single ranked HIGH SCORES
/// section. Entries with a score of `0` (default-zero, never-played slots)
/// are dropped to mirror PinMAME / VPReg behavior.
pub fn read_sections(path: &Path) -> Result<Vec<Section>, LookupError> {
    let raw = std::fs::read_to_string(path).map_err(|e| LookupError::ReadFailed(e.to_string()))?;
    extract_sections_from_text(&raw)
}

/// In-memory variant for tests. Split out so we can drive the parser from
/// string fixtures without writing temp files.
fn extract_sections_from_text(text: &str) -> Result<Vec<Section>, LookupError> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let (scores, names) = find_score_block(&lines).ok_or(LookupError::PatternNotFound)?;

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
        return Err(LookupError::PatternNotFound);
    }
    let ranked = rows.len() > 1;
    Ok(vec![Section {
        header: "HIGH SCORES".to_string(),
        rows,
        ranked,
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
        // 5 ints followed by 5 long strings (not initials). We require
        // the name-half to look like real initials (<=4 chars).
        let text = "100\n90\n80\n70\n60\nSentenceOne\nSentenceTwo\nSentenceThree\nSentenceFour\nSentenceFive\n";
        let err = extract_sections_from_text(text).expect_err("should not match");
        assert_eq!(err, LookupError::PatternNotFound);
    }

    #[test]
    fn rejects_window_where_names_are_all_digits() {
        // Names that are all-digit get rejected so a sequence like
        // `5 ints + 5 ints` doesn't accidentally match.
        let text = "100\n90\n80\n70\n60\n1\n2\n3\n4\n5\n";
        let err = extract_sections_from_text(text).expect_err("should not match");
        assert_eq!(err, LookupError::PatternNotFound);
    }
}
