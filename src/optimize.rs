//! The `optimize` command: the fixes from [`vpin::vpx::fix`] that keep a
//! table rendering and playing the same, and the repairs that make it play
//! in current vpinball, applied in memory and written back in one go, which
//! also compacts the file.

use crate::atomicwrite::atomic_write_path;
use colored::Colorize;
use std::io;
use std::path::Path;
use vpin::vpx;
use vpin::vpx::VPX;
use vpin::vpx::fix::{
    self, ConvertedImage, ConvertedSound, ImageConversion, RemovedFont, RenamedSound, SkippedImage,
    SkippedSound, SoundConversion,
};

/// Which fixes [`apply`] runs. The lossless image and font fixes and the
/// backglass marker repair always run; the mono downmix and the FLAC
/// conversion are opt-in because they change how vpinball 10.8.0 plays the
/// table: it plays stereo playfield sounds in stereo in its two speaker
/// mode, and only miniaudio (10.8.1 and later) decodes FLAC.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// Downmix stereo playfield sounds to the mono vpinball plays them as
    pub mono: bool,
    /// Re-encode PCM WAV sounds as FLAC
    pub flac: bool,
}

/// What [`apply`] did to a table
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// The embedded fonts nothing used
    pub removed_fonts: Vec<RemovedFont>,
    /// The sounds with the `* Backglass Output *` path, given a `.wav` one
    pub renamed_sounds: Vec<RenamedSound>,
    /// The bitmaps re-encoded as webp
    pub bitmaps: Vec<ConvertedImage>,
    /// The pngs re-encoded as webp
    pub pngs: Vec<ConvertedImage>,
    /// The tgas re-encoded as webp
    pub tgas: Vec<ConvertedImage>,
    /// The candidate images left alone, with the reason
    pub skipped: Vec<SkippedImage>,
    /// The stereo playfield sounds downmixed to mono, when `--mono` is set
    pub monos: Vec<ConvertedSound>,
    /// The PCM WAV sounds re-encoded as FLAC, when `--flac` is set
    pub flacs: Vec<ConvertedSound>,
    /// The candidate WAV sounds left alone, with the reason
    pub skipped_sounds: Vec<SkippedSound>,
}

