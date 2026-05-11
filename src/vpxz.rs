//! Build a `.vpxz` archive (a renamed zip) for transfer to the Visual Pinball
//! mobile app.
//!
//! Strategy: bundle the vpx's parent folder recursively, dropping
//! - any *other* `*.vpx` files (the mobile importer rejects archives with more
//!   than one .vpx),
//! - same-directory stem-keyed sidecars of each other vpx,
//! - any `.directb2s` whose stem does not match the chosen vpx,
//! - anything matching a user-configured glob in `vpxz_excludes`.
//!
//! Optionally inject the matching pinmame rom zip from a configured pinmame
//! folder if it is not already present in the source tree.

use crate::indexer;
use crate::indexer::Progress;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

pub struct VpxzExportOptions<'a> {
    /// User-configured glob patterns (gitignore-ish) of paths to exclude,
    /// matched relative to the vpx's parent folder.
    pub exclude_globs: &'a [String],
    /// Absolute path to a rom zip to inject as `pinmame/roms/<basename>` if it
    /// is not already present somewhere inside the vpx's parent folder.
    pub rom_zip: Option<&'a Path>,
    /// Optional progress sink. `set_length` is called once after the directory
    /// walk with the file count, then `set_position` once per file as it is
    /// processed (included or excluded), and `finish_and_clear` at the end.
    pub progress: Option<&'a dyn Progress>,
}

#[derive(Debug)]
pub struct VpxzReport {
    pub output: PathBuf,
    /// Archive-relative paths actually written, in insertion order.
    pub included: Vec<String>,
    /// Source-relative paths dropped, paired with the reason they were dropped.
    pub excluded: Vec<(String, ExcludeReason)>,
    /// Source path of a rom we injected from outside the tree, if any.
    pub injected_rom: Option<PathBuf>,
}

impl VpxzReport {
    /// Whether a `pinmame/roms/<rom_name>.zip` ended up in the archive, either
    /// from the source tree or via injection. Case-insensitive on the file name.
    pub fn rom_bundled(&self, rom_name: &str) -> bool {
        let suffix = format!("pinmame/roms/{}.zip", rom_name.to_lowercase());
        self.included
            .iter()
            .any(|p| p.to_lowercase().ends_with(&suffix))
    }
}

/// Why a file under the vpx's parent folder did not end up in the archive.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ExcludeReason {
    /// Another `.vpx` file in the tree; the mobile importer rejects archives
    /// that contain more than one .vpx.
    OtherVpx,
    /// Same directory and stem as an excluded `OtherVpx`; treated as that
    /// other vpx's sidecar.
    OtherVpxSidecar,
    /// `.directb2s` whose stem doesn't match the chosen vpx's stem.
    UnrelatedDirectb2s,
    /// A previously generated `.vpxz` archive sitting in the tree.
    VpxzArchive,
    /// Matched a user-configured glob in `vpxz_excludes`.
    UserGlob,
}

impl std::fmt::Display for ExcludeReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ExcludeReason::OtherVpx => "other_vpx",
            ExcludeReason::OtherVpxSidecar => "other_vpx_sidecar",
            ExcludeReason::UnrelatedDirectb2s => "unrelated_directb2s",
            ExcludeReason::VpxzArchive => "vpxz_archive",
            ExcludeReason::UserGlob => "vpxz_excludes",
        };
        f.write_str(s)
    }
}

