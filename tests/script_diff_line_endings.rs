//! Verify that `script diff` normalizes line endings so that differences
//! in CR/LF style alone don't produce spurious output.

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

/// When the sidecar .vbs differs from the VPX-embedded script only in line
/// endings (a mix of CRLF, bare CR, and bare LF — matching the real-world
/// scenario from tables like Mustang LE), `script diff` should produce no
/// output.
#[test]
fn script_diff_ignores_mixed_line_ending_differences() {
    let dir = testdir!();

    // Create a minimal VPX.
    let vpx_path = dir.join("table.vpx");
    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);

    // Extract the embedded script as a sidecar .vbs.
    let out = vpxtool(&dir, &["script", "extract", "table.vpx"]);
    assert!(
        out.status.success(),
        "vpxtool script extract failed: {:?}",
        out
    );

    // Read the sidecar and rebuild it with a mix of line endings:
    // some lines use bare CR, some use bare LF, the rest stay as CRLF.
    // This mirrors real tables where editors or copy-paste introduced
    // inconsistent terminators into the VBS script.
    let vbs_path = vpx_path.with_extension("vbs");
    let original = std::fs::read_to_string(&vbs_path).expect("read vbs");
    let lines: Vec<&str> = original.lines().collect();
    let mut mixed = String::new();
    for (i, line) in lines.iter().enumerate() {
        mixed.push_str(line);
        match i % 3 {
            0 => mixed.push_str("\r\n"), // CRLF (standard)
            1 => mixed.push('\r'),       // bare CR (the problematic case)
            _ => mixed.push('\n'),       // bare LF
        }
    }
    std::fs::write(&vbs_path, &mixed).expect("write vbs");

    // script diff should see no semantic difference.
    let out = vpxtool(&dir, &["script", "diff", "table.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "script diff exited with failure: {:?}\nstderr: {stderr}",
        out.status,
    );
    assert!(
        stdout.trim().is_empty(),
        "expected no diff output when only line endings differ, got:\n{stdout}"
    );
}

/// When the sidecar .vbs has a real content change on top of line-ending
/// differences, `script diff` should still report that change.
#[test]
fn script_diff_reports_real_changes_despite_mixed_endings() {
    let dir = testdir!();

    let vpx_path = dir.join("table.vpx");
    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);

    let out = vpxtool(&dir, &["script", "extract", "table.vpx"]);
    assert!(
        out.status.success(),
        "vpxtool script extract failed: {:?}",
        out
    );

    // Modify the sidecar: convert to LF and append a new line of code.
    let vbs_path = vpx_path.with_extension("vbs");
    let original = std::fs::read_to_string(&vbs_path).expect("read vbs");
    let mut modified = original.replace("\r\n", "\n").replace('\r', "\n");
    modified.push_str("' added comment\n");
    std::fs::write(&vbs_path, &modified).expect("write vbs");

    let out = vpxtool(&dir, &["script", "diff", "table.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("added comment"),
        "expected diff to contain the real change, got:\n{stdout}"
    );
}

/// A latin1 sidecar (as produced by VPinballX -ExtractVBS on legacy tables)
/// must not make `script diff` fail with a utf8 error; the real change must
/// still be reported.
#[test]
fn script_diff_handles_latin1_sidecar() {
    let dir = testdir!();

    let out = vpxtool(&dir, &["new", "table.vpx"]);
    assert!(out.status.success(), "vpxtool new failed: {:?}", out);

    let out = vpxtool(&dir, &["script", "extract", "table.vpx"]);
    assert!(
        out.status.success(),
        "vpxtool script extract failed: {:?}",
        out
    );

    // Append a comment containing a latin1 e-acute (0xE9), making the file
    // invalid utf8.
    let vbs_path = dir.join("table.vbs");
    let mut bytes = std::fs::read(&vbs_path).expect("read vbs");
    bytes.extend_from_slice(b"' caf\xE9 comment\r\n");
    std::fs::write(&vbs_path, &bytes).expect("write vbs");

    let out = vpxtool(&dir, &["script", "diff", "table.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "script diff exited with failure: {:?}\nstderr: {stderr}",
        out.status,
    );
    assert!(
        stdout.contains("comment"),
        "expected diff to contain the added latin1 line, got:\n{stdout}"
    );
}
