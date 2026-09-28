//! The `optimize` command: the fixes from [`vpin::vpx::fix`] that keep a
//! table rendering and playing the same, applied in memory and written
//! back in one go, which also compacts the file.

use crate::atomicwrite::atomic_write_path;
use colored::Colorize;
use std::io;
use std::path::Path;
use vpin::vpx;
use vpin::vpx::VPX;
use vpin::vpx::fix::{self, ConvertedImage, ImageConversion, RemovedFont, SkippedImage};

/// What [`apply`] did to a table
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// The embedded fonts nothing used
    pub removed_fonts: Vec<RemovedFont>,
    /// The bitmaps re-encoded as webp
    pub bitmaps: Vec<ConvertedImage>,
    /// The pngs re-encoded as webp
    pub pngs: Vec<ConvertedImage>,
    /// The tgas re-encoded as webp
    pub tgas: Vec<ConvertedImage>,
    /// The candidate images left alone, with the reason
    pub skipped: Vec<SkippedImage>,
}

impl Report {
    /// Whether the table was left as it was
    pub fn is_empty(&self) -> bool {
        self.removed_fonts.is_empty()
            && self.bitmaps.is_empty()
            && self.pngs.is_empty()
            && self.tgas.is_empty()
    }

    /// Bytes of stored data the changes save
    pub fn bytes_saved(&self) -> usize {
        let images = self
            .bitmaps
            .iter()
            .chain(&self.pngs)
            .chain(&self.tgas)
            .map(|image| image.bytes_before().saturating_sub(image.bytes_after()))
            .sum::<usize>();
        images
            + self
                .removed_fonts
                .iter()
                .map(RemovedFont::bytes)
                .sum::<usize>()
    }

    fn take(&mut self, conversion: ImageConversion) -> Vec<ConvertedImage> {
        self.skipped.extend(conversion.skipped().iter().cloned());
        conversion.converted().to_vec()
    }
}

/// Applies every lossless fix to the table in memory: drops the fonts
/// nothing uses and re-encodes bitmap, png and tga images as lossless webp
/// where that is smaller. Images the script hands to FlexDMD are left
/// alone.
pub fn apply(vpx: &mut VPX) -> Report {
    let mut report = Report {
        removed_fonts: fix::drop_unused_fonts(vpx),
        ..Report::default()
    };
    report.bitmaps = report.take(fix::bitmaps_to_webp(vpx));
    report.pngs = report.take(fix::pngs_to_webp(vpx));
    report.tgas = report.take(fix::tgas_to_webp(vpx));
    report
}

/// The outcome of [`optimize_file`]
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    /// What changed
    pub report: Report,
    /// Size of the file before
    pub file_bytes_before: u64,
    /// Size of the file after the rewrite, `None` for a dry run or when
    /// nothing changed
    pub file_bytes_after: Option<u64>,
}

/// Reads the table, applies the fixes and, unless `dry_run` or nothing
/// changed, writes it back in place through a temporary sibling file, so
/// an interrupted run leaves the original intact. The rewrite compacts
/// the file.
pub fn optimize_file(path: &Path, dry_run: bool) -> io::Result<Outcome> {
    let file_bytes_before = std::fs::metadata(path)?.len();
    let mut vpx = vpx::read(path)?;
    let report = apply(&mut vpx);
    let file_bytes_after = if dry_run || report.is_empty() {
        None
    } else {
        atomic_write_path(path, |tmp_path| vpx::write(tmp_path, &vpx))?;
        Some(std::fs::metadata(path)?.len())
    };
    Ok(Outcome {
        report,
        file_bytes_before,
        file_bytes_after,
    })
}

