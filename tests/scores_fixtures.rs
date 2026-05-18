//! Parser regression tests pinned to real-shape score files in
//! `tests/fixtures/`. Each test maps a fixture to the source table whose
//! shape it captures; if a future parser change breaks a known table's
//! file shape this test fails loudly.
//!
//! Fixture data is anonymized (`AAA`/`BBB` or character-name defaults
//! from the source table) - the *layout* is what matters.
//!
//! Tests use only the public path-based API so they double as smoke
//! tests for the API surface. Edge-case / algorithmic behavior stays in
//! the unit-test modules next to the parsers.

use std::path::PathBuf;

use vpxtool::scores;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

// ---- VPReg (modern HighScoreN/HighScoreNName) -----------------------------

#[test]
fn vpreg_modern_matrix_four_entries() {
    let sections = scores::vpreg::read_sections(&fixture("vpreg_modern_matrix.ini"), "TheMatrix")
        .expect("section");
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].header, "HIGH SCORES");
    assert!(sections[0].ranked);
    assert_eq!(sections[0].rows.len(), 4);
    // SOM 1,154,150 is the real high-score entry from a played game on
    // this fixture; the rest are the table's default AAA/BBB/CCC at 100k.
    assert_eq!(sections[0].rows[0], vec!["#1", "SOM", "1154150", ""]);
    assert_eq!(sections[0].rows[3], vec!["#4", "CCC", "100000", ""]);
}

#[test]
fn vpreg_modern_volkan_score_only_no_names() {
    // Score-only ranked list (no HighScoreNName keys) - 12 entries in
    // the real file cycling through Volkan's default tiers.
    let sections =
        scores::vpreg::read_sections(&fixture("vpreg_modern_volkan_no_names.ini"), "volkan")
            .expect("section");
    assert_eq!(sections[0].rows.len(), 12);
    assert_eq!(sections[0].rows[0], vec!["#1", "", "20000", ""]);
    assert_eq!(sections[0].rows[11], vec!["#12", "", "50000", ""]);
}

#[test]
fn vpreg_modern_lochness_single_entry_unranked() {
    // One entry surfaces as an unranked single-row section headed by the
    // table name (uppercased).
    let sections = scores::vpreg::read_sections(
        &fixture("vpreg_modern_lochness_single_entry.ini"),
        "Lochness",
    )
    .expect("section");
    assert_eq!(sections.len(), 1);
    assert!(!sections[0].ranked);
    assert_eq!(sections[0].header, "LOCHNESS");
    assert_eq!(sections[0].rows.len(), 1);
}

#[test]
fn vpreg_modern_name_before_n_van_halen() {
    // `HighScoreName<N>` (Name before the number) - zero-indexed entries.
    // Real file has 4 HighScoreN rows plus many unrelated Track*Score /
    // OldScoreN keys that the parser ignores (no HighScore prefix).
    let sections = scores::vpreg::read_sections(
        &fixture("vpreg_modern_name_before_n.ini"),
        "musictableVAN HALEN",
    )
    .expect("section");
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].rows.len(), 4);
    assert_eq!(sections[0].rows[0], vec!["#0", "BRIAN", "40000000", ""]);
    assert_eq!(sections[0].rows[3], vec!["#3", "RANDY", "25000000", ""]);
}

// ---- VPReg (legacy hiscore/hsa) -------------------------------------------

#[test]
fn vpreg_legacy_abracadabra_lowercase() {
    // `hiscore=10000` + lowercase `hsa1=4 hsa2=15 hsa3=7` -> "DOG"
    // in the 1-indexed alphabet `ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_<`.
    let sections =
        scores::vpreg::read_sections(&fixture("vpreg_legacy_abracadabra.ini"), "Abra_Ca_Dabra")
            .expect("section");
    assert_eq!(sections.len(), 1);
    assert!(!sections[0].ranked);
    assert_eq!(sections[0].header, "ABRA_CA_DABRA");
    assert_eq!(sections[0].rows[0], vec!["HIGH SCORE", "DOG", "10000", ""]);
}

