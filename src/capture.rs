use crate::config::ResolvedConfig;
use crate::describe_exit;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{ImageFormat, ImageReader};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::{fs, io, thread};

/// JPEG quality (0-100) used when encoding to jpg.
const JPEG_QUALITY: u8 = 80;

/// Output image format for a captured playfield screenshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaptureFormat {
    /// Lossy, ~15x smaller than png and fast to encode. The default.
    #[default]
    Jpg,
    /// Lossless, large (~10MB at 4K) and slow to encode.
    Png,
    /// Lossless WebP via the `image` crate; only marginally smaller than png.
    Webp,
    /// vpinball's native capture format, kept as-is (no decode/encode). Lossless
    /// and instant to save, but ~10MB and read by almost nothing.
    Qoi,
}

impl CaptureFormat {
    /// File extension (without leading dot) used for the generated image.
    pub fn extension(&self) -> &'static str {
        match self {
            CaptureFormat::Png => "png",
            CaptureFormat::Jpg => "jpg",
            CaptureFormat::Webp => "webp",
            CaptureFormat::Qoi => "qoi",
        }
    }
}

impl Display for CaptureFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

impl FromStr for CaptureFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "png" => Ok(CaptureFormat::Png),
            "jpg" | "jpeg" => Ok(CaptureFormat::Jpg),
            "webp" => Ok(CaptureFormat::Webp),
            "qoi" => Ok(CaptureFormat::Qoi),
            other => Err(format!(
                "Unknown image format '{other}', expected jpg, png, webp or qoi"
            )),
        }
    }
}

/// Options controlling a screenshot capture run.
#[derive(Debug, Clone)]
pub struct CaptureOptions {
    pub format: CaptureFormat,
    /// Regenerate the image even if it already exists.
    pub force: bool,
    /// Number of frames vpinball captures. The first one is used as the
    /// screenshot. More than one is rarely useful for a still image.
    pub frames: u32,
    /// Capture framerate passed to vpinball (irrelevant for a single frame).
    pub fps: u32,
    /// When set, downscale the image so its width does not exceed this many
    /// pixels (keeping aspect ratio). `None` keeps the native window resolution.
    pub max_width: Option<u32>,
    /// Kill vpinball and fail the capture if it runs longer than this, so a
    /// hanging table does not stall a batch. `None` waits indefinitely.
    pub timeout: Option<Duration>,
}

impl Default for CaptureOptions {
    fn default() -> Self {
        CaptureOptions {
            format: CaptureFormat::default(),
            force: false,
            frames: 1,
            fps: 1,
            max_width: None,
            timeout: Some(Duration::from_secs(60)),
        }
    }
}

/// Result of a single table capture.
#[derive(Debug)]
pub enum CaptureOutcome {
    /// A new image was written to the given path.
    Captured(PathBuf),
    /// vpinball hung and was killed after the timeout, but it had already
    /// written the frame, so the image was salvaged. The table is slow/flaky.
    CapturedAfterHang(PathBuf),
    /// An image already existed and `force` was not set.
    Skipped(PathBuf),
}

/// Subfolder (next to the `.vpx`) the playfield image is written into.
const MEDIA_DIR: &str = "media";

/// Basename used for the playfield image.
const PLAYFIELD_MEDIA_BASENAME: &str = "table";

/// Path where the playfield screenshot for the given table is written:
/// `<table dir>/media/table.<ext>`.
pub fn capture_image_path(vpx_path: &Path, format: CaptureFormat) -> PathBuf {
    let dir = vpx_path.parent().unwrap_or_else(|| Path::new(""));
    dir.join(MEDIA_DIR)
        .join(format!("{PLAYFIELD_MEDIA_BASENAME}.{}", format.extension()))
}

