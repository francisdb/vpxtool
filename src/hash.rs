//! Whole-file SHA-256 and the content-hash backup the optimize command
//! writes. The hash is lowercase hex over the raw file bytes, matching
//! `sha256sum`, so a backup names the exact bytes it holds.

use crate::atomicwrite::finalize_temp;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};

/// Streams a file through SHA-256 and returns the digest. A 1 MB buffer keeps
/// the read to a handful of large sequential reads rather than many small ones,
/// which matters for a library served over a NAS, and streaming it means a
/// large table is never held in memory just to hash it.
pub(crate) fn sha256_file(path: &Path) -> io::Result<String> {
    let mut reader = BufReader::with_capacity(1 << 20, File::open(path)?);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1 << 16];
    loop {
        let read = reader.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hex(&hasher.finalize()))
}

/// The sibling path a backup takes: the original path with `.<sha256>` added,
/// so it sits beside the file and keeps the `.vpx` in the name.
pub(crate) fn backup_path_for(path: &Path, sha256_hex: &str) -> PathBuf {
    let mut backup = path.as_os_str().to_owned();
    backup.push(".");
    backup.push(sha256_hex);
    PathBuf::from(backup)
}

/// Copies `original` to a sibling named with its whole-file SHA-256, unless
/// that backup already exists, and returns the backup path when one was made.
///
/// The copy hashes the bytes as it streams them to a temporary sibling, so the
/// original is read once, then renames the temp onto the hash name only after
/// it is fully written and fsynced. An interrupted copy leaves the `.tmp`, not
/// a truncated file under the final name, so a later run never mistakes a
/// partial backup for a complete one. When the hash name already exists the
/// temp is discarded and the existing backup is left untouched, which makes a
/// repeated run idempotent.
pub(crate) fn backup_if_absent(original: &Path) -> io::Result<Option<PathBuf>> {
    let mut tmp_os = original.as_os_str().to_owned();
    tmp_os.push(".backup.tmp");
    let tmp_path = PathBuf::from(tmp_os);

    let result = (|| -> io::Result<Option<PathBuf>> {
        let sha256_hex = {
            let mut reader = BufReader::with_capacity(1 << 20, File::open(original)?);
            let mut writer = File::create(&tmp_path)?;
            let mut hasher = Sha256::new();
            let mut buf = [0u8; 1 << 16];
            loop {
                let read = reader.read(&mut buf)?;
                if read == 0 {
                    break;
                }
                hasher.update(&buf[..read]);
                writer.write_all(&buf[..read])?;
            }
            writer.sync_all()?;
            hex(&hasher.finalize())
        };
        let backup = backup_path_for(original, &sha256_hex);
        if backup.exists() {
            // already backed up: drop the temp, leave the original backup be
            let _ = fs::remove_file(&tmp_path);
            return Ok(None);
        }
        finalize_temp(&tmp_path, &backup)?;
        Ok(Some(backup))
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

/// Renders bytes as lowercase hex.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// a unique scratch dir under the system temp
    fn scratch() -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "vpxtool-hash-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, bytes: &[u8]) {
        let mut f = File::create(path).unwrap();
        f.write_all(bytes).unwrap();
    }

    #[test]
    fn sha256_file_matches_a_known_vector() {
        // echo -n "abc" | sha256sum
        let dir = scratch();
        let p = dir.join("f");
        write(&p, b"abc");
        assert_eq!(
            sha256_file(&p).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn backup_copies_the_original_and_names_it_by_hash() {
        let dir = scratch();
        let original = dir.join("t.vpx");
        write(&original, b"original bytes");
        let sha = sha256_file(&original).unwrap();

        let made = backup_if_absent(&original).unwrap();
        let backup = backup_path_for(&original, &sha);
        assert_eq!(made.as_deref(), Some(backup.as_path()));
        assert_eq!(std::fs::read(&backup).unwrap(), b"original bytes");
        // the original is untouched and no temp is left behind
        assert_eq!(std::fs::read(&original).unwrap(), b"original bytes");
        assert!(!dir.join("t.vpx.backup.tmp").exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn backup_skips_and_leaves_an_existing_backup_untouched() {
        let dir = scratch();
        let original = dir.join("t.vpx");
        write(&original, b"original bytes");
        let sha = sha256_file(&original).unwrap();
        let backup = backup_path_for(&original, &sha);
        // a pre-existing file at the hash name, with different bytes
        write(&backup, b"existing");

        let made = backup_if_absent(&original).unwrap();
        assert_eq!(made, None);
        // left exactly as it was, and no temp survives
        assert_eq!(std::fs::read(&backup).unwrap(), b"existing");
        assert!(!dir.join("t.vpx.backup.tmp").exists());
        std::fs::remove_dir_all(&dir).ok();
    }
}
