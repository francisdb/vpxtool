//! `diff` compares two vpx files and exits non-zero when they differ.

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
fn diff_reports_no_differences_for_identical_files() {
    let dir = testdir!();
    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);
    std::fs::copy(dir.join("table.vpx"), dir.join("copy.vpx")).expect("copy vpx");

    let out = vpxtool(&dir, &["diff", "table.vpx", "copy.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "diff failed: {:?}", out);
    assert!(stdout.trim().is_empty(), "unexpected output: {stdout}");
}

#[test]
fn diff_reports_changed_record_and_exits_non_zero() {
    let dir = testdir!();
    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);
    std::fs::copy(dir.join("table.vpx"), dir.join("locked.vpx")).expect("copy vpx");
    // locking bumps the TLCK counter in the game data stream
    let out = vpxtool(&dir, &["lock", "locked.vpx"]);
    assert!(out.status.success(), "vpxtool lock failed: {:?}", out);

    let out = vpxtool(&dir, &["diff", "table.vpx", "locked.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "expected exit 1: {:?}", out);
    assert!(
        stdout.contains("/GameStg/GameData") && stdout.contains("TLCK"),
        "unexpected output: {stdout}"
    );
}