impl Report {
    /// Whether the table was left as it was
    pub fn is_empty(&self) -> bool {
        self.removed_fonts.is_empty()
            && self.renamed_sounds.is_empty()
            && self.bitmaps.is_empty()
            && self.pngs.is_empty()
            && self.tgas.is_empty()
            && self.monos.is_empty()
            && self.flacs.is_empty()
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
        let sounds = self
            .monos
            .iter()
            .chain(&self.flacs)
            .map(|sound| sound.bytes_before().saturating_sub(sound.bytes_after()))
            .sum::<usize>();
        images
            + sounds
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

    fn take_sounds(&mut self, conversion: SoundConversion) -> Vec<ConvertedSound> {
        self.skipped_sounds
            .extend(conversion.skipped().iter().cloned());
        conversion.converted().to_vec()
    }
}

/// Applies the fixes to the table in memory: drops the fonts nothing uses,
/// gives the sounds with the `* Backglass Output *` path a `.wav` one and
/// re-encodes bitmap, png and tga images as lossless webp where that is
/// smaller, always; when [`Options::mono`] is set, downmixes stereo
/// playfield sounds to mono; and when [`Options::flac`] is set, re-encodes
/// PCM WAV sounds as lossless FLAC. Images the script hands to FlexDMD are
/// left alone. The sound fixes run in that order, so the FLAC only encodes
/// the downmixed, renamed sounds.
pub fn apply(vpx: &mut VPX, options: Options) -> Report {
    let mut report = Report {
        removed_fonts: fix::drop_unused_fonts(vpx),
        renamed_sounds: fix::rename_backglass_marker_sounds(vpx),
        ..Report::default()
    };
    report.bitmaps = report.take(fix::bitmaps_to_webp(vpx));
    report.pngs = report.take(fix::pngs_to_webp(vpx));
    report.tgas = report.take(fix::tgas_to_webp(vpx));
    if options.mono {
        report.monos = report.take_sounds(fix::playfield_sounds_to_mono(vpx));
    }
    if options.flac {
        report.flacs = report.take_sounds(fix::wavs_to_flac(vpx));
    }
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
pub fn optimize_file(path: &Path, options: Options, dry_run: bool) -> io::Result<Outcome> {
    let file_bytes_before = std::fs::metadata(path)?.len();
    let mut vpx = vpx::read(path)?;
    let report = apply(&mut vpx, options);
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
    for sound in &report.renamed_sounds {
        out.push_str(&format!(
            "  sound {:?} path {:?} -> {:?}\n",
            sound.name(),
            sound.path_before(),
            sound.path_after()
        ));
    }
    for sound in &report.monos {
        out.push_str(&format!(
            "  sound {:?} stereo -> mono, {} -> {}\n",
            sound.name(),
            human_bytes(sound.bytes_before() as u64),
            human_bytes(sound.bytes_after() as u64)
        ));
    }
    for sound in &report.flacs {
        out.push_str(&format!(
            "  sound {:?} wav -> flac, {} -> {}\n",
            sound.name(),
            human_bytes(sound.bytes_before() as u64),
            human_bytes(sound.bytes_after() as u64)
        ));
    }
    for skipped in &report.skipped_sounds {
        out.push_str(&format!(
            "  sound {:?} left alone: {}\n",
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

        let report = apply(&mut vpx, Options::default());

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

    /// A PCM WAV sound: raw interleaved 16-bit samples, a slowly varying
    /// ramp flac compresses well
    fn pcm_wav(name: &str) -> vpin::vpx::sound::SoundData {
        let frames = 4096;
        let mut data = Vec::with_capacity(frames * 2 * 2);
        for frame in 0..frames {
            for ch in 0..2i32 {
                let v = (((frame as i32 + ch * 7) % 97) - 48) as i16;
                data.extend_from_slice(&v.to_le_bytes());
            }
        }
        vpin::vpx::sound::SoundData {
            name: name.to_string(),
            path: format!("C:\\sounds\\{name}.wav"),
            wave_form: vpin::vpx::sound::WaveForm {
                format_tag: 1,
                channels: 2,
                samples_per_sec: 44100,
                avg_bytes_per_sec: 44100 * 4,
                block_align: 4,
                bits_per_sample: 16,
                cb_size: 0,
            },
            data,
            internal_name: String::new(),
            fade: 0,
            volume: 0,
            balance: 0,
            output_target: vpin::vpx::sound::OutputTarget::Table,
        }
    }

    #[test]
    fn a_backglass_marker_stereo_sound_is_renamed_downmixed_then_flac() {
        let mut vpx = VPX::default();
        let mut sound = pcm_wav("bell");
        sound.path = "* Backglass Output *".to_string();
        vpx.sounds.push(sound);

        // the rename is a repair and always runs
        let report = apply(&mut vpx, Options::default());
        assert_eq!(report.renamed_sounds.len(), 1);
        assert_eq!(vpx.sounds[0].path, "bell.wav");
        assert_eq!(vpx.sounds[0].wave_form.channels, 2);

        let report = apply(
            &mut vpx,
            Options {
                mono: true,
                flac: true,
            },
        );
        assert_eq!(report.monos.len(), 1);
        assert_eq!(
            report.monos[0].bytes_after(),
            report.monos[0].bytes_before() / 2
        );
        assert_eq!(report.flacs.len(), 1);
        let sound = &vpx.sounds[0];
        assert_eq!(sound.path, "bell.flac");
        // the FLAC encodes the downmix: one channel in its STREAMINFO
        assert!(sound.data.starts_with(b"fLaC"));
        assert_eq!((sound.data[20] >> 1) & 0x07, 0);
    }

    #[test]
    fn flac_is_opt_in_and_reported() {
        let mut vpx = VPX::default();
        vpx.sounds.push(pcm_wav("fx"));

        // off by default: the wav is untouched
        let report = apply(&mut vpx, Options::default());
        assert!(report.flacs.is_empty());
        assert!(vpx.sounds[0].path.ends_with(".wav"));

        // opt in: the wav becomes flac and is reported
        let report = apply(
            &mut vpx,
            Options {
                flac: true,
                ..Options::default()
            },
        );
        assert_eq!(report.flacs.len(), 1);
        assert_eq!(report.flacs[0].name(), "fx");
        assert!(report.bytes_saved() > 0);
        assert!(vpx.sounds[0].path.ends_with(".flac"));
        assert!(vpx.sounds[0].data.starts_with(b"fLaC"));

        let outcome = Outcome {
            report,
            file_bytes_before: 20_000,
            file_bytes_after: Some(12_345),
        };
        colored::control::set_override(false);
        let plain = format_outcome(Path::new("t.vpx"), &outcome, false);
        assert!(plain.contains("sound \"fx\" wav -> flac, "), "{plain}");
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
