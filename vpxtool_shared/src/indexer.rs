use chrono::{DateTime, Utc};
use log::info;
use rayon::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fs::Metadata;
use std::io::Read;
use std::time::SystemTime;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};
use vpin::vpx;
use vpin::vpx::jsonmodel::json_to_info;
use vpin::vpx::tableinfo::TableInfo;
use walkdir::{DirEntry, FilterEntry, IntoIter, WalkDir};

use vpx::gamedata::GameData;

pub const DEFAULT_INDEX_FILE_NAME: &str = "vpxtool_index.json";

/// Introduced because we want full control over serialization
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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

#[derive(Clone, Copy, PartialEq, Debug, Eq, Ord, PartialOrd)]
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IndexedTable {
    pub path: PathBuf,
    pub table_info: IndexedTableInfo,
    pub game_name: Option<String>,
    pub b2s_path: Option<PathBuf>,
    /// The rom path, in the table folder or in the global pinmame roms folder
    rom_path: Option<PathBuf>,
    /// deprecated: only used for reading the old index format
    #[serde(skip_serializing_if = "Option::is_none")]
    local_rom_path: Option<PathBuf>,
    pub wheel_path: Option<PathBuf>,
    pub requires_pinmame: bool,
    pub last_modified: IsoSystemTime,
}

