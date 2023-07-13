use std::io::{self, Read, Seek, Write};
use std::path::MAIN_SEPARATOR_STR;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

use byteorder::{LittleEndian, WriteBytesExt};
use cfb::CompoundFile;

use gamedata::Record;
use nom::number::complete::le_u32;
use nom::IResult;

use md2::{Digest, Md2};

use crate::vpx::biff::BiffReader;

use self::tableinfo::{write_tableinfo, TableInfo};

pub mod biff;
pub mod collection;
pub mod font;
pub mod gamedata;
pub mod image;
pub mod sound;
pub mod tableinfo;

pub enum ExtractResult {
    Extracted(PathBuf),
    Existed(PathBuf),
}

pub enum VerifyResult {
    Ok(PathBuf),
    Failed(PathBuf, String),
}

pub fn new_minimal_vpx<P: AsRef<Path>>(vpx_file_path: P) -> std::io::Result<()> {
    let file = File::create(vpx_file_path)?;
    let mut comp = CompoundFile::create(file)?;
    write_minimal_vpx(&mut comp)
}

pub fn write_minimal_vpx<F: Read + Write + Seek>(
    comp: &mut CompoundFile<F>,
) -> std::io::Result<()> {
    let table_info = TableInfo::new();
    write_tableinfo(comp, &table_info)?;
    create_game_storage(comp)?;
    write_version(comp, 1072)?;
    write_game_data(comp, &[])?;
    // to be more efficient we could generate the mac while writing the different parts
    let mac = generate_mac(comp)?;
    write_mac(comp, &mac)
}

fn create_game_storage<F: Read + Write + Seek>(comp: &mut CompoundFile<F>) -> io::Result<()> {
    let game_stg_path = Path::new(MAIN_SEPARATOR_STR).join("GameStg");
    comp.create_storage(&game_stg_path)
}

pub fn extractvbs(
    vpx_file_path: &PathBuf,
    overwrite: bool,
    extension: Option<&str>,
) -> ExtractResult {
    let script_path = match extension {
        Some(ext) => path_for(vpx_file_path, ext),
        None => vbs_path_for(vpx_file_path),
    };

    if !script_path.exists() || (script_path.exists() && overwrite) {
        let mut comp = cfb::open(vpx_file_path).unwrap();
        let _version = read_version(&mut comp);
        let records = read_gamedata(&mut comp).unwrap();
        extract_script(&records, &script_path);
        ExtractResult::Extracted(script_path)
    } else {
        ExtractResult::Existed(script_path)
    }
}

pub fn verify(vpx_file_path: &PathBuf) -> VerifyResult {
    let mut comp = match cfb::open(vpx_file_path) {
        Ok(comp) => comp,
        Err(e) => {
            return VerifyResult::Failed(
                vpx_file_path.clone(),
                format!("Failed to open VPX file {}: {}", vpx_file_path.display(), e),
            )
        }
    };
    let mac = read_mac(&mut comp).unwrap();
    let generated_mac = generate_mac(&mut comp).unwrap();
    if mac == generated_mac {
        VerifyResult::Ok(vpx_file_path.clone())
    } else {
        VerifyResult::Failed(
            vpx_file_path.clone(),
            format!("MAC mismatch: {:?} != {:?}", mac, generated_mac),
        )
    }
}

pub fn vbs_path_for(vpx_file_path: &PathBuf) -> PathBuf {
    path_for(vpx_file_path, "vbs")
}

fn path_for(vpx_file_path: &PathBuf, extension: &str) -> PathBuf {
    PathBuf::from(vpx_file_path).with_extension(extension)
}

// Read version
// https://github.com/vbousquet/vpx_lightmapper/blob/331a8576bb7b86668a023b304e7dd04261487106/addons/vpx_lightmapper/vlm_import.py#L328
pub fn read_version<F: Read + Write + Seek>(
    comp: &mut cfb::CompoundFile<F>,
) -> std::io::Result<u32> {
    let mut file_version = Vec::new();
    let version_path = Path::new(MAIN_SEPARATOR_STR)
        .join("GameStg")
        .join("Version");
    let mut stream = comp.open_stream(version_path)?;
    stream.read_to_end(&mut file_version)?;

    fn read_version(input: &[u8]) -> IResult<&[u8], u32> {
        le_u32(input)
    }

    let (_, version) = read_version(&file_version[..]).unwrap();

    // let version_float = (version as f32)/100f32;
    // println!("VPX file version: {}", version);
    Ok(version)
}

