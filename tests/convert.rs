//! `convert` turns a vpx file into a table pack and back.

use std::fs;
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

fn table_name(path: &Path) -> Option<String> {
    vpin::vpx::read(path).expect("read vpx").info.table_name
}

#[test]
fn convert_between_vpx_and_packs() {
    let dir = testdir!();
    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);
    let name = table_name(&dir.join("table.vpx"));

    // vpx to a .vpz zip next to it
    let out = vpxtool(&dir, &["convert", "table.vpx"]);
    assert!(out.status.success(), "convert to vpz failed: {:?}", out);
    assert_eq!(&fs::read(dir.join("table.vpz")).unwrap()[..2], b"PK");

    // .vpz to a pack folder, and the folder back to a vpx next to it
    let out = vpxtool(&dir, &["convert", "table.vpz", "pack 1.5"]);
    assert!(out.status.success(), "convert to folder failed: {:?}", out);
    assert!(dir.join("pack 1.5").join("manifest.json").is_file());
    let out = vpxtool(&dir, &["convert", "pack 1.5"]);
    assert!(out.status.success(), "convert to vpx failed: {:?}", out);
    assert_eq!(table_name(&dir.join("pack 1.5.vpx")), name);

    // an existing file is only replaced with --force
    let out = vpxtool(&dir, &["convert", "table.vpz", "table.vpx"]);
    assert!(
        !out.status.success(),
        "overwrote without --force: {:?}",
        out
    );
    let out = vpxtool(&dir, &["convert", "--force", "table.vpz", "table.vpx"]);
    assert!(out.status.success(), "convert --force failed: {:?}", out);
    assert_eq!(table_name(&dir.join("table.vpx")), name);

    // a pack folder that is not empty is never written into
    let out = vpxtool(&dir, &["convert", "--force", "table.vpx", "pack 1.5"]);
    assert!(!out.status.success(), "wrote into a pack folder: {:?}", out);

    // two vpx files, nothing to convert
    let out = vpxtool(&dir, &["convert", "table.vpx", "other.vpx"]);
    assert!(!out.status.success(), "converted vpx to vpx: {:?}", out);
}