/// Write a vpxz archive containing the table and its assets.
pub fn export_vpxz(
    vpx_path: &Path,
    output_path: &Path,
    options: &VpxzExportOptions,
) -> io::Result<VpxzReport> {
    let stem = vpx_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("vpx path has no usable file stem: {}", vpx_path.display()),
            )
        })?
        .to_string();
    let parent = vpx_path
        .parent()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("vpx path has no parent directory: {}", vpx_path.display()),
            )
        })?
        .to_path_buf();

    let exclude_set = build_glob_set(options.exclude_globs)?;
    let auto_excluded_paths = collect_auto_excluded_paths(&parent, vpx_path, &stem)?;

    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(BufWriter::new(file));
    let file_opts =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut included = Vec::new();
    let mut excluded = Vec::new();
    let mut rom_already_in_tree = false;

    let entries = walkdir(&parent)?;
    if let Some(p) = options.progress {
        p.set_length(entries.len() as u64);
    }

    for (i, entry) in entries.into_iter().enumerate() {
        let abs = entry.path;
        let rel = abs.strip_prefix(&parent).unwrap();
        let rel_str = rel.to_string_lossy().into_owned();

        if let Some(reason) = auto_excluded_paths.get(&abs) {
            excluded.push((rel_str, *reason));
        } else if exclude_set.is_match(rel) {
            excluded.push((rel_str, ExcludeReason::UserGlob));
        } else {
            let rel_archive = to_archive_path(rel);
            let archive_path = format!("{stem}/{rel_archive}");
            if let Some(rom_zip) = options.rom_zip
                && let Some(rom_name_lower) = file_name_lower(rom_zip)
                && rel_archive
                    .to_lowercase()
                    .ends_with(&format!("pinmame/roms/{rom_name_lower}"))
            {
                rom_already_in_tree = true;
            }
            add_file(&mut zip, &abs, &archive_path, file_opts)?;
            included.push(archive_path);
        }

        if let Some(p) = options.progress {
            p.set_position((i + 1) as u64);
        }
    }

    let mut injected_rom = None;
    if let Some(rom_zip) = options.rom_zip
        && !rom_already_in_tree
        && rom_zip.is_file()
    {
        let rom_name = rom_zip
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("rom zip path has no file name: {}", rom_zip.display()),
                )
            })?;
        let archive_path = format!("{stem}/pinmame/roms/{rom_name}");
        add_file(&mut zip, rom_zip, &archive_path, file_opts)?;
        included.push(archive_path);
        injected_rom = Some(rom_zip.to_path_buf());
    }

    zip.finish().map_err(io::Error::other)?;
    if let Some(p) = options.progress {
        p.finish_and_clear();
    }

    Ok(VpxzReport {
        output: output_path.to_path_buf(),
        included,
        excluded,
        injected_rom,
    })
}

/// Locate the rom zip for a vpx: read the script for the rom name, then probe
/// the configured / global pinmame folders. Returns `Ok(None)` for non-PinMAME
/// tables or when no rom file can be located.
pub fn find_rom_zip(
    vpx_path: &Path,
    configured_pinmame_folder: Option<&Path>,
    global_pinmame_folder: Option<&Path>,
) -> io::Result<Option<PathBuf>> {
    let Some(rom_name) = indexer::get_romname_from_vpx(vpx_path)? else {
        return Ok(None);
    };
    let rom_file = format!("{}.zip", rom_name.to_lowercase());
    let vpx_parent = vpx_path.parent().unwrap_or(Path::new("."));

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(p) = configured_pinmame_folder {
        let base = if p.is_relative() {
            vpx_parent.join(p)
        } else {
            p.to_path_buf()
        };
        candidates.push(base.join("roms").join(&rom_file));
    }
    if let Some(p) = global_pinmame_folder {
        candidates.push(p.join("roms").join(&rom_file));
    }
    Ok(candidates.into_iter().find(|p| p.is_file()))
}

/// Default output path for a vpxz: parent of the vpx's directory.
pub fn default_output_path(vpx_path: &Path) -> io::Result<PathBuf> {
    let stem = vpx_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("vpx path has no usable file stem: {}", vpx_path.display()),
            )
        })?;
    let parent = vpx_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("vpx path has no parent directory: {}", vpx_path.display()),
        )
    })?;
    let grandparent = parent.parent().unwrap_or(parent);
    Ok(grandparent.join(format!("{stem}.vpxz")))
}

fn build_glob_set(patterns: &[String]) -> io::Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for raw in patterns {
        // Allow trailing "/" to mean "this directory and everything under it",
        // both at the top level and anywhere in the tree. We expand `Foo/` to
        // two globs: `Foo/**` (top-level) and `**/Foo/**` (nested).
        let raw = raw.trim();
        if let Some(dir) = raw.strip_suffix('/') {
            let dir = dir.trim_start_matches("./");
            builder.add(glob_or_err(&format!("{dir}/**"))?);
            builder.add(glob_or_err(&format!("**/{dir}/**"))?);
        } else {
            builder.add(glob_or_err(raw)?);
        }
    }
    builder.build().map_err(io::Error::other)
}

