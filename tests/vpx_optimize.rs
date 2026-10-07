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

/// A table with a stereo playfield sound, which `--mono` downmixes
fn table_with_stereo_sound(path: &Path) {
    vpin::vpx::new_minimal_vpx(path).expect("new table");
    let mut vpx = vpin::vpx::read(path).expect("read table");
    let mut data = Vec::new();
    for frame in 0..4096i32 {
        for channel in 0..2 {
            data.extend((((frame + channel * 7) % 97 - 48) as i16).to_le_bytes());
        }
    }
    vpx.sounds.push(vpin::vpx::sound::SoundData {
        name: "hit".to_string(),
        path: "hit.wav".to_string(),
        data,
        wave_form: vpin::vpx::sound::WaveForm {
            format_tag: 1,
            channels: 2,
            samples_per_sec: 44100,
            avg_bytes_per_sec: 44100 * 4,
            block_align: 4,
            bits_per_sample: 16,
            cb_size: 0,
        },
        internal_name: String::new(),
        fade: 0,
        volume: 0,
        balance: 0,
        output_target: vpin::vpx::sound::OutputTarget::Table,
    });
    std::fs::remove_file(path).expect("remove table");
    vpin::vpx::write(path, &vpx).expect("write table");
}

/// Rewrites the table in place, which on Windows needs the temporary file
/// flushed through a handle with write access
#[test]
fn a_table_with_something_to_fix_is_rewritten_in_place() {
    let dir = testdir!();
    table_with_stereo_sound(&dir.join("table.vpx"));

    let out = vpxtool(&dir, &["optimize", "--mono", "table.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "optimize failed: {:?}", out);
    assert!(
        stdout.contains("stereo -> mono"),
        "unexpected output: {stdout}"
    );
    assert!(!dir.join("table.vpx.tmp").exists());
    let vpx = vpin::vpx::read(&dir.join("table.vpx")).expect("read optimized table");
    assert_eq!(vpx.sounds[0].wave_form.channels, 1);
}

/// A table with a 600x300 png, over a 512 pixel limit
fn table_with_large_image(path: &Path) {
    vpin::vpx::new_minimal_vpx(path).expect("new table");
    let mut vpx = vpin::vpx::read(path).expect("read table");
    let pixels = image::RgbaImage::from_fn(600, 300, |x, y| {
        image::Rgba([(x % 256) as u8, (y % 256) as u8, ((x * y) % 256) as u8, 255])
    });
    let mut data = Vec::new();
    pixels
        .write_to(
            &mut std::io::Cursor::new(&mut data),
            image::ImageFormat::Png,
        )
        .expect("png encodes");
    vpx.add_or_replace_image(vpin::vpx::image::ImageData {
        name: "playfield".to_string(),
        path: "playfield.png".to_string(),
        width: 600,
        height: 300,
        jpeg: Some(vpin::vpx::pinbinary::PinBinary {
            path: "playfield.png".to_string(),
            name: "playfield".to_string(),
            internal_name: None,
            data,
        }),
        ..Default::default()
    });
    std::fs::remove_file(path).expect("remove table");
    vpin::vpx::write(path, &vpx).expect("write table");
}

#[test]
fn images_over_the_max_image_size_are_scaled_down() {
    let dir = testdir!();
    table_with_large_image(&dir.join("table.vpx"));

    let out = vpxtool(&dir, &["optimize", "--max-image-size", "512", "table.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "optimize failed: {:?}", out);
    assert!(
        stdout.contains("image \"playfield\" 600x300 -> 512x256, "),
        "unexpected output: {stdout}"
    );
    assert!(
        stdout.contains("1 images: 720.0 KB -> 524.3 KB of texture memory"),
        "unexpected output: {stdout}"
    );
    let vpx = vpin::vpx::read(&dir.join("table.vpx")).expect("read optimized table");
    assert_eq!((vpx.images[0].width, vpx.images[0].height), (512, 256));
    // the shrunk png went on to become a webp
    assert_eq!(vpx.images[0].ext(), "webp");

    // the mobile apps offer nothing below 256
    let out = vpxtool(&dir, &["optimize", "--max-image-size", "128", "table.vpx"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("256..=16384"), "unexpected error: {stderr}");
}

#[test]
fn the_bare_flag_means_the_mobile_default() {
    let dir = testdir!();
    table_with_large_image(&dir.join("table.vpx"));

    // 600x300 fits in 1536, so the bare flag has nothing to shrink
    let out = vpxtool(&dir, &["optimize", "table.vpx", "--max-image-size"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "optimize failed: {:?}", out);
    assert!(
        !stdout.contains("600x300 ->"),
        "unexpected output: {stdout}"
    );

    // a value needs the = form when the path follows
    let out = vpxtool(&dir, &["optimize", "--max-image-size=512", "table.vpx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "optimize failed: {:?}", out);
    assert!(
        stdout.contains("600x300 -> 512x256"),
        "unexpected output: {stdout}"
    );
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
