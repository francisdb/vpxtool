//! `extract --only` and `--no-media` write part of a table.

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

/// All files below `dir`, relative, sorted, with `/` separators
fn files(dir: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("read dir").flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let relative = path.strip_prefix(root).expect("below root");
                out.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

fn fixture(dir: &Path) -> String {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join("completely_blank_table_10_7_4.vpx");
    std::fs::copy(source, dir.join("table.vpx")).expect("copy fixture");
    "table.vpx".to_string()
}

#[test]
fn only_keeps_the_named_file() {
    let dir = testdir!();
    let table = fixture(&dir);
    let out = vpxtool(
        &dir,
        &["extract", "--only", "gamedata.json", "-o", "out", &table],
    );
    assert!(out.status.success(), "extract failed: {:?}", out);
    assert_eq!(files(&dir.join("out")), vec!["gamedata.json"]);
}

#[test]
fn only_glob_does_not_cross_directories() {
    let dir = testdir!();
    let table = fixture(&dir);
    let out = vpxtool(
        &dir,
        &["extract", "--only", "gameitems/*.json", "-o", "out", &table],
    );
    assert!(out.status.success(), "extract failed: {:?}", out);
    let written = files(&dir.join("out"));
    assert!(!written.is_empty());
    assert!(
        written
            .iter()
            .all(|f| f.starts_with("gameitems/") && f.ends_with(".json"))
    );
    assert!(!written.contains(&"gameitems.json".to_string()));
}

#[test]
fn no_media_skips_the_image_files_but_keeps_the_index() {
    let dir = testdir!();
    let table = fixture(&dir);
    let out = vpxtool(&dir, &["extract", "--no-media", "-o", "out", &table]);
    assert!(out.status.success(), "extract failed: {:?}", out);
    let written = files(&dir.join("out"));
    assert!(written.contains(&"images.json".to_string()));
    assert!(written.contains(&"gamedata.json".to_string()));
    assert!(
        written
            .iter()
            .all(|f| !f.starts_with("images/") && !f.starts_with("sounds/"))
    );
    assert!(!dir.join("out").join("images").exists());
}

#[test]
fn bad_glob_fails_before_writing() {
    let dir = testdir!();
    let table = fixture(&dir);
    let out = vpxtool(
        &dir,
        &["extract", "--only", "gameitems/[", "-o", "out", &table],
    );
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        (stderr.to_string() + &stdout).contains("invalid --only glob"),
        "unexpected output: {stdout} {stderr}"
    );
    assert!(!dir.join("out").exists());
}