fn glob_or_err(s: &str) -> io::Result<Glob> {
    Glob::new(s).map_err(|e| io::Error::other(format!("invalid vpxz_excludes glob '{s}': {e}")))
}

fn collect_auto_excluded_paths(
    root: &Path,
    chosen_vpx: &Path,
    chosen_stem: &str,
) -> io::Result<HashMap<PathBuf, ExcludeReason>> {
    let mut excludes: HashMap<PathBuf, ExcludeReason> = HashMap::new();

    // Pass 1: find every other .vpx and their stem-keyed sidecars.
    let mut other_stems_by_dir: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    for entry in walkdir(root)? {
        let p = &entry.path;
        if path_has_extension(p, "vpx") && p != chosen_vpx {
            excludes.insert(p.clone(), ExcludeReason::OtherVpx);
            if let (Some(parent), Some(stem)) = (p.parent(), p.file_stem().and_then(|s| s.to_str()))
            {
                other_stems_by_dir
                    .entry(parent.to_path_buf())
                    .or_default()
                    .insert(stem.to_string());
            }
        }
    }

    // Pass 2: anything sharing a directory and an exact stem with an excluded
    // other-vpx is itself excluded as a sidecar.
    for entry in walkdir(root)? {
        let p = &entry.path;
        if excludes.contains_key(p) {
            continue;
        }
        if let (Some(parent), Some(stem)) = (p.parent(), p.file_stem().and_then(|s| s.to_str()))
            && other_stems_by_dir
                .get(parent)
                .map(|s| s.contains(stem))
                .unwrap_or(false)
        {
            excludes.insert(p.clone(), ExcludeReason::OtherVpxSidecar);
        }
    }

    // Pass 3: .directb2s files whose stem does not match the chosen vpx.
    for entry in walkdir(root)? {
        let p = &entry.path;
        if excludes.contains_key(p) {
            continue;
        }
        if path_has_extension(p, "directb2s")
            && p.file_stem().and_then(|s| s.to_str()) != Some(chosen_stem)
        {
            excludes.insert(p.clone(), ExcludeReason::UnrelatedDirectb2s);
        }
    }

    // Pass 4: any .vpxz files - we never bundle a previous archive into a new
    // one, regardless of where it sits in the tree.
    for entry in walkdir(root)? {
        let p = &entry.path;
        if excludes.contains_key(p) {
            continue;
        }
        if path_has_extension(p, "vpxz") {
            excludes.insert(p.clone(), ExcludeReason::VpxzArchive);
        }
    }

    Ok(excludes)
}

struct WalkEntry {
    path: PathBuf,
}

/// Depth-first walk yielding only files (no directories, no symlinks).
fn walkdir(root: &Path) -> io::Result<Vec<WalkEntry>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let read = match std::fs::read_dir(&dir) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("vpxz: cannot read {}: {e}", dir.display());
                continue;
            }
        };
        for entry in read.flatten() {
            let path = entry.path();
            let ft = match entry.file_type() {
                Ok(ft) => ft,
                Err(e) => {
                    log::warn!("vpxz: cannot stat {}: {e}", path.display());
                    continue;
                }
            };
            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                stack.push(path);
            } else if ft.is_file() {
                out.push(WalkEntry { path });
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn path_has_extension(p: &Path, ext: &str) -> bool {
    p.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case(ext))
        .unwrap_or(false)
}

fn file_name_lower(p: &Path) -> Option<String> {
    p.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
}

