use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

/// Write `path` atomically: writes through a sibling `.tmp` file, fsyncs the
/// content, renames on success, then fsyncs the parent directory. The `.tmp`
/// is removed on any error path. An interrupted run (Ctrl-C, crash) leaves
/// the previous file intact instead of a half-written one.
///
/// The parent-directory fsync makes the rename durable against power loss
/// (without it, a crash in the brief window after rename returns could leave
/// the old file under the target name). On filesystems that don't support
/// directory fsync - notably macOS SMB/NFS mounts which return ENOTSUP -
/// we treat it as a soft success, since the rename itself has already
/// happened. This is the same trade-off the popular atomic-write crates
/// make implicitly, except they propagate the unsupported-error and there's
/// no opt-out:
///   - atomicwrites: https://github.com/untitaker/rust-atomicwrites
///     (issue #45 about making the dir fsync optional)
///   - atomic-write-file: https://github.com/andreacorbellini/rust-atomic-write-file
pub(crate) fn atomic_write<F>(path: &Path, write: F) -> io::Result<()>
where
    F: FnOnce(&mut File) -> io::Result<()>,
{
    let mut tmp_os = path.as_os_str().to_owned();
    tmp_os.push(".tmp");
    let tmp_path = PathBuf::from(tmp_os);
    let result = (|| -> io::Result<()> {
        let mut file = File::create(&tmp_path)?;
        write(&mut file)?;
        file.sync_all()?;
        fs::rename(&tmp_path, path)?;
        fsync_parent_best_effort(path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

#[cfg(unix)]
fn fsync_parent_best_effort(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    match File::open(parent).and_then(|f| f.sync_all()) {
        Ok(()) => Ok(()),
        Err(e) if is_dir_fsync_unsupported(&e) => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(not(unix))]
fn fsync_parent_best_effort(_path: &Path) -> io::Result<()> {
    // Windows opens directories with a different API and NTFS journals
    // metadata aggressively; skip parent fsync there.
    Ok(())
}

#[cfg(unix)]
fn is_dir_fsync_unsupported(e: &io::Error) -> bool {
    if e.kind() == io::ErrorKind::Unsupported {
        return true;
    }
    // macOS SMB/NFS returns ENOTSUP (errno 45), which Rust's ErrorKind
    // doesn't classify as Unsupported.
    cfg!(target_os = "macos") && e.raw_os_error() == Some(45)
}