/// Capture a single playfield screenshot for `vpx_path` using vpinball's
/// `-CaptureAttract` mode and write it to `<table dir>/media/table.<ext>`.
///
/// vpinball renders the playfield into a `Capture/` folder next to the table as
/// lossless `.qoi` frames, fast-forwarding ~30s of attract play before grabbing
/// the frame. We convert the first frame to the requested format and clean up.
pub fn capture_table(
    config: &ResolvedConfig,
    vpx_path: &Path,
    options: &CaptureOptions,
) -> io::Result<CaptureOutcome> {
    let output_path = capture_image_path(vpx_path, options.format);
    if output_path.exists() && !options.force {
        return Ok(CaptureOutcome::Skipped(output_path));
    }

    let capture_dir = vpx_path
        .parent()
        .map(|p| p.join("Capture"))
        .unwrap_or_else(|| PathBuf::from("Capture"));

    // Start from a clean slate so we never pick up frames from a previous run.
    remove_capture_dir(&capture_dir);

    crate::println!("Launching capture for {}", vpx_path.display())?;
    let started = Instant::now();
    let run = run_capture(config, vpx_path, options)?;
    let elapsed = started.elapsed();
    let timed_out = run.status.is_none();

    // A clean non-zero exit (e.g. a crash) means no usable frame; fail loudly,
    // reporting the signal when vpinball was killed rather than exiting normally.
    if let Some(status) = run.status
        && !status.success()
    {
        remove_capture_dir(&capture_dir);
        return Err(io::Error::other(format!(
            "vpinball {} while capturing {}{}",
            describe_exit(status),
            vpx_path.display(),
            log_tail(&run.log)
        )));
    }

    // On timeout we still try to use the frame: many tables only hang on
    // shutdown, after the playfield has already been written.
    let frame = match first_playfield_frame(&capture_dir) {
        Ok(frame) => frame,
        Err(e) => {
            remove_capture_dir(&capture_dir);
            if timed_out {
                let secs = options.timeout.map(|t| t.as_secs()).unwrap_or(0);
                return Err(io::Error::other(format!(
                    "vpinball did not finish within {secs}s and was terminated before \
                     capturing a frame for {}{}",
                    vpx_path.display(),
                    log_tail(&run.log)
                )));
            }
            return Err(e);
        }
    };

    // Ensure the media/ folder exists before writing the image into it.
    if let Some(parent) = output_path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        remove_capture_dir(&capture_dir);
        return Err(e);
    }

    crate::println!(
        "Capture finished in {:.1}s, converting image",
        elapsed.as_secs_f32()
    )?;
    let convert_result = convert_frame(&frame, &output_path, options.format, options.max_width);

    // Always clean up the intermediate qoi frames, even on conversion errors.
    remove_capture_dir(&capture_dir);
    convert_result?;

    let table_file = vpx_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("table");
    crate::println!(
        "Capture complete for {} at {}",
        table_file,
        output_path.display()
    )?;

    if timed_out {
        Ok(CaptureOutcome::CapturedAfterHang(output_path))
    } else {
        Ok(CaptureOutcome::Captured(output_path))
    }
}

/// Result of a vpinball capture run.
struct CaptureRun {
    /// `Some(status)` if vpinball exited on its own; `None` if it hit the
    /// timeout and we killed it.
    status: Option<ExitStatus>,
    /// vpinball's (verbose) stdout/stderr, surfaced only on failure.
    log: String,
}

/// Spawn vpinball in capture mode and wait for it to exit (or be killed after
/// the timeout). vpinball's verbose stdout/stderr is redirected to a temp file
/// (avoiding pipe-buffer deadlocks if it stalls mid-output) and read back so the
/// caller can surface a tail on failure.
fn run_capture(
    config: &ResolvedConfig,
    vpx_path: &Path,
    options: &CaptureOptions,
) -> io::Result<CaptureRun> {
    let log_path = std::env::temp_dir().join(format!("vpxtool-capture-{}.log", std::process::id()));
    let log_file = File::create(&log_path)?;
    let log_file_err = log_file.try_clone()?;

    let mut cmd = Command::new(&config.vpx_executable);
    // Reuse the environment from the first launch template (e.g. SDL_VIDEO_DRIVER
    // on Wayland) so capture behaves like a normal launch.
    if let Some(env) = config.launch_templates.first().and_then(|t| t.env.as_ref()) {
        for (key, value) in env.iter() {
            cmd.env(key, value);
        }
    }
    cmd.arg("-Ini");
    cmd.arg(&config.vpx_config);
    // -CaptureAttract <frames> <fps> <table> noloop
    cmd.arg("-CaptureAttract");
    cmd.arg(options.frames.to_string());
    cmd.arg(options.fps.to_string());
    cmd.arg(vpx_path);
    cmd.arg("noloop");
    cmd.stdout(Stdio::from(log_file));
    cmd.stderr(Stdio::from(log_file_err));

    let mut child = cmd.spawn()?;
    let status = wait_with_timeout(&mut child, options.timeout)?;

    let log = fs::read_to_string(&log_path).unwrap_or_default();
    let _ = fs::remove_file(&log_path);
    Ok(CaptureRun { status, log })
}

/// Wait for the child to exit, polling so we can kill it once `timeout` elapses.
/// Returns `Ok(None)` if it was killed for exceeding the timeout.
fn wait_with_timeout(
    child: &mut Child,
    timeout: Option<Duration>,
) -> io::Result<Option<ExitStatus>> {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if let Some(timeout) = timeout
            && start.elapsed() >= timeout
        {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(200));
    }
}

/// Keep only the last few lines of vpinball output for an error message,
/// prefixed with a blank line, or an empty string when there is nothing.
fn log_tail(output: &str) -> String {
    let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return String::new();
    }
    let start = lines.len().saturating_sub(10);
    format!("\n{}", lines[start..].join("\n"))
}

