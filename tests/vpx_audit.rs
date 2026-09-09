//! `audit` reports consistency problems and only fails on error findings.

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
fn audit_reports_findings_for_new_table() {
    let dir = testdir!();
    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);

    let out = vpxtool(&dir, &["audit", "table.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    // a fresh table has warnings (no table name) but nothing of error severity
    assert!(out.status.success(), "audit failed: {:?}", out);
    assert!(stdout.contains("table.vpx"), "unexpected output: {stdout}");
    assert!(
        stdout.contains("table info has no table name"),
        "unexpected output: {stdout}"
    );
    assert!(stdout.contains("0 errors"), "unexpected output: {stdout}");
}
