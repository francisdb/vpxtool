use std::io::{self, Read};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

use cfb::CompoundFile;

use gamedata::Record;
use nom::number::complete::le_u32;
use nom::IResult;

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

pub fn extractvbs(vpx_file_path: &str, overwrite: bool) -> ExtractResult {
    let vbs_path = vbs_path_for(vpx_file_path);

    if !vbs_path.exists() || (vbs_path.exists() && overwrite) {
        let mut comp = cfb::open(vpx_file_path).unwrap();
        let _version = read_version(&mut comp);
        let records = read_gamedata(&mut comp);
        extract_script(&records, &vbs_path);
        ExtractResult::Extracted(vbs_path.to_path_buf())
    } else {
        ExtractResult::Existed(vbs_path.to_path_buf())
    }
}

pub fn vbs_path_for(vpx_file_path: &str) -> PathBuf {
    let vbs_path_str = vpx_file_path.replace(".vpx", ".vbs");
    Path::new(&vbs_path_str).to_path_buf()
}

// TODO: read version
//  https://github.com/vbousquet/vpx_lightmapper/blob/331a8576bb7b86668a023b304e7dd04261487106/addons/vpx_lightmapper/vlm_import.py#L328
pub fn read_version(comp: &mut cfb::CompoundFile<std::fs::File>) -> u32 {
    let mut file_version = Vec::new();
    comp.open_stream("/GameStg/Version")
        .unwrap()
        .read_to_end(&mut file_version)
        .unwrap();

    fn read_version(input: &[u8]) -> IResult<&[u8], u32> {
        le_u32(input)
    }

    // use lut to read as u32
    let (_, version) = read_version(&file_version[..]).unwrap();

    // let version_float = (version as f32)/100f32;
    // println!("VPX file version: {}", version);
    version
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

pub fn diff(vpx_file_path: &str) -> io::Result<String> {
    let vbs_path_str = vpx_file_path.replace(".vpx", ".vbs");
    let vbs_path = Path::new(&vbs_path_str);
    let original_vbs_path_str = vpx_file_path.replace(".vpx", ".vbs.original.tmp");
    let original_vbs_path = Path::new(&original_vbs_path_str);

    if vbs_path.exists() {
        match cfb::open(vpx_file_path) {
            Ok(mut comp) => {
                let records = read_gamedata(&mut comp);
                let script = find_script(&records);
                std::fs::write(original_vbs_path, script).unwrap();

                let output = run_diff(original_vbs_path, vbs_path)?;

                if original_vbs_path.exists() {
                    std::fs::remove_file(original_vbs_path).unwrap();
                }
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            Err(e) => {
                let msg = format!("Not a valid vpx file {}: {}", vpx_file_path, e);
                Err(io::Error::new(io::ErrorKind::InvalidData, msg))
            }
        }
    } else {
        // wrap the error
        let msg = format!("No sidecar vbs file found: {}", vbs_path.display());
        Err(io::Error::new(io::ErrorKind::NotFound, msg))
    }
}

fn run_diff(original_vbs_path: &Path, vbs_path: &Path) -> Result<std::process::Output, io::Error> {
    std::process::Command::new("diff")
        .arg("-u")
        .arg("-w")
        .arg("--color=always")
        .arg(original_vbs_path)
        .arg(vbs_path)
        .output()
}
