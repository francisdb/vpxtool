use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::Metadata;
use std::time::SystemTime;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use colored::Colorize;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use walkdir::WalkDir;

use crate::tableinfo::{read_tableinfo, TableInfo};

/// Introduced because we want full control over serialization
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct IndexedTableInfo {
    pub table_name: Option<String>,
    pub author_name: Option<String>,
    //pub screenshot: Option<Vec<u8>>,
    pub table_blurb: Option<String>,
    pub table_rules: Option<String>,
    pub author_email: Option<String>,
    pub release_date: Option<String>,
    pub table_save_rev: Option<String>,
    pub table_version: Option<String>,
    pub author_website: Option<String>,
    pub table_save_date: Option<String>,
    pub table_description: Option<String>,
    // the keys (and ordering) for these are defined in "GameStg/CustomInfoTags"
    pub properties: HashMap<String, String>,
}
impl From<TableInfo> for IndexedTableInfo {
    fn from(table_info: TableInfo) -> Self {
        IndexedTableInfo {
            table_name: table_info.table_name,
            author_name: table_info.author_name,
            //screenshot: table_info.screenshot, // TODO we might want to write this to a file next to the table?
            table_blurb: table_info.table_blurb,
            table_rules: table_info.table_rules,
            author_email: table_info.author_email,
            release_date: table_info.release_date,
            table_save_rev: table_info.table_save_rev,
            table_version: table_info.table_version,
            author_website: table_info.author_website,
            table_save_date: table_info.table_save_date,
            table_description: table_info.table_description,
            properties: table_info.properties,
        }
    }
}