/// Find the lowest-indexed `Playfield_*.qoi` frame in the capture folder.
fn first_playfield_frame(capture_dir: &Path) -> io::Result<PathBuf> {
    if !capture_dir.is_dir() {
        return Err(io::Error::other(format!(
            "vpinball did not produce a capture folder at {}",
            capture_dir.display()
        )));
    }
    let mut frames: Vec<PathBuf> = fs::read_dir(capture_dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| is_playfield_frame(p))
        .collect();
    frames.sort();
    frames.into_iter().next().ok_or_else(|| {
        io::Error::other(format!(
            "no playfield frame found in {}",
            capture_dir.display()
        ))
    })
}

/// Whether the path is a `Playfield_*.qoi` frame written by vpinball. The
/// `noloop` capture keeps a `_tmp` suffix, so match on the prefix and extension.
fn is_playfield_frame(path: &Path) -> bool {
    let is_qoi = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("qoi"));
    let is_playfield = path
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("Playfield_"));
    is_qoi && is_playfield
}

/// Write the captured qoi frame to `output_path` in the requested format,
/// optionally downscaling first.
fn convert_frame(
    frame: &Path,
    output_path: &Path,
    format: CaptureFormat,
    max_width: Option<u32>,
) -> io::Result<()> {
    // qoi: keep vpinball's native frame as-is, no decode/encode (max_width does
    // not apply since we never decode it).
    if format == CaptureFormat::Qoi {
        fs::copy(frame, output_path)?;
        return Ok(());
    }

    let mut image = ImageReader::open(frame)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| io::Error::other(format!("Unable to decode {}: {e}", frame.display())))?;

    // Downscale (never upscale) so the width fits max_width, keeping aspect ratio.
    if let Some(max_width) = max_width
        && image.width() > max_width
    {
        let height =
            ((image.height() as u64 * max_width as u64) / image.width() as u64).max(1) as u32;
        image = image.resize(max_width, height, FilterType::Lanczos3);
    }

    let write_err = |e| io::Error::other(format!("Unable to write {}: {e}", output_path.display()));
    match format {
        // png and webp keep the alpha channel as captured.
        CaptureFormat::Png => image
            .save_with_format(output_path, ImageFormat::Png)
            .map_err(write_err),
        CaptureFormat::Webp => image
            .save_with_format(output_path, ImageFormat::WebP)
            .map_err(write_err),
        // jpeg has no alpha channel, so drop it first; encode at JPEG_QUALITY.
        CaptureFormat::Jpg => {
            let rgb = image.to_rgb8();
            let mut writer = BufWriter::new(File::create(output_path)?);
            JpegEncoder::new_with_quality(&mut writer, JPEG_QUALITY)
                .encode_image(&rgb)
                .map_err(write_err)
        }
        CaptureFormat::Qoi => unreachable!("qoi handled above"),
    }
}

fn remove_capture_dir(capture_dir: &Path) {
    if capture_dir.exists() {
        // Best effort: leftover frames are harmless and a failure here should
        // not mask the capture result.
        let _ = fs::remove_dir_all(capture_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_format_from_str() {
        assert_eq!(CaptureFormat::from_str("png"), Ok(CaptureFormat::Png));
        assert_eq!(CaptureFormat::from_str("PNG"), Ok(CaptureFormat::Png));
        assert_eq!(CaptureFormat::from_str("jpg"), Ok(CaptureFormat::Jpg));
        assert_eq!(CaptureFormat::from_str("jpeg"), Ok(CaptureFormat::Jpg));
        assert_eq!(CaptureFormat::from_str("webp"), Ok(CaptureFormat::Webp));
        assert_eq!(CaptureFormat::from_str("WebP"), Ok(CaptureFormat::Webp));
        assert_eq!(CaptureFormat::from_str("qoi"), Ok(CaptureFormat::Qoi));
        assert!(CaptureFormat::from_str("gif").is_err());
    }

    #[test]
    fn test_capture_image_path() {
        // table.<ext> next to the vpx, named "table" regardless of the vpx
        // filename (vpinfe-compatible via its table-dir fallback).
        let vpx = Path::new("/tables/Foo (Bar 1999)/Foo (Bar 1999).vpx");
        assert_eq!(
            capture_image_path(vpx, CaptureFormat::Png),
            PathBuf::from("/tables/Foo (Bar 1999)/media/table.png")
        );
        assert_eq!(
            capture_image_path(vpx, CaptureFormat::Jpg),
            PathBuf::from("/tables/Foo (Bar 1999)/media/table.jpg")
        );
    }

    #[test]
    fn test_capture_image_path_no_parent() {
        let vpx = Path::new("Foo.vpx");
        assert_eq!(
            capture_image_path(vpx, CaptureFormat::Png),
            PathBuf::from("media/table.png")
        );
    }

    #[test]
    fn test_is_playfield_frame() {
        assert!(is_playfield_frame(Path::new("/c/Playfield_00001_tmp.qoi")));
        assert!(is_playfield_frame(Path::new("/c/Playfield_00001.qoi")));
        assert!(!is_playfield_frame(Path::new("/c/Backglass_00001.qoi")));
        assert!(!is_playfield_frame(Path::new("/c/Playfield_00001.png")));
    }
}
