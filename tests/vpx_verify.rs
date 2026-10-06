//! `verify --json` reports the stored and the computed MAC of each file.

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
fn verify_json_reports_the_mac_of_each_file() {
    let dir = testdir!();
    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);

    let out = vpxtool(&dir, &["verify", "--json", "table.vpx", "missing.vpx"]);
    assert!(out.status.success(), "verify failed: {:?}", out);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json output");

    let table = &json[0];
    assert_eq!(table["path"], "table.vpx");
    assert_eq!(table["valid"], true);
    let mac = table["mac"].as_str().expect("a mac");
    assert_eq!(mac.len(), 32);
    assert_eq!(table["computed_mac"], mac);

    let missing = &json[1];
    assert_eq!(missing["path"], "missing.vpx");
    assert_eq!(missing["mac"], serde_json::Value::Null);
    assert_eq!(missing["valid"], false);
    assert!(missing["error"].is_string());
}
