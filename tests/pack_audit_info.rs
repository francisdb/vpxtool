//! `audit` and `info show` read a table pack, a `.vpz` file or a folder.

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

/// A new table saved as `table.vpz` and as the pack folder `table`
fn packs(dir: &Path) {
    let out = vpxtool(dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);
    let vpx = vpin::vpx::read(&dir.join("table.vpx")).expect("read table");
    let pack = vpin::vpz::from_vpx(&vpx, "Tue Oct  6 10:00:00 2026").expect("convert");
    vpin::vpz::write(&pack, dir.join("table.vpz")).expect("write zip");
    vpin::vpz::write(&pack, dir.join("table")).expect("write folder");
}

#[test]
fn audit_reads_a_pack() {
    let dir = testdir!();
    packs(&dir);
    for pack in ["table.vpz", "table"] {
        let out = vpxtool(&dir, &["audit", pack]);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "audit {pack} failed: {:?}", out);
        assert!(
            stdout.contains("table info has no table name"),
            "unexpected output for {pack}: {stdout}"
        );
        assert!(stdout.contains("0 errors"), "unexpected output: {stdout}");
    }
}

#[test]
fn info_show_reads_a_pack() {
    let dir = testdir!();
    packs(&dir);
    for pack in ["table.vpz", "table"] {
        let out = vpxtool(&dir, &["info", "show", pack]);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "info show {pack} failed: {:?}", out);
        assert!(stdout.contains("Pack Version:"), "unexpected output: {stdout}");
        assert!(stdout.contains("Table Name:"), "unexpected output: {stdout}");
    }
}