pub struct PathWithMetadata {
    pub path: PathBuf,
    pub last_modified: SystemTime,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct IsoSystemTime(SystemTime);
impl From<SystemTime> for IsoSystemTime {
    fn from(system_time: SystemTime) -> Self {
        IsoSystemTime(system_time)
    }
}
impl From<IsoSystemTime> for SystemTime {
    fn from(iso_system_time: IsoSystemTime) -> Self {
        iso_system_time.0
    }
}
impl Serialize for IsoSystemTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let now: DateTime<Utc> = self.0.into();
        now.to_rfc3339().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for IsoSystemTime {
    fn deserialize<D>(deserializer: D) -> Result<IsoSystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
        Ok(IsoSystemTime(dt.into()))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct IndexedTable {
    pub path: PathBuf,
    pub table_info: IndexedTableInfo,
    pub last_modified: IsoSystemTime,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TablesIndex {
    pub tables: Vec<IndexedTable>,
}

impl TablesIndex {
    pub(crate) fn empty() -> TablesIndex {
        TablesIndex { tables: vec![] }
    }

    pub fn add_all(&mut self, new_tables: Vec<IndexedTable>) {
        self.tables.extend(new_tables);
    }

    pub(crate) fn should_index(&self, path_with_metadata: &PathWithMetadata) -> bool {
        // if exists with different last modified or missing
        let existing = self
            .tables
            .iter()
            .find(|t| t.path == path_with_metadata.path);
        match existing {
            Some(existing) => {
                let existing_last_modified: SystemTime = existing.last_modified.into();
                existing_last_modified != path_with_metadata.last_modified
            }
            None => true,
        }
    }

    pub(crate) fn remove_missing(&mut self, paths: &[PathWithMetadata]) -> usize {
        // create a hashset with the paths
        let len = self.tables.len();
        let paths_set: HashSet<PathBuf> = paths.iter().map(|p| p.path.clone()).collect();
        self.tables.retain(|t| paths_set.contains(&t.path));
        len - self.tables.len()
    }
}

pub fn find_vpx_files<P: AsRef<Path>>(
    recursive: bool,
    tables_path: P,
) -> io::Result<Vec<PathWithMetadata>> {
    if recursive {
        let mut vpx_files = Vec::new();
        let mut entries = WalkDir::new(tables_path).into_iter();
        entries.try_for_each(|entry| {
            let dir_entry = entry?;
            let path = dir_entry.path();
            if path.is_file() {
                if let Some("vpx") = path.extension().and_then(OsStr::to_str) {
                    let last_modified = last_modified(path)?;
                    vpx_files.push(PathWithMetadata {
                        path: path.to_path_buf(),
                        last_modified,
                    });
                }
            }
            Ok::<(), io::Error>(())
        })?;
        Ok(vpx_files)
    } else {
        let mut vpx_files = Vec::new();
        // TODO is there a cleaner version like try_filter_map?
        let mut dirs = fs::read_dir(tables_path)?;
        dirs.try_for_each(|entry| {
            let dir_entry = entry?;
            let path = dir_entry.path();
            if path.is_file() {
                if let Some("vpx") = path.extension().and_then(OsStr::to_str) {
                    let last_modified = last_modified(&path)?;
                    vpx_files.push(PathWithMetadata {
                        path: path.to_path_buf(),
                        last_modified,
                    });
                }
            }
            Ok::<(), io::Error>(())
        })?;
        Ok(vpx_files)
    }
}

pub trait Progress {
    fn set_length(&self, len: u64);
    fn set_position(&self, i: u64);
    fn finish_and_clear(&self);
}
// void progress
struct VoidProgress;
impl Progress for VoidProgress {
    fn set_length(&self, _len: u64) {}
    fn set_position(&self, _i: u64) {}
    fn finish_and_clear(&self) {}
}

/// Indexes all vpx files in the given folder and writes the index to a file.
/// Returns the index.
/// If the index file already exists, it will be read and updated.
/// If the index file does not exist, it will be created.
pub fn index_folder<P: AsRef<Path>>(
    recursive: bool,
    tables_folder: P,
    tables_index_path: P,
    progress: &impl Progress,
) -> io::Result<TablesIndex> {
    println!("Indexing {}", tables_folder.as_ref().display());

    let existing_index = read_index_json(&tables_index_path)?;
    if let Some(index) = &existing_index {
        println!("  Found existing index with {} files", index.tables.len());
    }
    let mut index = existing_index.unwrap_or(TablesIndex::empty());

    let vpx_files = find_vpx_files(recursive, tables_folder)?;
    println!("  Found {} files", vpx_files.len());
    // remove files that are missing
    let removed_len = index.remove_missing(&vpx_files);
    println!("  {} missing files have been removed", removed_len);

    // find files that are missing or have been modified
    let mut vpx_files_to_index = Vec::new();
    for vpx_file in vpx_files {
        if index.should_index(&vpx_file) {
            vpx_files_to_index.push(vpx_file);
        }
    }

    println!("  {} files need (re)indexing.", vpx_files_to_index.len());
    let vpx_files_with_tableinfo = index_vpx_files(&vpx_files_to_index, progress);

    // add new files to index
    index.add_all(vpx_files_with_tableinfo.tables);

    // write the index to a file
    write_index_json(&index, tables_index_path)?;

    Ok(index)
}

pub fn index_vpx_files(vpx_files: &[PathWithMetadata], progress: &impl Progress) -> TablesIndex {
    // TODO tried using rayon here but it's not faster and uses up all cpu
    // use rayon::prelude::*;
    // .par_iter() instead of .iter()
    progress.set_length(vpx_files.len() as u64);

    let mut vpx_files_with_tableinfo: Vec<IndexedTable> = vpx_files
        .iter()
        .enumerate()
        .flat_map(|(i, vpx_file)| {
            let result = cfb::open(&vpx_file.path).and_then(|mut comp| read_tableinfo(&mut comp));
            let optional = match result {
                Ok(table_info) => {
                    let last_modified = last_modified(&vpx_file.path).unwrap();
                    let indexed_table_info = IndexedTableInfo::from(table_info);
                    let indexed = IndexedTable {
                        path: vpx_file.path.clone(),
                        table_info: indexed_table_info,
                        last_modified: IsoSystemTime(last_modified),
                    };
                    Some(indexed)
                }
                Err(e) => {
                    // TODO we want to return any failures instead of printing here
                    let warning =
                        format!("Not a valid vpx file {}: {}", vpx_file.path.display(), e).red();
                    println!("{}", warning);
                    None
                }
            };
            progress.set_position((i + 1) as u64);
            optional
        })
        .collect();

    // sort by name
    vpx_files_with_tableinfo.sort_by(|a, b| table_name_compare(a, b));
    TablesIndex {
        tables: vpx_files_with_tableinfo,
    }
}

fn table_name_compare(a: &IndexedTable, b: &IndexedTable) -> Ordering {
    // TODO get rid of clone() here
    let a_lower = a
        .table_info
        .table_name
        .clone()
        .unwrap_or("".to_string())
        .to_lowercase();
    let b_lower = b
        .table_info
        .table_name
        .clone()
        .unwrap_or("".to_string())
        .to_lowercase();
    a_lower.cmp(&b_lower)
}

fn last_modified<P: AsRef<Path>>(path: P) -> io::Result<SystemTime> {
    let metadata: Metadata = path.as_ref().metadata()?;
    metadata.modified()
}

pub fn write_index_json<P: AsRef<Path>>(
    indexed_tables: &TablesIndex,
    json_path: P,
) -> io::Result<()> {
    let json_file = File::create(json_path)?;
    serde_json::to_writer_pretty(json_file, &indexed_tables)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub fn read_index_json<P: AsRef<Path>>(json_path: P) -> io::Result<Option<TablesIndex>> {
    if !json_path.as_ref().exists() {
        return Ok(None);
    }
    let json_file = File::open(json_path)?;
    serde_json::from_reader(json_file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vpx;
    use testdir::testdir;

    #[test]
    fn test_index_vpx_files() -> io::Result<()> {
        // create empty files in testdir
        let test_dir = testdir!();
        let vpx_1_path = test_dir.join("test.vpx");
        let vpx_2_path = test_dir.join("test2.vpx");
        vpx::new_minimal_vpx(&vpx_1_path)?;
        vpx::new_minimal_vpx(&vpx_2_path)?;
        let vpx_files = vec![
            PathWithMetadata {
                path: vpx_1_path.clone(),
                last_modified: SystemTime::UNIX_EPOCH,
            },
            PathWithMetadata {
                path: vpx_2_path.clone(),
                last_modified: SystemTime::UNIX_EPOCH,
            },
        ];
        println!("vpx_files");
        let indexed_tables = index_vpx_files(&vpx_files, &VoidProgress);
        assert_eq!(indexed_tables.tables.len(), 2);
        assert_eq!(indexed_tables.tables[0].path, vpx_1_path);
        assert_eq!(indexed_tables.tables[1].path, vpx_2_path);
        Ok(())
    }

    #[test]
    fn test_write_read_empty_index() -> io::Result<()> {
        let index = TablesIndex::empty();
        let test_dir = testdir!();
        let index_path = test_dir.join("test.json");
        write_index_json(&index, &index_path)?;
        let read = read_index_json(&index_path)?;
        assert_eq!(read, Some(index));
        Ok(())
    }

    #[test]
    fn test_write_read_single_item_index() -> io::Result<()> {
        let index = TablesIndex {
            tables: vec![IndexedTable {
                path: PathBuf::from("test.vpx"),
                table_info: IndexedTableInfo {
                    table_name: Some("test".to_string()),
                    author_name: Some("test".to_string()),
                    table_blurb: None,
                    table_rules: None,
                    author_email: None,
                    release_date: None,
                    table_save_rev: None,
                    table_version: None,
                    author_website: None,
                    table_save_date: None,
                    table_description: None,
                    properties: HashMap::new(),
                },
                last_modified: IsoSystemTime(SystemTime::UNIX_EPOCH),
            }],
        };
        let test_dir = testdir!();
        let index_path = test_dir.join("test.json");
        write_index_json(&index, &index_path)?;
        let read = read_index_json(&index_path)?;
        assert_eq!(read, Some(index));
        Ok(())
    }

    #[test]
    fn test_read_index_missing() -> io::Result<()> {
        let index_path = PathBuf::from("missing_index_file.json");
        let read = read_index_json(&index_path)?;
        assert_eq!(read, None);
        Ok(())
    }
}
