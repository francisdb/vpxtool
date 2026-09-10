//! `table diff` compares two tables and exits non-zero when they differ.

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
fn diff_reports_no_differences_for_identical_tables() {
    let dir = testdir!();
    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);
    std::fs::copy(dir.join("table.vpx"), dir.join("copy.vpx")).expect("copy vpx");

    let out = vpxtool(&dir, &["table", "diff", "table.vpx", "copy.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "diff failed: {:?}", out);
    assert!(
        stdout.contains("no differences"),
        "unexpected output: {stdout}"
    );
}

#[test]
fn diff_reports_script_change_and_exits_non_zero() {
    let dir = testdir!();
    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);
    std::fs::copy(dir.join("table.vpx"), dir.join("modified.vpx")).expect("copy vpx");

    // append a line to the script of the copy
    let out = vpxtool(&dir, &["script", "extract", "modified.vpx"]);
    assert!(out.status.success(), "script extract failed: {:?}", out);
    let vbs_path = dir.join("modified.vbs");
    let mut script = std::fs::read_to_string(&vbs_path).expect("read vbs");
    script.push_str("' added by the diff test\r\n");
    std::fs::write(&vbs_path, script).expect("write vbs");
    let out = vpxtool(&dir, &["script", "import", "modified.vpx"]);
    assert!(out.status.success(), "script import failed: {:?}", out);

    let out = vpxtool(&dir, &["table", "diff", "table.vpx", "modified.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "expected exit 1: {:?}", out);
    assert!(
        stdout.contains("script changed (+1 -0 lines)"),
        "unexpected output: {stdout}"
    );
    assert!(
        !stdout.contains("added by the diff test"),
        "script text should only show with --script: {stdout}"
    );

    let out = vpxtool(
        &dir,
        &["table", "diff", "--script", "table.vpx", "modified.vpx"],
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "expected exit 1: {:?}", out);
    assert!(
        stdout.contains("+' added by the diff test"),
        "unexpected output: {stdout}"
    );
}
