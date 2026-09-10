//! Launch history of the frontend: one JSON line per vpinball run.
//!
//! The log is append only and lives in the platform data directory
//! (`~/.local/share/vpxtool/play_log.jsonl` on Linux). Reading folds it
//! into per table statistics; runs shorter than [`MIN_PLAY_SECONDS`]
//! stay in the log but do not count as a play, so a table that crashed
//! on load does not show up as recently played.

use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Runs shorter than this are a failed or aborted start, not a play
pub const MIN_PLAY_SECONDS: u64 = 30;

/// One vpinball run started from the frontend
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayRecord {
    /// Absolute path of the vpx file
    pub path: PathBuf,
    /// Start time as RFC 3339 in UTC, which sorts chronologically as text
    pub started: String,
    /// How long vpinball ran
    pub seconds: u64,
    /// "ok" for a clean exit, otherwise how vpinball ended
    pub exit: String,
}

/// What the log knows about one table
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableStats {
    pub plays: u32,
    /// Start time of the most recent play, RFC 3339 UTC
    pub last_played: Option<String>,
    pub total_seconds: u64,
}

impl TableStats {
    /// The date part of the last play, for display
    pub fn last_played_date(&self) -> Option<&str> {
        self.last_played
            .as_deref()
            .map(|started| &started[..10.min(started.len())])
    }
}

/// The play log location, `None` when the platform has no data directory
pub fn play_log_path() -> Option<PathBuf> {
    dirs::data_dir().map(|data_dir| data_dir.join("vpxtool").join("play_log.jsonl"))
}

/// Appends one record, creating the log and its directory when needed
pub fn append(path: &Path, record: &PlayRecord) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(record)?;
    writeln!(file, "{line}")
}

/// Reads all records, skipping lines that do not parse. A missing log is empty.
pub fn read(path: &Path) -> io::Result<Vec<PlayRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let reader = BufReader::new(File::open(path)?);
    let mut records = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<PlayRecord>(&line) {
            Ok(record) => records.push(record),
            Err(e) => warn!("Skipping play log line {}: {e}", index + 1),
        }
    }
    Ok(records)
}

/// Folds the records into per table statistics, counting only real plays
pub fn fold(records: &[PlayRecord]) -> HashMap<PathBuf, TableStats> {
    let mut stats: HashMap<PathBuf, TableStats> = HashMap::new();
    for record in records {
        if record.seconds < MIN_PLAY_SECONDS {
            continue;
        }
        let entry = stats.entry(record.path.clone()).or_default();
        entry.plays += 1;
        entry.total_seconds += record.seconds;
        if entry
            .last_played
            .as_ref()
            .is_none_or(|last| record.started > *last)
        {
            entry.last_played = Some(record.started.clone());
        }
    }
    stats
}

/// Reads and folds the log at the default location, empty when there is none
pub fn load_stats() -> HashMap<PathBuf, TableStats> {
    let Some(path) = play_log_path() else {
        return HashMap::new();
    };
    match read(&path) {
        Ok(records) => fold(&records),
        Err(e) => {
            warn!("Unable to read play log {}: {e}", path.display());
            HashMap::new()
        }
    }
}

/// A short human duration: "45s", "12m", "2h 05m"
pub fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let hours = minutes / 60;
    if hours > 0 {
        format!("{hours}h {:02}m", minutes % 60)
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        format!("{seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn record(path: &str, started: &str, seconds: u64) -> PlayRecord {
        PlayRecord {
            path: PathBuf::from(path),
            started: started.to_string(),
            seconds,
            exit: "ok".to_string(),
        }
    }

    #[test]
    fn append_and_read_round_trip() -> io::Result<()> {
        let dir = testdir::testdir!();
        let path = dir.join("nested").join("play_log.jsonl");
        let first = record("/tables/a.vpx", "2026-09-10T20:00:00Z", 120);
        let second = record("/tables/b.vpx", "2026-09-10T21:00:00Z", 5);
        append(&path, &first)?;
        append(&path, &second)?;

        assert_eq!(read(&path)?, vec![first, second]);
        Ok(())
    }

    #[test]
    fn read_skips_bad_lines_and_missing_log() -> io::Result<()> {
        let dir = testdir::testdir!();
        let path = dir.join("play_log.jsonl");
        assert_eq!(read(&path)?, Vec::new());

        let good = record("/tables/a.vpx", "2026-09-10T20:00:00Z", 120);
        std::fs::write(
            &path,
            format!("not json\n{}\n\n", serde_json::to_string(&good)?),
        )?;
        assert_eq!(read(&path)?, vec![good]);
        Ok(())
    }

    #[test]
    fn fold_counts_plays_above_the_threshold() {
        let records = vec![
            record("/tables/a.vpx", "2026-09-01T10:00:00Z", 600),
            record("/tables/a.vpx", "2026-09-03T10:00:00Z", 10),
            record("/tables/a.vpx", "2026-09-02T10:00:00Z", 300),
            record("/tables/b.vpx", "2026-09-05T10:00:00Z", 29),
        ];
        let stats = fold(&records);
        assert_eq!(
            stats.get(Path::new("/tables/a.vpx")),
            Some(&TableStats {
                plays: 2,
                last_played: Some("2026-09-02T10:00:00Z".to_string()),
                total_seconds: 900,
            })
        );
        assert_eq!(stats.get(Path::new("/tables/b.vpx")), None);
    }

    #[test]
    fn duration_formatting() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(12 * 60 + 30), "12m");
        assert_eq!(format_duration(2 * 3600 + 5 * 60), "2h 05m");
    }
}