impl IndexedTable {
    pub fn rom_path(&self) -> Option<&PathBuf> {
        self.rom_path.as_ref().or(self.local_rom_path.as_ref())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TablesIndex {
    tables: HashMap<PathBuf, IndexedTable>,
}

impl TablesIndex {
    pub(crate) fn empty() -> TablesIndex {
        TablesIndex {
            tables: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.tables.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    pub(crate) fn insert(&mut self, table: IndexedTable) {
        self.tables.insert(table.path.clone(), table);
    }

    pub fn insert_all(&mut self, new_tables: Vec<IndexedTable>) {
        for table in new_tables {
            self.insert(table);
        }
    }

    pub fn merge(&mut self, other: TablesIndex) {
        self.tables.extend(other.tables);
    }

    pub fn tables(&self) -> Vec<IndexedTable> {
        self.tables.values().cloned().collect()
    }

    pub(crate) fn should_index(&self, path_with_metadata: &PathWithMetadata) -> bool {
        // if exists with different last modified or missing
        match self.tables.get(&path_with_metadata.path) {
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
        self.tables.retain(|path, _| paths_set.contains(path));
        len - self.tables.len()
    }
}

/// We prefer keeping a flat index instead of an object
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TablesIndexJson {
    tables: Vec<IndexedTable>,
}

impl From<TablesIndex> for TablesIndexJson {
    fn from(index: TablesIndex) -> Self {
        TablesIndexJson {
            tables: index.tables(),
        }
    }
}

impl From<&TablesIndex> for TablesIndexJson {
    fn from(table: &TablesIndex) -> Self {
        TablesIndexJson {
            tables: table.tables(),
        }
    }
}

impl From<TablesIndexJson> for TablesIndex {
    fn from(index: TablesIndexJson) -> Self {
        let mut tables = HashMap::new();
        for table in index.tables {
            tables.insert(table.path.clone(), table);
        }
        TablesIndex { tables }
    }
}

/// Returns all roms names lower case for the roms in the given folder
pub fn find_roms(rom_path: &Path) -> io::Result<HashMap<String, PathBuf>> {
    if !rom_path.exists() {
        return Ok(HashMap::new());
    }
    // TODO
    // TODO if there is an ini file for the table we might have to check locally for the rom
    //   currently only a standalone feature
    let mut roms = HashMap::new();
    // TODO is there a cleaner version like try_filter_map?
    let mut entries = fs::read_dir(rom_path)?;
    entries.try_for_each(|entry| {
        let dir_entry = entry?;
        let path = dir_entry.path();
        if path.is_file() {
            if let Some("zip") = path.extension().and_then(OsStr::to_str) {
                let rom_name = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .to_lowercase();
                roms.insert(rom_name, path);
            }
        }
        Ok::<(), io::Error>(())
    })?;
    Ok(roms)
}

pub fn find_vpx_files(recursive: bool, tables_path: &Path) -> io::Result<Vec<PathWithMetadata>> {
    if recursive {
        let mut vpx_files = Vec::new();
        let mut entries = walk_dir_filtered(tables_path);
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

/// Walks the directory and filters out .git and __MACOSX folders
fn walk_dir_filtered(tables_path: &Path) -> FilterEntry<IntoIter, fn(&DirEntry) -> bool> {
    WalkDir::new(tables_path).into_iter().filter_entry(|entry| {
        let path = entry.path();
        let git = std::path::Component::Normal(".git".as_ref());
        let macosx = std::path::Component::Normal("__MACOSX".as_ref());
        !path.components().any(|c| c == git) && !path.components().any(|c| c == macosx)
    })
}

pub trait Progress {
    fn set_length(&self, len: u64);
    fn set_position(&self, i: u64);
    fn finish_and_clear(&self);
}

pub struct VoidProgress;
impl Progress for VoidProgress {
    fn set_length(&self, _len: u64) {}
    fn set_position(&self, _i: u64) {}
    fn finish_and_clear(&self) {}
}

pub enum IndexError {
    FolderDoesNotExist(PathBuf),
    IoError(io::Error),
}
impl Debug for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexError::FolderDoesNotExist(path) => {
                write!(f, "Folder does not exist: {}", path.display())
            }
            IndexError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}
impl From<IndexError> for io::Error {
    fn from(e: IndexError) -> io::Error {
        io::Error::new(io::ErrorKind::Other, format!("{:?}", e))
    }
}

impl From<io::Error> for IndexError {
    fn from(e: io::Error) -> Self {
        IndexError::IoError(e)
    }
}

/// Indexes all vpx files in the given folder and writes the index to a file.
/// Returns the index.
/// If the index file already exists, it will be read and updated.
/// If the index file does not exist, it will be created.
///
/// Arguments:
/// * `recursive`: if true, all subdirectories will be searched for vpx files.
/// * `tables_folder`: the folder to search for vpx files.
/// * `tables_index_path`: the path to the index file.
/// * `global_pinmame_path`: the path to the global pinmame folder. Eg ~/.pinmame/roms on *nix systems.
/// * `configured_pinmame_path`: the path to the local pinmame folder configured in the vpinball config.
/// * `progress`: lister for progress updates.
/// * `force_reindex`: a list of vpx files to reindex, even if they are not modified.
pub fn index_folder(
    recursive: bool,
    tables_folder: &Path,
    tables_index_path: &Path,
    global_pinmame_path: Option<&Path>,
    configured_pinmame_path: Option<&Path>,
    progress: &impl Progress,
    force_reindex: Vec<PathBuf>,
) -> Result<TablesIndex, IndexError> {
    info!("Indexing {}", tables_folder.display());

    if !tables_folder.exists() {
        return Err(IndexError::FolderDoesNotExist(tables_folder.to_path_buf()));
    }

    let existing_index = read_index_json(tables_index_path)?;
    if let Some(index) = &existing_index {
        info!(
            "  Found existing index with {} tables at {}",
            index.tables.len(),
            tables_index_path.display()
        );
    }
    let mut index = existing_index.unwrap_or(TablesIndex::empty());

    let vpx_files = find_vpx_files(recursive, tables_folder)?;
    info!("  Found {} tables", vpx_files.len());
    // remove files that are missing
    let removed_len = index.remove_missing(&vpx_files);
    info!("  {} missing tables have been removed", removed_len);

    let tables_with_missing_rom = index
        .tables()
        .iter()
        .filter_map(|table| {
            table
                .rom_path()
                .filter(|rom_path| !rom_path.exists())
                .map(|_| table.path.clone())
        })
        .collect::<HashSet<PathBuf>>();
    info!(
        "  {} tables will be re-indexed because their rom is missing",
        tables_with_missing_rom.len()
    );

    // find files that are missing or have been modified
    let mut vpx_files_to_index = Vec::new();
    for vpx_file in vpx_files {
        if tables_with_missing_rom.contains(&vpx_file.path)
            || force_reindex.contains(&vpx_file.path)
            || index.should_index(&vpx_file)
        {
            vpx_files_to_index.push(vpx_file);
        }
    }

    info!("  {} tables need (re)indexing.", vpx_files_to_index.len());
    let vpx_files_with_table_info = index_vpx_files(
        vpx_files_to_index,
        global_pinmame_path,
        configured_pinmame_path,
        progress,
    )?;

    // add new files to index
    index.merge(vpx_files_with_table_info);

    // write the index to a file
    write_index_json(&index, tables_index_path)?;

    Ok(index)
}

/// Indexes all vpx files in the given folder and returns the index.
/// note: The index is unordered, so the order of the tables is not guaranteed.
///
/// Arguments:
/// * `vpx_files`: the vpx files to index.
/// * `global_roms_path`: the path to the global roms folder. Eg ~/.pinmame/roms on *nix systems.
/// * `pinmame_roms_path`: the path to the local pinmame roms folder configureded in the vpinball config
/// * `progress`: lister for progress updates.
///
/// see https://github.com/francisdb/vpxtool/issues/526
pub fn index_vpx_files(
    vpx_files: Vec<PathWithMetadata>,
    global_pinmame_path: Option<&Path>,
    configured_pinmame_path: Option<&Path>,
    progress: &impl Progress,
) -> io::Result<TablesIndex> {
    let global_roms = global_pinmame_path
        .map(|pinmame_path| {
            let roms_path = pinmame_path.join("roms");
            find_roms(&roms_path)
        })
        .unwrap_or_else(|| Ok(HashMap::new()))?;

    let pinmame_roms_path = configured_pinmame_path.map(|p| p.join("roms").to_path_buf());

    let (progress_tx, progress_rx) = std::sync::mpsc::channel();

    progress.set_length(vpx_files.len() as u64);
    let index_thread = std::thread::spawn(move || {
        vpx_files
            .par_iter()
            .flat_map(|vpx_file| {
                let res = match index_vpx_file(vpx_file, pinmame_roms_path.as_deref(), &global_roms)
                {
                    Ok(indexed_table) => Some(indexed_table),
                    Err(e) => {
                        // TODO we want to return any failures instead of printing here
                        let warning =
                            format!("Not a valid vpx file {}: {}", vpx_file.path.display(), e);
                        println!("{}", warning);
                        None
                    }
                };
                // We don't care if something fails, it's just progress reporting.
                let _ = progress_tx.send(1);
                res
            })
            .collect()
    });

    let mut finished = 0;
    // The sender is automatically closed when it goes out of scope, we can be sure
    // that this does not block forever.
    for i in progress_rx {
        finished += i;
        progress.set_position(finished);
    }

    let vpx_files_with_table_info = index_thread
        .join()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    Ok(TablesIndex {
        tables: vpx_files_with_table_info,
    })
}

fn index_vpx_file(
    vpx_file_path: &PathWithMetadata,
    configured_roms_path: Option<&Path>,
    global_roms: &HashMap<String, PathBuf>,
) -> io::Result<(PathBuf, IndexedTable)> {
    let path = &vpx_file_path.path;
    let mut vpx_file = vpx::open(path)?;
    // if there's an .info.json file, we should use that instead of the info in the vpx file
    let info_file_path = path.with_extension("info.json");
    let table_info = if info_file_path.exists() {
        read_table_info_json(&info_file_path)
    } else {
        vpx_file.read_tableinfo()
    }?;
    let game_data = vpx_file.read_gamedata()?;
    let code = consider_sidecar_vbs(path, game_data)?;
    //  also this sidecar should be part of the cache key
    let game_name = extract_game_name(&code);
    let requires_pinmame = requires_pinmame(&code);
    let rom_path = find_local_rom_path(path, &game_name, configured_roms_path)?.or_else(|| {
        game_name
            .as_ref()
            .and_then(|game_name| global_roms.get(&game_name.to_lowercase()).cloned())
    });
    let b2s_path = find_b2s_path(path);
    let wheel_path = find_wheel_path(path);
    let last_modified = last_modified(path)?;
    let indexed_table_info = IndexedTableInfo::from(table_info);

    let indexed = IndexedTable {
        path: path.clone(),
        table_info: indexed_table_info,
        game_name,
        b2s_path,
        rom_path,
        local_rom_path: None,
        wheel_path,
        requires_pinmame,
        last_modified: IsoSystemTime(last_modified),
    };
    Ok((indexed.path.clone(), indexed))
}

pub fn get_romname_from_vpx(vpx_path: &Path) -> io::Result<Option<String>> {
    let mut vpx_file = vpx::open(vpx_path)?;
    let game_data = vpx_file.read_gamedata()?;
    let code = consider_sidecar_vbs(vpx_path, game_data)?;
    let game_name = extract_game_name(&code);
    let requires_pinmame = requires_pinmame(&code);
    if requires_pinmame {
        Ok(game_name)
    } else {
        Ok(None)
    }
}

fn read_table_info_json(info_file_path: &Path) -> io::Result<TableInfo> {
    let mut info_file = File::open(info_file_path)?;
    let json = serde_json::from_reader(&mut info_file).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to parse/read json {}: {}",
                info_file_path.display(),
                e
            ),
        )
    })?;
    let (table_info, _custom_info_tags) = json_to_info(json, None)?;
    Ok(table_info)
}

/// Visual pinball always falls back to the [vpx_folder]/pinmame/roms folder,
/// even if the PinMAMEPath folder is configured in the vpinball config.
fn find_local_rom_path(
    vpx_file_path: &Path,
    game_name: &Option<String>,
    configured_roms_path: Option<&Path>,
) -> io::Result<Option<PathBuf>> {
    if let Some(game_name) = game_name {
        let rom_file_name = format!("{}.zip", game_name.to_lowercase());

        let pinmame_roms_path = if let Some(configured_roms_path) = configured_roms_path {
            let configured_roms_path = if configured_roms_path.is_relative() {
                vpx_file_path.parent().unwrap().join(configured_roms_path)
            } else {
                configured_roms_path.to_owned()
            };
            if configured_roms_path.exists() {
                configured_roms_path
            } else {
                vpx_file_path.parent().unwrap().join("pinmame").join("roms")
            }
        } else {
            vpx_file_path.parent().unwrap().join("pinmame").join("roms")
        };

        let rom_path = pinmame_roms_path.join(rom_file_name);
        return if rom_path.exists() {
            Ok(Some(rom_path.canonicalize()?))
        } else {
            Ok(None)
        };
    };
    Ok(None)
}

fn find_b2s_path(vpx_file_path: &Path) -> Option<PathBuf> {
    let b2s_file_name = format!(
        "{}.directb2s",
        vpx_file_path.file_stem().unwrap().to_string_lossy()
    );
    let b2s_path = vpx_file_path.parent().unwrap().join(b2s_file_name);
    if b2s_path.exists() {
        Some(b2s_path)
    } else {
        None
    }
}

/// Tries to find a wheel image for the given vpx file.
/// 2 locations are tried:
/// * ../wheels/<vpx_file_name>.png
/// * <vpx_file_name>.wheel.png
fn find_wheel_path(vpx_file_path: &Path) -> Option<PathBuf> {
    let wheel_file_name = format!(
        "wheels/{}.png",
        vpx_file_path.file_stem().unwrap().to_string_lossy()
    );
    let wheel_path = vpx_file_path.parent().unwrap().join(wheel_file_name);
    if wheel_path.exists() {
        return Some(wheel_path);
    }
    let wheel_path = vpx_file_path.with_extension("wheel.png");
    if wheel_path.exists() {
        return Some(wheel_path);
    }
    None
}

/// If there is a file with the same name and extension .vbs we pick that code
/// instead of the code in the vpx file.
///
/// TODO if this file changes the index entry is currently not invalidated
fn consider_sidecar_vbs(path: &Path, game_data: GameData) -> io::Result<String> {
    let vbs_path = path.with_extension("vbs");
    let code = if vbs_path.exists() {
        let mut vbs_file = File::open(vbs_path)?;
        let mut code = String::new();
        vbs_file.read_to_string(&mut code)?;
        code
    } else {
        game_data.code.string
    };
    Ok(code)
}

fn last_modified(path: &Path) -> io::Result<SystemTime> {
    let metadata: Metadata = path.metadata()?;
    metadata.modified()
}

pub fn write_index_json(indexed_tables: &TablesIndex, json_path: &Path) -> io::Result<()> {
    let json_file = File::create(json_path)?;
    let indexed_tables_json: TablesIndexJson = indexed_tables.into();
    serde_json::to_writer_pretty(json_file, &indexed_tables_json)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub fn read_index_json(json_path: &Path) -> io::Result<Option<TablesIndex>> {
    if !json_path.exists() {
        return Ok(None);
    }
    let json_file = File::open(json_path)?;
    match serde_json::from_reader::<_, TablesIndexJson>(json_file) {
        Ok(indexed_tables_json) => {
            let indexed_tables: TablesIndex = indexed_tables_json.into();
            Ok(Some(indexed_tables))
        }
        Err(e) => {
            println!(
                "Failed to parse index file, ignoring existing index. ({})",
                e
            );
            Ok(None)
        }
    }
}

fn extract_game_name<S: AsRef<str>>(code: S) -> Option<String> {
    // TODO can we find a first match through an option?
    // needs to be all lowercase to match with (?i) case insensitive
    const LINE_WITH_CGAMENAME_RE: &str =
        r#"(?i)(?:.*?)*cgamename\s*=\s*\"([^"\\]*(?:\\.[^"\\]*)*)\""#;
    const LINE_WITH_DOT_GAMENAME_RE: &str =
        r#"(?i)(?:.*?)\.gamename\s*=\s*\"([^"\\]*(?:\\.[^"\\]*)*)\""#;
    let cgamename_re = regex::Regex::new(LINE_WITH_CGAMENAME_RE).unwrap();
    let dot_gamename_re = regex::Regex::new(LINE_WITH_DOT_GAMENAME_RE).unwrap();
    let unified = unify_line_endings(code.as_ref());
    unified
        .lines()
        // skip rows that start with ' or whitespace followed by '
        .filter(|line| !line.trim().starts_with('\''))
        .filter(|line| {
            let lower: String = line.to_owned().to_lowercase().trim().to_string();
            lower.contains("cgamename") || lower.contains(".gamename")
        })
        .flat_map(|line| {
            let caps = cgamename_re
                .captures(line)
                .or(dot_gamename_re.captures(line))?;
            let first = caps.get(1)?;
            Some(first.as_str().to_string())
        })
        .next()
}

fn requires_pinmame<S: AsRef<str>>(code: S) -> bool {
    let unified = unify_line_endings(code.as_ref());
    let lower = unified.to_lowercase();
    const RE: &str = r#"sub\s*loadvpm"#;
    let re = regex::Regex::new(RE).unwrap();
    lower
        .lines()
        .filter(|line| !line.trim().starts_with('\''))
        .any(|line| line.contains("loadvpm") && !re.is_match(line))
}

/// Some scripts contain only CR as line separator. Eg "Monte Carlo (Premier 1987) (10.7) 1.6.vpx"
/// Therefore we replace first all CRLF and then all leftover CR with LF
fn unify_line_endings(code: &str) -> String {
    code.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::io::Write;
    use testdir::testdir;
    use vpin::vpx;

    #[test]
    fn test_index_vpx_files() -> io::Result<()> {
        // Test setup looks like this:
        // test_dir/
        // ├── test.vpx
        // ├── test2.vpx
        // ├── subdir
        // │   └── test3.vpx
        // ├── test3.vpx
        // ├── __MACOSX/
        // │   └── ignored.vpx
        // ├── .git/
        // │   └── ignored2.vpx
        // ├── pinmame/
        // │   └── roms/
        // │       └── testgamename.zip
        // global_pinmame/
        // ├── roms/
        // │   └── testgamename2.zip
        let global_pinmame_dir = testdir!().join("global_pinmame");
        fs::create_dir(&global_pinmame_dir)?;
        let global_rom_dir = global_pinmame_dir.join("roms");
        fs::create_dir(&global_rom_dir)?;
        let tables_dir = testdir!().join("tables");
        fs::create_dir(&tables_dir)?;
        let temp_dir = testdir!().join("temp");
        fs::create_dir(&temp_dir)?;
        // the next two folders should be ignored
        let macosx = tables_dir.join("__MACOSX");
        fs::create_dir(&macosx)?;
        File::create(macosx.join("ignored.vpx"))?;
        let git = tables_dir.join(".git");
        fs::create_dir(&git)?;
        File::create(git.join("ignored2.vpx"))?;
        fs::create_dir(tables_dir.join("subdir"))?;
        // actual vpx files to index
        let vpx_1_path = tables_dir.join("test.vpx");
        let vpx_2_path = tables_dir.join("test2.vpx");
        let vpx_3_path = tables_dir.join("subdir").join("test3.vpx");

        vpx::new_minimal_vpx(&vpx_1_path)?;
        let script1 = test_script(&temp_dir, "testgamename")?;
        vpx::importvbs(&vpx_1_path, Some(script1))?;
        // local rom
        let mut rom1_path_local = tables_dir
            .join("pinmame")
            .join("roms")
            .join("testgamename.zip");
        // recursively create dir
        fs::create_dir_all(rom1_path_local.parent().unwrap())?;
        File::create(&rom1_path_local)?;
        // this canonicalize makes a strange dir starting with //?/
        rom1_path_local = rom1_path_local.canonicalize()?;

        vpx::new_minimal_vpx(&vpx_2_path)?;
        let script2 = test_script(&temp_dir, "testgamename2")?;
        vpx::importvbs(&vpx_2_path, Some(script2))?;
        // global rom
        let rom2_path_global = global_rom_dir.join("testgamename2.zip");
        File::create(&rom2_path_global)?;

        vpx::new_minimal_vpx(&vpx_3_path)?;
        // no rom

        // let output = std::process::Command::new("tree")
        //     .arg(&tables_dir)
        //     .output()
        //     .expect("failed to execute process");
        // let output_str = String::from_utf8_lossy(&output.stdout);
        // println!("test_dir:\n{}", output_str);
        //
        // let output = std::process::Command::new("tree")
        //     .arg(&global_pinmame_dir)
        //     .output()
        //     .expect("failed to execute process");
        // let output_str = String::from_utf8_lossy(&output.stdout);
        // println!("global_pinmame_dir:\n{}", output_str);

        let vpx_files = find_vpx_files(true, &tables_dir)?;
        assert_eq!(vpx_files.len(), 3);
        let global_roms = find_roms(&global_rom_dir)?;
        assert_eq!(global_roms.len(), 1);
        let configured_roms_path = Some(PathBuf::from("./"));
        let indexed_tables = index_vpx_files(
            vpx_files,
            Some(&global_pinmame_dir),
            configured_roms_path.as_deref(),
            &VoidProgress,
        )?;
        assert_eq!(indexed_tables.tables.len(), 3);
        let table1 = indexed_tables
            .tables
            .get(&vpx_1_path)
            .expect("table1 not found");
        let table2 = indexed_tables
            .tables
            .get(&vpx_2_path)
            .expect("table2 not found");
        let table3 = indexed_tables
            .tables
            .get(&vpx_3_path)
            .expect("table3 not found");
        assert_eq!(table1.path, vpx_1_path);
        assert_eq!(table2.path, vpx_2_path);
        assert_eq!(table3.path, vpx_3_path);
        assert_eq!(table1.rom_path, Some(rom1_path_local.clone()));
        assert_eq!(table2.rom_path, Some(rom2_path_global.clone()));
        assert_eq!(table3.rom_path, None);
        Ok(())
    }

    fn test_script(temp_dir: &Path, game_name: &str) -> io::Result<PathBuf> {
        // write simple script in tempdir
        let script = format!(
            r#"
    Const cGameName = "{}"
    Sub LoadVPM
    "#,
            game_name
        );
        let script_path = temp_dir.join(game_name).with_extension("vbs");
        let mut script_file = File::create(&script_path)?;
        script_file.write_all(script.as_bytes())?;
        Ok(script_path)
    }

    #[test]
    fn test_write_read_empty_array() -> io::Result<()> {
        let index = TablesIndex::empty();
        let test_dir = testdir!();
        let index_path = test_dir.join("test.json");
        // write empty json array using serde_json
        let json_file = File::create(&index_path)?;
        let json_object = json!({
            "tables": []
        });
        serde_json::to_writer_pretty(json_file, &json_object)?;
        let read = read_index_json(&index_path)?;
        assert_eq!(read, Some(index));
        Ok(())
    }

    #[test]
    fn test_write_read_invalid_file() -> io::Result<()> {
        let test_dir = testdir!();
        let index_path = test_dir.join("test.json");
        // write empty json array using serde_json
        let json_file = File::create(&index_path)?;
        // write garbage to file
        serde_json::to_writer_pretty(json_file, &"garbage")?;
        let read = read_index_json(&index_path)?;
        assert_eq!(read, None);
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
        let mut index = TablesIndex::empty();
        index.insert(IndexedTable {
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
            game_name: Some("testrom".to_string()),
            b2s_path: Some(PathBuf::from("test.b2s")),
            rom_path: Some(PathBuf::from("testrom.zip")),
            local_rom_path: None,
            wheel_path: Some(PathBuf::from("test.png")),
            requires_pinmame: true,
            last_modified: IsoSystemTime(SystemTime::UNIX_EPOCH),
        });
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

    #[test]
    fn test_extract_game_name() {
        let code = r#"
  Dim tableheight: tableheight = Table1.height

  Const cGameName="godzilla",UseSolenoids=2,UseLamps=1,UseGI=0, SCoin=""
  Const UseVPMModSol = True

"#
        .to_string();
        let game_name = extract_game_name(code);
        assert_eq!(game_name, Some("godzilla".to_string()));
    }

    #[test]
    fn test_extract_game_name_commented() {
        let code = r#"
  'Const cGameName = "commented"
  Const cGameName = "actual"
"#
        .to_string();
        let game_name = extract_game_name(code);
        assert_eq!(game_name, Some("actual".to_string()));
    }

    #[test]
    fn test_extract_game_name_spaced() {
        let code = r#"
  Const cGameName = "gg"
"#
        .to_string();
        let game_name = extract_game_name(code);
        assert_eq!(game_name, Some("gg".to_string()));
    }

    #[test]
    fn test_extract_game_name_casing() {
        let code = r#"
  const cgamenamE = "othercase"
"#
        .to_string();
        let game_name = extract_game_name(code);
        assert_eq!(game_name, Some("othercase".to_string()));
    }

    #[test]
    fn test_extract_game_name_uppercase_name() {
        let code = r#"
Const cGameName = "BOOM"
"#
        .to_string();
        let game_name = extract_game_name(code);
        assert_eq!(game_name, Some("BOOM".to_string()));
    }

    #[test]
    fn test_extract_game_name_with_underscore() {
        let code = r#"
Const cGameName="simp_a27",UseSolenoids=1,UseLamps=0,UseGI=0,SSolenoidOn="SolOn",SSolenoidOff="SolOff", SCoin="coin"

LoadVPM "01000200", "DE.VBS", 3.36
"#
            .to_string();
        let game_name = extract_game_name(code);
        assert_eq!(game_name, Some("simp_a27".to_string()));
    }

    #[test]
    fn test_extract_game_name_multidef_end() {
        let code = r#"
Const UseSolenoids=2,UseLamps=0,UseSync=1,UseGI=0,SCoin="coin",cGameName="barbwire"
"#
        .to_string();
        let game_name = extract_game_name(code);
        assert_eq!(game_name, Some("barbwire".to_string()));
    }

    /// https://github.com/francisdb/vpxtool/issues/203
    #[test]
    fn test_extract_game_name_in_controller() {
        let code = r#"
        Sub Gorgar_Init
    LoadLUT
	On Error Resume Next
	With Controller
	.GameName="grgar_l1"
        "#;
        let game_name = extract_game_name(code);
        assert_eq!(game_name, Some("grgar_l1".to_string()));
    }

    #[test]
    fn test_extract_game_name_2_line_dim() {
        let code = r#"
Dim cGameName
cGameName = "abv106"
"#
        .to_string();
        let game_name = extract_game_name(code);
        assert_eq!(game_name, Some("abv106".to_string()));
    }

    #[test]
    fn test_requires_pinmame() {
        let code = r#"#
  LoadVPM "01210000", "sys80.VBS", 3.1
"#
        .to_string();
        assert!(requires_pinmame(code));
    }

    #[test]
    fn test_requires_pinmame_other_casing() {
        let code = r#"
  loadVpm "01210000", \"sys80.VBS\", 3.1
"#
        .to_string();
        assert!(requires_pinmame(code));
    }

    #[test]
    fn test_requires_pinmame_not() {
        let code = r#"
Const cGameName = "GTB_4Square_1971"
"#
        .to_string();
        assert!(!requires_pinmame(code));
    }

    #[test]
    fn test_requires_pinmame_with_same_sub() {
        // got this from blood machines
        let code = r#"
Sub LoadVPM(VPMver, VBSfile, VBSver)
	LoadVBSFiles VPMver, VBSfile, VBSver
	LoadController("VPM")
End Sub
"#
        .to_string();
        assert!(!requires_pinmame(code));
    }

    #[test]
    fn test_requires_pinmame_comment() {
        // got this from blood machines
        let code = r#"
' VRRoom set based on RenderingMode
' Internal DMD in Desktop Mode, using a textbox (must be called before LoadVPM)
Dim UseVPMDMD, VRRoom, DesktopMode
If RenderingMode = 2 Then VRRoom = VRRoomChoice Else VRRoom = 0
"#
        .to_string();
        assert!(!requires_pinmame(code));
    }

    #[test]
    fn test_requires_pinmame_comment_and_used() {
        // got this from blood machines
        let code = r#"
Const SCoin="coin3",cCredits=""

LoadVPM "01210000","sys80.vbs",3.10

'Sub LoadVPM(VPMver, VBSfile, VBSver)
'	On Error Resume Next
"#
        .to_string();
        assert!(requires_pinmame(code));
    }

    #[test]
    fn test_requires_pinmame_cr_only_lines_and_commented_sub_loadvpm() {
        // This code was taken from "Monte Carlo (Premier 1987) (10.7) 1.6.vpx"
        let code =
            "LoadVPM \"01210000\", \"sys80.VBS\", 3.1\r\r'Sub LoadVPM(VPMver, VBSfile, VBSver)\r";
        assert!(requires_pinmame(code));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_find_local_rom_path_relative_linux() {
        // On Batocera the PinMAMEPath is configured as ./
        // That gives us a roms path of ./roms
        let test_table_dir = testdir!();
        let vpx_path = test_table_dir.join("test.vpx");
        let expected_rom_path = test_table_dir.join("roms").join("testgamename.zip");
        fs::create_dir_all(expected_rom_path.parent().unwrap()).unwrap();
        File::create(&expected_rom_path).unwrap();

        let local_rom = find_local_rom_path(
            &vpx_path,
            &Some("testgamename".to_string()),
            Some(&PathBuf::from("./roms")),
        )
        .unwrap();
        assert_eq!(local_rom, Some(expected_rom_path));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_find_local_rom_path_relative_not_found_linux() {
        // On Batocera the PinMAMEPath is configured as ./
        // That gives us a roms path of ./roms
        let test_table_dir = testdir!();
        let vpx_path = test_table_dir.join("test.vpx");
        let expected_rom_path = test_table_dir.join("roms").join("testgamename.zip");
        fs::create_dir_all(expected_rom_path.parent().unwrap()).unwrap();

        let local_rom = find_local_rom_path(
            &vpx_path,
            &Some("testgamename".to_string()),
            Some(&PathBuf::from("./roms")),
        )
        .unwrap();
        assert_eq!(local_rom, None);
    }
}