fn to_archive_path(rel: &Path) -> String {
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn add_file(
    zip: &mut ZipWriter<BufWriter<File>>,
    src: &Path,
    archive_path: &str,
    options: SimpleFileOptions,
) -> io::Result<()> {
    zip.start_file(archive_path, options)
        .map_err(io::Error::other)?;
    let mut reader = BufReader::new(File::open(src)?);
    io::copy(&mut reader, zip)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::io::Read;
    use std::io::Write;
    use testdir::testdir;

    fn write_bytes(path: &Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut f = File::create(path).unwrap();
        f.write_all(bytes).unwrap();
    }

    fn archive_entries(vpxz: &Path) -> BTreeSet<String> {
        let reader = BufReader::new(File::open(vpxz).unwrap());
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect()
    }

    fn archive_bytes(vpxz: &Path, name: &str) -> Vec<u8> {
        let reader = BufReader::new(File::open(vpxz).unwrap());
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let mut entry = archive.by_name(name).unwrap();
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).unwrap();
        buf
    }

    #[test]
    fn bundles_lone_vpx() {
        let dir = testdir!();
        let table_dir = dir.join("MyTable");
        let vpx = table_dir.join("MyTable.vpx");
        write_bytes(&vpx, b"vpx");

        let out = dir.join("MyTable.vpxz");
        let report = export_vpxz(
            &vpx,
            &out,
            &VpxzExportOptions {
                exclude_globs: &[],
                rom_zip: None,
                progress: None,
            },
        )
        .unwrap();

        assert_eq!(
            archive_entries(&out),
            BTreeSet::from(["MyTable/MyTable.vpx".to_string()])
        );
        assert!(report.excluded.is_empty());
        assert!(report.injected_rom.is_none());
    }

    #[test]
    fn excludes_other_vpx_and_their_sidecars() {
        let dir = testdir!();
        let table_dir = dir.join("MyTable");
        let vpx = table_dir.join("MyTable v1.1.vpx");
        write_bytes(&vpx, b"v1.1");
        write_bytes(&table_dir.join("MyTable v1.1.vbs"), b"v1.1-vbs");
        write_bytes(&table_dir.join("MyTable v1.0.vpx"), b"v1.0");
        write_bytes(&table_dir.join("MyTable v1.0.vbs"), b"v1.0-vbs");
        write_bytes(&table_dir.join("MyTable v1.0.ini"), b"v1.0-ini");
        // A shared backglass that does NOT match the chosen stem -> excluded.
        write_bytes(&table_dir.join("MyTable.directb2s"), b"b2s");
        // pinmame folder is preserved untouched
        write_bytes(&table_dir.join("pinmame/roms/foo.zip"), b"rom");

        let out = dir.join("out.vpxz");
        let report = export_vpxz(
            &vpx,
            &out,
            &VpxzExportOptions {
                exclude_globs: &[],
                rom_zip: None,
                progress: None,
            },
        )
        .unwrap();

        let expected: BTreeSet<String> = [
            "MyTable v1.1/MyTable v1.1.vpx",
            "MyTable v1.1/MyTable v1.1.vbs",
            "MyTable v1.1/pinmame/roms/foo.zip",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(archive_entries(&out), expected);

        let excluded_reasons: BTreeSet<ExcludeReason> =
            report.excluded.iter().map(|(_, r)| *r).collect();
        assert!(excluded_reasons.contains(&ExcludeReason::OtherVpx));
        assert!(excluded_reasons.contains(&ExcludeReason::OtherVpxSidecar));
        assert!(excluded_reasons.contains(&ExcludeReason::UnrelatedDirectb2s));
    }

    #[test]
    fn keeps_directb2s_matching_chosen_stem() {
        let dir = testdir!();
        let table_dir = dir.join("Table");
        let vpx = table_dir.join("Table.vpx");
        write_bytes(&vpx, b"vpx");
        write_bytes(&table_dir.join("Table.directb2s"), b"b2s");

        let out = dir.join("Table.vpxz");
        export_vpxz(
            &vpx,
            &out,
            &VpxzExportOptions {
                exclude_globs: &[],
                rom_zip: None,
                progress: None,
            },
        )
        .unwrap();

        assert_eq!(
            archive_entries(&out),
            BTreeSet::from([
                "Table/Table.vpx".to_string(),
                "Table/Table.directb2s".to_string()
            ])
        );
    }

    #[test]
    fn applies_user_exclude_patterns() {
        let dir = testdir!();
        let table_dir = dir.join("Table");
        let vpx = table_dir.join("Table.vpx");
        write_bytes(&vpx, b"vpx");
        write_bytes(&table_dir.join("Downloads/Original.zip"), b"junk");
        write_bytes(&table_dir.join("subdir/Thumbs.db"), b"junk");
        write_bytes(&table_dir.join("good.txt"), b"good");

        let out = dir.join("Table.vpxz");
        export_vpxz(
            &vpx,
            &out,
            &VpxzExportOptions {
                exclude_globs: &["Downloads/".to_string(), "**/Thumbs.db".to_string()],
                rom_zip: None,
                progress: None,
            },
        )
        .unwrap();

        let entries = archive_entries(&out);
        assert!(entries.contains("Table/Table.vpx"));
        assert!(entries.contains("Table/good.txt"));
        assert!(
            !entries.iter().any(|e| e.contains("Downloads")),
            "Downloads/ should be excluded: {entries:?}"
        );
        assert!(
            !entries.iter().any(|e| e.contains("Thumbs.db")),
            "Thumbs.db should be excluded: {entries:?}"
        );
    }

    #[test]
    fn injects_rom_when_not_already_in_tree() {
        let dir = testdir!();
        let table_dir = dir.join("Table");
        let vpx = table_dir.join("Table.vpx");
        write_bytes(&vpx, b"vpx");
        let rom = dir.join("global_pinmame/roms/mygame.zip");
        write_bytes(&rom, b"rom-bytes");

        let out = dir.join("Table.vpxz");
        let report = export_vpxz(
            &vpx,
            &out,
            &VpxzExportOptions {
                exclude_globs: &[],
                rom_zip: Some(&rom),
                progress: None,
            },
        )
        .unwrap();

        let entries = archive_entries(&out);
        assert!(entries.contains("Table/pinmame/roms/mygame.zip"));
        assert_eq!(
            archive_bytes(&out, "Table/pinmame/roms/mygame.zip"),
            b"rom-bytes"
        );
        assert_eq!(report.injected_rom.as_deref(), Some(rom.as_path()));
    }

    #[test]
    fn does_not_double_inject_rom_already_present() {
        let dir = testdir!();
        let table_dir = dir.join("Table");
        let vpx = table_dir.join("Table.vpx");
        write_bytes(&vpx, b"vpx");
        let tree_rom = table_dir.join("pinmame/roms/mygame.zip");
        write_bytes(&tree_rom, b"rom-in-tree");
        let global_rom = dir.join("global/mygame.zip");
        write_bytes(&global_rom, b"rom-from-global");

        let out = dir.join("Table.vpxz");
        let report = export_vpxz(
            &vpx,
            &out,
            &VpxzExportOptions {
                exclude_globs: &[],
                rom_zip: Some(&global_rom),
                progress: None,
            },
        )
        .unwrap();

        let entries = archive_entries(&out);
        assert!(entries.contains("Table/pinmame/roms/mygame.zip"));
        // The in-tree copy wins (no overwrite, no duplicate entry).
        assert_eq!(
            archive_bytes(&out, "Table/pinmame/roms/mygame.zip"),
            b"rom-in-tree"
        );
        assert!(report.injected_rom.is_none());
    }

    #[test]
    fn always_excludes_prior_vpxz_archives() {
        let dir = testdir!();
        let table_dir = dir.join("Table");
        let vpx = table_dir.join("Table.vpx");
        write_bytes(&vpx, b"vpx");
        // A previous export, either next to the vpx or nested deeper:
        write_bytes(&table_dir.join("Table.vpxz"), b"old");
        write_bytes(&table_dir.join("subdir/another.vpxz"), b"old2");

        let out = dir.join("Table.vpxz");
        let report = export_vpxz(
            &vpx,
            &out,
            &VpxzExportOptions {
                exclude_globs: &[],
                rom_zip: None,
                progress: None,
            },
        )
        .unwrap();

        let entries = archive_entries(&out);
        assert!(!entries.iter().any(|e| e.ends_with(".vpxz")));
        let reasons: BTreeSet<ExcludeReason> = report.excluded.iter().map(|(_, r)| *r).collect();
        assert!(reasons.contains(&ExcludeReason::VpxzArchive));
    }

    #[test]
    fn default_output_path_uses_grandparent_and_stem() {
        let p = Path::new("/tables/My Table/Table v1.1.vpx");
        assert_eq!(
            default_output_path(p).unwrap(),
            PathBuf::from("/tables/Table v1.1.vpxz")
        );
    }
}