#[test]
fn vpreg_legacy_a_go_go_uppercase() {
    // CamelCase `HighScore=719` + `HSA1=19 HSA2=15 HSA3=13` -> "SOM".
    // The parser does case-insensitive key matching.
    let sections =
        scores::vpreg::read_sections(&fixture("vpreg_legacy_a_go_go_uppercase.ini"), "A-Go-Go")
            .expect("section");
    assert_eq!(sections[0].rows[0], vec!["HIGH SCORE", "SOM", "719", ""]);
}

// ---- GLF (vpx-glf framework) ----------------------------------------------

#[test]
fn glf_dark_chaos_grand_champion_split() {
    // First label is distinct ("GRAND CHAMPION") so it's lifted into its
    // own unranked section; the remaining three become a ranked HIGH SCORES.
    let sections = scores::glf::read_sections(&fixture("glf_dark_chaos.ini")).expect("section");
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].header, "GRAND CHAMPION");
    assert!(!sections[0].ranked);
    assert_eq!(sections[0].rows[0][1], "DAN");
    assert_eq!(sections[0].rows[0][2], "9000000");
    assert_eq!(sections[1].header, "HIGH SCORES");
    assert!(sections[1].ranked);
    assert_eq!(sections[1].rows.len(), 3);
}

// ---- EMHS Black's 5+5 -----------------------------------------------------

#[test]
fn emhs_5plus5_carnaval() {
    let sections =
        scores::emhs::read_sections(&fixture("emhs_5plus5_carnaval.txt")).expect("section");
    assert_eq!(sections[0].header, "HIGH SCORES");
    assert!(sections[0].ranked);
    assert_eq!(sections[0].rows.len(), 5);
    assert_eq!(sections[0].rows[0], vec!["#1", "AAA", "95510", ""]);
    assert_eq!(sections[0].rows[4], vec!["#5", "BBB", "51000", ""]);
}

#[test]
fn emhs_5plus5_8ball_with_six_header_bytes() {
    // Six bytes of header (marker, credits, four dip settings) before
    // the score block.
    let sections = scores::emhs::read_sections(&fixture("emhs_5plus5_8ball.txt")).expect("section");
    assert_eq!(sections[0].rows.len(), 5);
    assert_eq!(sections[0].rows[0], vec!["#1", "AAA", "5000", ""]);
    assert_eq!(sections[0].rows[4], vec!["#5", "BBB", "2500", ""]);
}

#[test]
fn emhs_5plus5_roller_coaster_with_trailing_bytes() {
    // Five header bytes + score block + two trailing bytes after the
    // initials.
    let sections =
        scores::emhs::read_sections(&fixture("emhs_5plus5_roller_coaster.txt")).expect("section");
    assert_eq!(sections[0].rows.len(), 5);
    assert_eq!(sections[0].rows[0], vec!["#1", "AAA", "7500", ""]);
    assert_eq!(sections[0].rows[4], vec!["#5", "BBB", "5000", ""]);
}

#[test]
fn emhs_5plus5_with_long_character_names_cuphead() {
    // Black's 5+5 shape, but the initials slots hold full character
    // names (CUPHEAD/MUGMAN/KINGDICE) rather than 3-char initials.
    let sections =
        scores::emhs::read_sections(&fixture("emhs_5plus5_long_names.txt")).expect("section");
    assert_eq!(sections[0].rows.len(), 5);
    assert_eq!(sections[0].rows[0], vec!["#1", "CUPHEAD", "75000", ""]);
    assert_eq!(sections[0].rows[3], vec!["#4", "KINGDICE", "55000", ""]);
}

// ---- EMHS single-hisc (no initials) ---------------------------------------

#[test]
fn emhs_single_2in1() {
    // 8-line all-integer file; the maximum is the high score.
    let sections = scores::emhs::read_sections(&fixture("emhs_single_2in1.txt")).expect("section");
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].header, "HIGH SCORE");
    assert!(!sections[0].ranked);
    assert_eq!(sections[0].rows[0], vec!["HIGH SCORE", "", "1000", ""]);
}

#[test]
fn emhs_single_4queens() {
    // Real played file: `32040 1 1 50000 5 5`. Max value (50000) is the
    // high score.
    let sections =
        scores::emhs::read_sections(&fixture("emhs_single_4queens.txt")).expect("section");
    assert_eq!(sections[0].rows[0], vec!["HIGH SCORE", "", "50000", ""]);
}
