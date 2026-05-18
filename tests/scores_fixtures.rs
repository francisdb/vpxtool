//! Parser regression tests pinned to real-shape score files in
//! `tests/fixtures/`. Each test maps a fixture to the format pattern it
//! exercises so a future parser change that breaks a known shape fails
//! loudly.
//!
//! Fixtures use synthesized placeholder data (AAA/BBB or character-name
//! defaults from the source table) - the layout is what matters.

use std::path::PathBuf;

use vpxtool::scores;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn vpreg_modern_name_before_n_van_halen() {
    // Van Halen (Original 2020) writes `HighScoreName<N>` (Name before
    // the number) rather than the more common `HighScore<N>Name`.
    let sections = scores::vpreg::read_sections(
        &fixture("vpreg_modern_name_before_n.ini"),
        "musictableVAN HALEN",
    )
    .expect("section");
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].header, "HIGH SCORES");
    assert_eq!(sections[0].rows.len(), 4);
    assert_eq!(sections[0].rows[0], vec!["#0", "AAA", "40000000", ""]);
    assert_eq!(sections[0].rows[3], vec!["#3", "DDD", "25000000", ""]);
}

#[test]
fn emhs_5plus5_with_long_character_names_cuphead() {
    // Cuphead Pro Package: Black's 5+5 shape, but the initials slots hold
    // full character names (CUPHEAD/MUGMAN/KINGDICE) rather than 3-char
    // initials.
    let sections =
        scores::emhs::read_sections(&fixture("emhs_5plus5_long_names.txt")).expect("section");
    assert_eq!(sections[0].rows.len(), 5);
    assert_eq!(sections[0].rows[0], vec!["#1", "CUPHEAD", "75000", ""]);
    assert_eq!(sections[0].rows[3], vec!["#4", "KINGDICE", "55000", ""]);
}
