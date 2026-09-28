//! `optimize` rewrites a table only when a fix applies and reports either way.

use std::path::Path;
use std::process::{Command, Output, Stdio};
use testdir::testdir;

fn vpxtool(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vpxtool"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn vpxtool")
        .wait_with_output()
        .expect("failed to wait for vpxtool")
}

#[test]
fn a_new_table_has_nothing_to_optimize_and_is_left_untouched() {
    let dir = testdir!();
    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);
    let before = std::fs::read(dir.join("table.vpx")).unwrap();

    let out = vpxtool(&dir, &["optimize", "table.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "optimize failed: {:?}", out);
    assert!(stdout.contains("table.vpx"), "unexpected output: {stdout}");
    assert!(
        stdout.contains("nothing to optimize"),
        "unexpected output: {stdout}"
    );
    assert_eq!(std::fs::read(dir.join("table.vpx")).unwrap(), before);
    assert!(!dir.join("table.vpx.tmp").exists());
}

#[test]
fn a_missing_table_fails() {
    let dir = testdir!();
    let out = vpxtool(&dir, &["optimize", "missing.vpx"]);
    assert!(!out.status.success());
}

#[test]
fn the_british_spelling_is_accepted_but_not_advertised() {
    let dir = testdir!();
    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);

    let out = vpxtool(&dir, &["optimise", "--dry-run", "table.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "optimise failed: {:?}", out);
    assert!(
        stdout.contains("nothing to optimize"),
        "unexpected output: {stdout}"
    );

    let out = vpxtool(&dir, &["--help"]);
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(help.contains("optimize"), "unexpected help: {help}");
    assert!(!help.contains("optimise"), "unexpected help: {help}");
}
