//! Aborting at an overwrite prompt must exit non-zero so scripted callers
//! can detect that nothing was written.
//! See https://github.com/francisdb/vpxtool/issues/827

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use testdir::testdir;

/// Runs the vpxtool binary in `dir`. `stdin` of `None` means stdin is closed
/// (the no-TTY scripted case); `Some` pipes the given answer to the prompt.
fn vpxtool(dir: &Path, args: &[&str], stdin: Option<&str>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vpxtool"))
        .args(args)
        .current_dir(dir)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn vpxtool");
    if let Some(input) = stdin {
        child
            .stdin
            .as_mut()
            .expect("stdin not piped")
            .write_all(input.as_bytes())
            .expect("failed to write to stdin");
    }
    child
        .wait_with_output()
        .expect("failed to wait for vpxtool")
}

fn assert_aborted(output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("Aborted"),
        "expected \"Aborted\" on stdout, got stdout: {stdout:?}, stderr: {stderr:?}"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit code 1 on abort, got {:?}, stdout: {stdout:?}, stderr: {stderr:?}",
        output.status.code()
    );
}

#[test]
fn assemble_abort_answer_no_exits_nonzero() {
    let dir = testdir!();
    std::fs::create_dir(dir.join("expanded")).unwrap();
    std::fs::write(dir.join("out.vpx"), b"original").unwrap();

    let output = vpxtool(&dir, &["assemble", "expanded", "out.vpx"], Some("n\n"));

    assert_aborted(&output);
    assert_eq!(std::fs::read(dir.join("out.vpx")).unwrap(), b"original");
}

#[test]
fn assemble_abort_closed_stdin_exits_nonzero() {
    let dir = testdir!();
    std::fs::create_dir(dir.join("expanded")).unwrap();
    std::fs::write(dir.join("out.vpx"), b"original").unwrap();

    let output = vpxtool(&dir, &["assemble", "expanded", "out.vpx"], None);

    assert_aborted(&output);
    assert_eq!(std::fs::read(dir.join("out.vpx")).unwrap(), b"original");
}

#[test]
fn extract_abort_answer_no_exits_nonzero() {
    let dir = testdir!();
    std::fs::write(dir.join("table.vpx"), b"not read before the prompt").unwrap();
    // default output dir is the vpx path without extension
    std::fs::create_dir(dir.join("table")).unwrap();
    std::fs::write(dir.join("table").join("keep.txt"), b"keep").unwrap();

    let output = vpxtool(&dir, &["extract", "table.vpx"], Some("n\n"));

    assert_aborted(&output);
    assert!(dir.join("table").join("keep.txt").exists());
}

#[test]
fn info_extract_abort_answer_no_exits_nonzero() {
    let dir = testdir!();
    std::fs::write(dir.join("table.vpx"), b"not read before the prompt").unwrap();
    std::fs::write(dir.join("table.info.json"), b"original").unwrap();

    let output = vpxtool(&dir, &["info", "extract", "table.vpx"], Some("n\n"));

    assert_aborted(&output);
    assert_eq!(
        std::fs::read(dir.join("table.info.json")).unwrap(),
        b"original"
    );
}

#[test]
fn export_vpxz_abort_answer_no_exits_nonzero() {
    let dir = testdir!();
    std::fs::write(dir.join("table.vpx"), b"not read before the prompt").unwrap();
    std::fs::write(dir.join("out.vpxz"), b"original").unwrap();

    let output = vpxtool(
        &dir,
        &["export", "vpxz", "table.vpx", "-o", "out.vpxz"],
        Some("n\n"),
    );

    assert_aborted(&output);
    assert_eq!(std::fs::read(dir.join("out.vpxz")).unwrap(), b"original");
}

/// Positive control: confirming the prompt still succeeds with exit code 0.
#[test]
fn info_extract_answer_yes_exits_zero() {
    let dir = testdir!();
    let output = vpxtool(&dir, &["new", "table.vpx"], None);
    assert!(
        output.status.success(),
        "failed to create test vpx: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(dir.join("table.info.json"), b"stale").unwrap();

    let output = vpxtool(&dir, &["info", "extract", "table.vpx"], Some("y\n"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout:?}, stderr: {stderr:?}"
    );
    assert_ne!(
        std::fs::read(dir.join("table.info.json")).unwrap(),
        b"stale"
    );
}
