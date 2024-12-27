use chrono::{DateTime, Utc};
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

use colored::Colorize;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use vpin::vpx::jsonmodel::json_to_info;
use walkdir::{DirEntry, FilterEntry, IntoIter, WalkDir};

use crate::tableinfo::TableInfo;
use crate::vpx;
use crate::vpx::gamedata::GameData;

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
    local_rom_path: Option<PathBuf>,
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

    pub(crate) fn len(&self) -> usize {
        self.tables.len()
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
pub fn find_roms<P: AsRef<Path>>(rom_path: P) -> io::Result<HashMap<String, PathBuf>> {
    if !rom_path.as_ref().exists() {
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

pub fn find_vpx_files<P: AsRef<Path>>(
    recursive: bool,
    tables_path: P,
) -> io::Result<Vec<PathWithMetadata>> {
    if recursive {
        let mut vpx_files = Vec::new();
        let mut entries = walk_dir_filtered(&tables_path);
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
fn walk_dir_filtered<P: AsRef<Path>>(
    tables_path: &P,
) -> FilterEntry<IntoIter, fn(&DirEntry) -> bool> {
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

impl From<io::Error> for IndexError {
    fn from(e: io::Error) -> Self {
        IndexError::IoError(e)
    }
}

/// Indexes all vpx files in the given folder and writes the index to a file.
/// Returns the index.
/// If the index file already exists, it will be read and updated.
/// If the index file does not exist, it will be created.
pub fn index_folder<P: AsRef<Path>>(
    recursive: bool,
    tables_folder: P,
    tables_index_path: P,
    global_roms_path: Option<P>,
    progress: &impl Progress,
    force_reindex: Vec<PathBuf>,
) -> Result<TablesIndex, IndexError> {
    let global_roms = global_roms_path
        .map(find_roms)
        .unwrap_or_else(|| Ok(HashMap::new()))?;
    println!("Indexing {}", tables_folder.as_ref().display());

    if !tables_folder.as_ref().exists() {
        return Err(IndexError::FolderDoesNotExist(
            tables_folder.as_ref().to_path_buf(),
        ));
    }

    let existing_index = read_index_json(&tables_index_path)?;
    if let Some(index) = &existing_index {
        println!(
            "  Found existing index with {} tables at {}",
            index.tables.len(),
            tables_index_path.as_ref().display()
        );
    }
    let mut index = existing_index.unwrap_or(TablesIndex::empty());

    let vpx_files = find_vpx_files(recursive, tables_folder)?;
    println!("  Found {} tables", vpx_files.len());
    // remove files that are missing
    let removed_len = index.remove_missing(&vpx_files);
    println!("  {} missing tables have been removed", removed_len);

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
    println!(
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

    println!("  {} tables need (re)indexing.", vpx_files_to_index.len());
    let vpx_files_with_table_info = index_vpx_files(&vpx_files_to_index, &global_roms, progress);

    // add new files to index
    index.merge(vpx_files_with_table_info);

    // write the index to a file
    write_index_json(&index, tables_index_path)?;

    Ok(index)
}

pub fn index_vpx_files(
    vpx_files: &[PathWithMetadata],
    global_roms: &HashMap<String, PathBuf>,
    progress: &impl Progress,
) -> TablesIndex {
    // TODO tried using rayon here but it's not faster and uses up all cpu
    // use rayon::prelude::*;
    // .par_iter() instead of .iter()
    progress.set_length(vpx_files.len() as u64);

    let vpx_files_with_table_info: HashMap<PathBuf, IndexedTable> = vpx_files
        .iter()
        .enumerate()
        .flat_map(|(i, vpx_file)| {
            let optional = match index_vpx_file(vpx_file, global_roms) {
                Ok(indexed_table) => Some(indexed_table),
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
    // vpx_files_with_table_info.sort_by(|a, b| table_name_compare(a, b));
    TablesIndex {
        tables: vpx_files_with_table_info,
    }
}

fn index_vpx_file(
    vpx_file_path: &PathWithMetadata,
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
    let rom_path = find_local_rom_path(vpx_file_path, &game_name).or_else(|| {
        game_name
            .as_ref()
            .and_then(|game_name| global_roms.get(&game_name.to_lowercase()).cloned())
    });
    let b2s_path = find_b2s_path(vpx_file_path);

    let last_modified = last_modified(path).unwrap();
    let indexed_table_info = IndexedTableInfo::from(table_info);

    let indexed = IndexedTable {
        path: path.clone(),
        table_info: indexed_table_info,
        game_name,
        b2s_path,
        rom_path,
        local_rom_path: None,
        requires_pinmame,
        last_modified: IsoSystemTime(last_modified),
    };
    Ok((indexed.path.clone(), indexed))
}

pub(crate) fn get_romname_from_vpx(vpx_path: &Path) -> io::Result<Option<String>> {
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

fn find_local_rom_path(
    vpx_file_path: &PathWithMetadata,
    game_name: &Option<String>,
) -> Option<PathBuf> {
    game_name.as_ref().and_then(|game_name| {
        let rom_file_name = format!("{}.zip", game_name.to_lowercase());
        let rom_path = vpx_file_path
            .path
            .parent()
            .unwrap()
            .join("pinmame")
            .join("roms")
            .join(rom_file_name);
        if rom_path.exists() {
            Some(rom_path)
        } else {
            None
        }
    })
}

fn find_b2s_path(vpx_file_path: &PathWithMetadata) -> Option<PathBuf> {
    let b2s_file_name = format!(
        "{}.directb2s",
        vpx_file_path.path.file_stem().unwrap().to_string_lossy()
    );
    let b2s_path = vpx_file_path.path.parent().unwrap().join(b2s_file_name);
    if b2s_path.exists() {
        Some(b2s_path)
    } else {
        None
    }
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

fn last_modified<P: AsRef<Path>>(path: P) -> io::Result<SystemTime> {
    let metadata: Metadata = path.as_ref().metadata()?;
    metadata.modified()
}

pub fn write_index_json<P: AsRef<Path>>(
    indexed_tables: &TablesIndex,
    json_path: P,
) -> io::Result<()> {
    let json_file = File::create(json_path)?;
    let indexed_tables_json: TablesIndexJson = indexed_tables.into();
    serde_json::to_writer_pretty(json_file, &indexed_tables_json)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub fn read_index_json<P: AsRef<Path>>(json_path: P) -> io::Result<Option<TablesIndex>> {
    if !json_path.as_ref().exists() {
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
    use crate::vpx;
    use serde_json::json;
    use std::io::Write;
    use testdir::testdir;

    struct VoidProgress;
    impl Progress for VoidProgress {
        fn set_length(&self, _len: u64) {}
        fn set_position(&self, _i: u64) {}
        fn finish_and_clear(&self) {}
    }

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
        // global_rom_dir/
        // ├── testgamename2.zip
        let global_rom_dir = testdir!();
        let test_dir = testdir!();
        let temp_dir = testdir!();
        // the next two folders should be ignored
        let macosx = test_dir.join("__MACOSX");
        fs::create_dir(&macosx)?;
        File::create(macosx.join("ignored.vpx"))?;
        let git = test_dir.join(".git");
        fs::create_dir(&git)?;
        File::create(git.join("ignored2.vpx"))?;
        fs::create_dir(test_dir.join("subdir"))?;
        // actual vpx files to index
        let vpx_1_path = test_dir.join("test.vpx");
        let vpx_2_path = test_dir.join("test2.vpx");
        let vpx_3_path = test_dir.join("subdir").join("test3.vpx");

        vpx::new_minimal_vpx(&vpx_1_path)?;
        let script1 = test_script(&temp_dir, "testgamename")?;
        vpx::importvbs(&vpx_1_path, Some(script1))?;
        // local rom
        let rom1_path_local = test_dir
            .join("pinmame")
            .join("roms")
            .join("testgamename.zip");
        // recursively create dir
        fs::create_dir_all(rom1_path_local.parent().unwrap())?;
        File::create(&rom1_path_local)?;

        vpx::new_minimal_vpx(&vpx_2_path)?;
        let script2 = test_script(&temp_dir, "testgamename2")?;
        vpx::importvbs(&vpx_2_path, Some(script2))?;
        // global rom
        let rom2_path_global = global_rom_dir.join("testgamename2.zip");
        File::create(&rom2_path_global)?;

        vpx::new_minimal_vpx(&vpx_3_path)?;
        // no rom

        let vpx_files = find_vpx_files(true, &test_dir)?;
        assert_eq!(vpx_files.len(), 3);
        let global_roms = find_roms(&global_rom_dir)?;
        assert_eq!(global_roms.len(), 1);
        let indexed_tables = index_vpx_files(&vpx_files, &global_roms, &VoidProgress);
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
        let read = read_index_json(index_path)?;
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
}