pub fn write_version<F: Read + Write + Seek>(
    comp: &mut CompoundFile<F>,
    version: u32,
) -> std::io::Result<()> {
    // we expect GameStg to exist
    let version_path = Path::new(MAIN_SEPARATOR_STR)
        .join("GameStg")
        .join("Version");
    let mut stream = comp.create_stream(version_path)?;
    stream.write_u32::<LittleEndian>(version)
}

pub fn read_mac<F: Read + Write + Seek>(
    comp: &mut cfb::CompoundFile<F>,
) -> std::io::Result<Vec<u8>> {
    let mac_path = Path::new(MAIN_SEPARATOR_STR).join("GameStg").join("MAC");
    if !comp.exists(&mac_path) {
        // fail
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "MAC stream not found",
        ));
    }
    let mut mac_stream = comp.open_stream(mac_path)?;
    let mut mac = Vec::new();
    mac_stream.read_to_end(&mut mac)?;
    Ok(mac)
}

pub fn write_mac<F: Read + Write + Seek>(
    comp: &mut CompoundFile<F>,
    mac: &[u8],
) -> std::io::Result<()> {
    let mac_path = Path::new(MAIN_SEPARATOR_STR).join("GameStg").join("MAC");
    let mut mac_stream = comp.create_stream(mac_path)?;
    mac_stream.write_all(mac)
}

#[derive(Clone, Debug)]
enum FileType {
    UnstructuredBytes,
    Biff,
}

#[derive(Debug)]
struct FileStructureItem {
    path: PathBuf,
    file_type: FileType,
    hashed: bool,
}
// contructor with default values
impl FileStructureItem {
    fn new(path: &str, file_type: FileType, hashed: bool) -> Self {
        FileStructureItem {
            path: PathBuf::from(path),
            file_type,
            hashed,
        }
    }
}

pub fn generate_mac<F: Read + Seek>(comp: &mut CompoundFile<F>) -> Result<Vec<u8>, io::Error> {
    // Regarding mac generation, see
    //  https://github.com/freezy/VisualPinball.Engine/blob/ec1e9765cd4832c134e889d6e6d03320bc404bd5/VisualPinball.Engine/VPT/Table/TableWriter.cs#L42
    //  https://github.com/vbousquet/vpx_lightmapper/blob/ca5fddd4c2a0fbe817fd546c5f4db609f9d0da9f/addons/vpx_lightmapper/vlm_export.py#L906-L913
    //  https://github.com/vpinball/vpinball/blob/d9d22a5923ad5a9902a27fae296bc6b2e9ed95ca/pintable.cpp#L2634-L2667
    //  ordering of writes is important co come up with the correct hash

    fn item_path(path: &Path, index: i32) -> PathBuf {
        path.with_file_name(format!(
            "{}{}",
            path.file_name().unwrap().to_string_lossy(),
            index
        ))
    }

    fn append_structure<F: Seek + Read>(
        file_structure: &mut Vec<FileStructureItem>,
        comp: &mut CompoundFile<F>,
        src_path: &str,
        file_type: FileType,
        hashed: bool,
    ) {
        let mut index = 0;
        let path = PathBuf::from(src_path);
        while comp.exists(item_path(&path, index)) {
            file_structure.push(FileStructureItem {
                path: item_path(&path, index),
                file_type: file_type.clone(),
                hashed,
            });
            index += 1;
        }
    }

    use FileType::*;

    // above pythin code converted to rust
    let mut file_structure: Vec<FileStructureItem> = vec![
        FileStructureItem::new("GameStg/Version", UnstructuredBytes, true),
        FileStructureItem::new("TableInfo/TableName", UnstructuredBytes, true),
        FileStructureItem::new("TableInfo/AuthorName", UnstructuredBytes, true),
        FileStructureItem::new("TableInfo/TableVersion", UnstructuredBytes, true),
        FileStructureItem::new("TableInfo/ReleaseDate", UnstructuredBytes, true),
        FileStructureItem::new("TableInfo/AuthorEmail", UnstructuredBytes, true),
        FileStructureItem::new("TableInfo/AuthorWebSite", UnstructuredBytes, true),
        FileStructureItem::new("TableInfo/TableBlurb", UnstructuredBytes, true),
        FileStructureItem::new("TableInfo/TableDescription", UnstructuredBytes, true),
        FileStructureItem::new("TableInfo/TableRules", UnstructuredBytes, true),
        FileStructureItem::new("TableInfo/TableSaveDate", UnstructuredBytes, false),
        FileStructureItem::new("TableInfo/TableSaveRev", UnstructuredBytes, false),
        FileStructureItem::new("TableInfo/Screenshot", UnstructuredBytes, true),
        FileStructureItem::new("GameStg/CustomInfoTags", Biff, true), // custom info tags must be hashed just after this stream
        FileStructureItem::new("GameStg/GameData", Biff, true),
    ];
    // //append_structure(&mut file_structure, comp, "GameStg/GameItem", Biff, false);
    //append_structure(&mut file_structure, comp, "GameStg/Sound", Biff, false);
    // //append_structure(&mut file_structure, comp, "GameStg/Image", Biff, false);
    //append_structure(&mut file_structure, comp, "GameStg/Font", Biff, false);
    append_structure(&mut file_structure, comp, "GameStg/Collection", Biff, true);

    let mut hasher = Md2::new();

    // header is always there.
    hasher.update(b"Visual Pinball");

    for item in file_structure {
        if !item.hashed {
            continue;
        }
        if !comp.exists(&item.path) {
            continue;
        }
        match item.file_type {
            FileType::UnstructuredBytes => {
                let bytes = read_bytes_at(&item.path, comp)?;
                hasher.update(&bytes);
            }
            FileType::Biff => {
                // println!("reading biff: {:?}", item.path);
                let bytes = read_bytes_at(&item.path, comp)?;
                let mut biff = BiffReader::new(&bytes);

                loop {
                    if biff.is_eof() {
                        break;
                    }
                    biff.next(biff::WARN);
                    // println!("reading biff: {:?} {}", item.path, biff.tag());
                    let tag = biff.tag();
                    let tag_str = tag.as_str();
                    match tag_str {
                        "CODE" => {
                            //  For some reason, the code length info is not hashed, just the tag and code string
                            hasher.update(b"CODE");
                            // code is a special case, it indicates a length of 4 (only the tag)
                            // so already 0 bytes remaining
                            let code_length = biff.get_u32_no_remaining_update();
                            let code = biff.get_no_remaining_update(code_length as usize);
                            hasher.update(code);
                        }
                        _other => {
                            // Biff tags and data are hashed but not their size
                            hasher.update(biff.get_record_data(true));
                        }
                    }
                }
            }
        }

        if item.path.ends_with("CustomInfoTags") {
            let bytes = read_bytes_at(&item.path, comp)?;
            let mut biff = BiffReader::new(&bytes);

            loop {
                if biff.is_eof() {
                    break;
                }
                biff.next(biff::WARN);
                if biff.tag() == "CUST" {
                    let cust_name = biff.get_string();
                    //println!("Hashing custom information block {}", cust_name);
                    let path = format!("TableInfo/{}", cust_name);
                    if comp.exists(&path) {
                        let data = read_bytes_at(&path, comp)?;
                        hasher.update(&data);
                    }
                } else {
                    biff.skip_tag();
                }
            }
        }
    }
    let result = hasher.finalize();
    Ok(result.to_vec())
}