/// Formats the outcome as a report: one line per change or skip, then a
/// summary
pub fn format_outcome(path: &Path, outcome: &Outcome, dry_run: bool) -> String {
    let mut out = format!("{}\n", path.display().to_string().bold());
    let report = &outcome.report;
    for font in &report.removed_fonts {
        out.push_str(&format!(
            "  font {:?} removed, {}\n",
            font.name(),
            human_bytes(font.bytes() as u64)
        ));
    }
    for (images, from) in [
        (&report.bitmaps, "bitmap"),
        (&report.pngs, "png"),
        (&report.tgas, "tga"),
    ] {
        for image in images {
            out.push_str(&format!(
                "  image {:?} {from} -> webp, {} -> {}\n",
                image.name(),
                human_bytes(image.bytes_before() as u64),
                human_bytes(image.bytes_after() as u64)
            ));
        }
    }
    for skipped in &report.skipped {
        out.push_str(&format!(
            "  image {:?} left alone: {}\n",
            skipped.name(),
            skipped.reason()
        ));
    }
    let summary = if report.is_empty() {
        "nothing to optimize".to_string()
    } else if let Some(after) = outcome.file_bytes_after {
        format!(
            "saved {}, file {} -> {}",
            human_bytes(outcome.file_bytes_before.saturating_sub(after)),
            human_bytes(outcome.file_bytes_before),
            human_bytes(after)
        )
    } else if dry_run {
        format!(
            "would save {} of stored data, file not written",
            human_bytes(report.bytes_saved() as u64)
        )
    } else {
        format!(
            "saved {} of stored data",
            human_bytes(report.bytes_saved() as u64)
        )
    };
    out.push_str(&format!("  {summary}"));
    out
}

/// A byte count for people: `812 B`, `3.4 KB`, `12.0 MB`
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if bytes < 1000 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = "B";
    for next in UNITS {
        if value < 1000.0 {
            break;
        }
        value /= 1000.0;
        unit = next;
    }
    format!("{value:.1} {unit}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use vpin::vpx::image::ImageData;
    use vpin::vpx::pinbinary::PinBinary;

    /// A png stored with the fastest, unfiltered compression, which webp
    /// makes much smaller
    fn loose_png(name: &str) -> ImageData {
        use image::codecs::png::{CompressionType, FilterType, PngEncoder};
        let pixels = image::RgbaImage::from_fn(64, 64, |x, y| {
            image::Rgba([x as u8, y as u8, (x + y) as u8, 255])
        });
        let mut data = Vec::new();
        pixels
            .write_with_encoder(PngEncoder::new_with_quality(
                &mut data,
                CompressionType::Fast,
                FilterType::NoFilter,
            ))
            .expect("png encodes");
        ImageData {
            name: name.to_string(),
            path: format!("C:\\images\\{name}.png"),
            width: 64,
            height: 64,
            jpeg: Some(PinBinary {
                path: format!("C:\\images\\{name}.png"),
                name: name.to_string(),
                internal_name: None,
                data,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn pngs_are_converted_and_flexdmd_images_reported() {
        let mut vpx = VPX::default();
        vpx.add_or_replace_image(loose_png("apron"));
        vpx.add_or_replace_image(loose_png("logo"));
        vpx.gamedata.set_code(
            "Option Explicit\r\nSet img = FlexDMD.NewImage(\"logo\", \"VPX.Logo\")\r\n".to_string(),
        );

        let report = apply(&mut vpx);

        assert_eq!(report.pngs.len(), 1);
        assert_eq!(report.pngs[0].name(), "apron");
        assert_eq!(report.skipped.len(), 1);
        assert_eq!(report.skipped[0].name(), "logo");
        assert!(!report.is_empty());
        assert!(report.bytes_saved() > 0);
        assert_eq!(vpx.images[0].ext(), "webp");
        assert_eq!(vpx.images[1].ext(), "png");

        let outcome = Outcome {
            report,
            file_bytes_before: 20_000,
            file_bytes_after: Some(12_345),
        };
        colored::control::set_override(false);
        let plain = format_outcome(Path::new("t.vpx"), &outcome, false);
        assert!(plain.contains("image \"apron\" png -> webp, "), "{plain}");
        assert!(
            plain.contains("image \"logo\" left alone: FlexDMD reads it and cannot read webp"),
            "{plain}"
        );
        assert!(
            plain.ends_with("saved 7.7 KB, file 20.0 KB -> 12.3 KB"),
            "{plain}"
        );
    }

    #[test]
    fn a_table_with_nothing_to_do_says_so() {
        let outcome = Outcome {
            report: Report::default(),
            file_bytes_before: 500,
            file_bytes_after: None,
        };
        colored::control::set_override(false);
        let text = format_outcome(Path::new("t.vpx"), &outcome, true);
        assert!(text.ends_with("nothing to optimize"), "{text}");
    }

    #[test]
    fn bytes_read_like_a_file_manager() {
        assert_eq!(human_bytes(812), "812 B");
        assert_eq!(human_bytes(3_400), "3.4 KB");
        assert_eq!(human_bytes(12_000_000), "12.0 MB");
    }
}
