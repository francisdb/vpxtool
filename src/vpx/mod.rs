use std::io::{self, Read, Write};
use std::path::MAIN_SEPARATOR_STR;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use cfb::CompoundFile;

use gamedata::Record;
use nom::number::complete::le_u32;
use nom::IResult;

use self::tableinfo::{write_tableinfo, TableInfo};

pub mod biff;
pub mod font;
pub mod gamedata;
pub mod image;
pub mod sound;
pub mod tableinfo;

pub enum ExtractResult {
    Extracted(PathBuf),
    Existed(PathBuf),
}

pub fn new_minimal_vpx<P: AsRef<Path>>(vpx_file_path: P) -> std::io::Result<()> {
    let table_info = TableInfo::new();
    let file = File::create(vpx_file_path)?;
    let mut comp = CompoundFile::create(file)?;
    write_tableinfo(&mut comp, &table_info)?;
    let game_stg_path = Path::new(MAIN_SEPARATOR_STR).join("GameStg");
    comp.create_storage(&game_stg_path)?;
    write_version(&mut comp, 1072)?;
    let game_data_path = game_stg_path.join("GameData");
    let game_data_stream = comp.create_stream(game_data_path)?;
    write_endb(game_data_stream)?;
    write_mac(&mut comp)
}

pub fn extractvbs(
    vpx_file_path: &PathBuf,
    overwrite: bool,
    extension: Option<&str>,
) -> ExtractResult {
    let script_path = match extension {
        Some(ext) => path_for(vpx_file_path, &ext),
        None => vbs_path_for(vpx_file_path),
    };

    if !script_path.exists() || (script_path.exists() && overwrite) {
        let mut comp = cfb::open(vpx_file_path).unwrap();
        let _version = read_version(&mut comp);
        let records = read_gamedata(&mut comp);
        extract_script(&records, &script_path);
        ExtractResult::Extracted(script_path.to_path_buf())
    } else {
        ExtractResult::Existed(script_path.to_path_buf())
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
pub fn read_version(comp: &mut cfb::CompoundFile<std::fs::File>) -> std::io::Result<u32> {
    let mut file_version = Vec::new();
    comp.open_stream("/GameStg/Version")
        .unwrap()
        .read_to_end(&mut file_version)?;

    fn read_version(input: &[u8]) -> IResult<&[u8], u32> {
        le_u32(input)
    }

    let (_, version) = read_version(&file_version[..]).unwrap();

    // let version_float = (version as f32)/100f32;
    // println!("VPX file version: {}", version);
    Ok(version)
}

pub fn write_version(
    comp: &mut cfb::CompoundFile<std::fs::File>,
    version: u32,
) -> std::io::Result<()> {
    // we expect GameStg to exist
    let version_path = Path::new(MAIN_SEPARATOR_STR)
        .join("GameStg")
        .join("Version");
    let mut stream = comp.create_stream(version_path)?;
    stream.write_u32::<LittleEndian>(version)
}

pub fn write_endb(mut stream: cfb::Stream<File>) -> Result<(), io::Error> {
    // write 4 as le_u32
    stream.write_u32::<LittleEndian>(4)?;
    // write "ENDB" tag as ascii/utf8 string
    let bytes = "ENDB".as_bytes();
    stream.write_all(bytes)
}

pub fn write_mac(comp: &mut cfb::CompoundFile<std::fs::File>) -> Result<(), io::Error> {
    let game_stg_path = Path::new(MAIN_SEPARATOR_STR).join("GameStg");
    let mac_path = game_stg_path.join("MAC");
    let mut mac_stream = comp.create_stream(mac_path)?;
    // TODO implement mac generation, see
    //  https://github.com/freezy/VisualPinball.Engine/blob/ec1e9765cd4832c134e889d6e6d03320bc404bd5/VisualPinball.Engine/VPT/Table/TableWriter.cs#L42
    //  https://github.com/vbousquet/vpx_lightmapper/blob/ca5fddd4c2a0fbe817fd546c5f4db609f9d0da9f/addons/vpx_lightmapper/vlm_export.py#L906-L913
    //  https://github.com/vpinball/vpinball/blob/d9d22a5923ad5a9902a27fae296bc6b2e9ed95ca/pintable.cpp#L2634-L2667
    //  ordering of writes is important co come up with the correct hash

    // write 16 zeroes (for standalone only 16 bytes are read and no checks are applied)
    mac_stream.write_all(&[0; 16])
}

pub fn extract_script<P: AsRef<Path>>(records: &[Record], vbs_path: &P) {
    let script = find_script(records);
    std::fs::write(vbs_path, script).unwrap();
}

pub fn read_gamedata(comp: &mut CompoundFile<File>) -> Vec<Record> {
    let mut game_data_vec = Vec::new();
    comp.open_stream("/GameStg/GameData")
        .unwrap()
        .read_to_end(&mut game_data_vec)
        .unwrap();

    // let result = parseGameData(&game_data_vec[..]);
    // dump(result);

    let (_, records) = gamedata::read_all_gamedata_records(&game_data_vec[..]).unwrap();
    records
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

pub fn diff(vpx_file_path: PathBuf) -> io::Result<String> {
    // set extension for PathBuf
    let vbs_path = vpx_file_path.with_extension("vbs");
    let original_vbs_path = vpx_file_path.with_extension("vbs.original.tmp");

    if vbs_path.exists() {
        match cfb::open(&vpx_file_path) {
            Ok(mut comp) => {
                let records = read_gamedata(&mut comp);
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
                let msg = format!("Not a valid vpx file {}: {}", vpx_file_path.display(), e);
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