// TODO this is not very efficient as we copy the bytes around a lot
fn read_bytes_at<F: Read + Seek, P: AsRef<Path>>(
    path: P,
    comp: &mut CompoundFile<F>,
) -> Result<Vec<u8>, io::Error> {
    // println!("reading bytes at: {:?}", path.as_ref());
    let mut bytes = Vec::new();
    let mut stream = comp.open_stream(path)?;
    stream.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn extract_script<P: AsRef<Path>>(records: &[Record], vbs_path: &P) {
    let script = find_script(records);
    std::fs::write(vbs_path, script).unwrap();
}

pub fn read_gamedata<F: Seek + Read>(comp: &mut CompoundFile<F>) -> std::io::Result<Vec<Record>> {
    let mut game_data_vec = Vec::new();
    let game_data_path = Path::new(MAIN_SEPARATOR_STR)
        .join("GameStg")
        .join("GameData");
    let mut stream = comp.open_stream(game_data_path)?;
    stream.read_to_end(&mut game_data_vec)?;

    // let result = parseGameData(&game_data_vec[..]);
    // dump(result);

    //let (_, records) = gamedata::read_all_gamedata_records2(&game_data_vec[..]).unwrap();
    let records = gamedata::read_all_gamedata_records(&game_data_vec[..]);
    Ok(records)
}

fn write_game_data<F: Read + Write + Seek>(
    comp: &mut CompoundFile<F>,
    data: &[Record],
) -> Result<(), io::Error> {
    let game_data_path = Path::new(MAIN_SEPARATOR_STR)
        .join("GameStg")
        .join("GameData");
    let mut game_data_stream = comp.create_stream(&game_data_path)?;
    let data = gamedata::write_all_gamedata_records(data);
    game_data_stream.write_all(&data)
    // this flush was required before but now it's working without
    // game_data_stream.flush()
}

pub fn find_script(records: &[Record]) -> String {
    let code = records
        .iter()
        .find_map(|r| match r {
            Record::Code { script } => Some(script),
            _ => None,
        })
        .unwrap();

    code.to_owned()
}

pub fn diff<P: AsRef<Path>>(vpx_file_path: P) -> io::Result<String> {
    // set extension for PathBuf
    let vbs_path = vpx_file_path.as_ref().with_extension("vbs");
    let original_vbs_path = vpx_file_path.as_ref().with_extension("vbs.original.tmp");

    if vbs_path.exists() {
        match cfb::open(&vpx_file_path) {
            Ok(mut comp) => {
                let records = read_gamedata(&mut comp)?;
                let script = find_script(&records);
                std::fs::write(&original_vbs_path, script).unwrap();
                let diff_color = if colored::control::SHOULD_COLORIZE.should_colorize() {
                    DiffColor::Always
                } else {
                    DiffColor::Never
                };
                let output = run_diff(&original_vbs_path, &vbs_path, diff_color)?;

                if original_vbs_path.exists() {
                    std::fs::remove_file(original_vbs_path).unwrap();
                }
                Ok(String::from_utf8_lossy(&output).to_string())
            }
            Err(e) => {
                let msg = format!(
                    "Not a valid vpx file {}: {}",
                    vpx_file_path.as_ref().display(),
                    e
                );
                Err(io::Error::new(io::ErrorKind::InvalidData, msg))
            }
        }
    } else {
        // wrap the error
        let msg = format!("No sidecar vbs file found: {}", vbs_path.display());
        Err(io::Error::new(io::ErrorKind::NotFound, msg))
    }
}

pub enum DiffColor {
    Always,
    Never,
}

impl DiffColor {
    // to color arg
    fn to_diff_arg(&self) -> String {
        match self {
            DiffColor::Always => String::from("always"),
            DiffColor::Never => String::from("never"),
        }
    }
}

pub fn run_diff(
    original_vbs_path: &Path,
    vbs_path: &Path,
    color: DiffColor,
) -> Result<Vec<u8>, io::Error> {
    let parent = vbs_path
        .parent()
        //.and_then(|f| f.parent())
        .unwrap_or(Path::new("."));
    let original_vbs_filename = original_vbs_path
        .file_name()
        .unwrap_or(original_vbs_path.as_os_str());
    let vbs_filename = vbs_path.file_name().unwrap_or(vbs_path.as_os_str());
    std::process::Command::new("diff")
        .current_dir(parent)
        .arg("-u")
        .arg("-w")
        .arg(format!("--color={}", color.to_diff_arg()))
        .arg(original_vbs_filename)
        .arg(vbs_filename)
        .output()
        .map(|o| o.stdout)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_write_read() {
        let buff = Cursor::new(vec![0; 15]);
        let mut comp = CompoundFile::create(buff).unwrap();
        write_minimal_vpx(&mut comp).unwrap();

        let version = read_version(&mut comp).unwrap();
        let tableinfo = tableinfo::read_tableinfo(&mut comp).unwrap();
        let game_data = read_gamedata(&mut comp).unwrap();

        assert_eq!(tableinfo, TableInfo::new());
        assert_eq!(version, 1072);
        let expected: Vec<Record> = vec![];
        assert_eq!(game_data, expected);
    }

    #[test]
    fn test_mac_generation() {
        let path = PathBuf::from("testdata/completely_blank_table_10_7_4.vpx");
        let mut comp = cfb::open(path).unwrap();

        let expected = [
            231, 121, 242, 251, 174, 227, 247, 90, 58, 105, 13, 92, 13, 73, 151, 86,
        ];

        let mac = read_mac(&mut comp).unwrap();
        assert_eq!(mac, expected);

        let generated_mac = generate_mac(&mut comp).unwrap();
        assert_eq!(mac, generated_mac);
    }

    #[test]
    fn test_minimal_mac() {
        let buff = Cursor::new(vec![0; 15]);
        let mut comp = CompoundFile::create(buff).unwrap();
        write_minimal_vpx(&mut comp).unwrap();

        let mac = read_mac(&mut comp).unwrap();
        let expected = [
            62, 193, 68, 87, 87, 196, 78, 210, 132, 41, 127, 127, 148, 175, 9, 37,
        ];
        assert_eq!(mac, expected);
    }

    #[test]
    fn read_write_gamedata() {
        let path = PathBuf::from("testdata/completely_blank_table_10_7_4.vpx");
        let mut comp = cfb::open(path).unwrap();
        let original = read_gamedata(&mut comp).unwrap();

        let buff = Cursor::new(vec![0; 15]);
        let mut comp = CompoundFile::create(buff).unwrap();
        create_game_storage(&mut comp).unwrap();
        write_game_data(&mut comp, &original).unwrap();

        let read = read_gamedata(&mut comp).unwrap();

        assert_eq!(original, read);
    }
}
