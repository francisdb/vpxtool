use std::io::{self, Read, Seek, Write};
use std::path::MAIN_SEPARATOR_STR;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

use cfb::CompoundFile;

use md2::{Digest, Md2};

use crate::vpx::biff::BiffReader;

use tableinfo::{write_tableinfo, TableInfo};
use version::Version;

use self::collection::Collection;
use self::font::FontData;
use self::gamedata::GameData;
use self::gameitem::GameItemEnum;
use self::image::ImageData;
use self::sound::SoundData;

pub mod biff;
pub mod collection;
pub mod color;
pub mod expanded;
pub mod font;
pub mod gamedata;
pub mod gameitem;
pub mod image;
pub mod sound;
pub mod tableinfo;
pub mod version;

pub struct VPX {
    info: TableInfo,
    version: Version,
    gamedata: GameData,
    gameitems: Vec<gameitem::GameItemEnum>,
    images: Vec<image::ImageData>,
    sounds: Vec<sound::SoundData>,
    fonts: Vec<font::FontData>,
    collections: Vec<collection::Collection>,
}

pub enum ExtractResult {
    Extracted(PathBuf),
    Existed(PathBuf),
}

pub enum VerifyResult {
    Ok(PathBuf),
    Failed(PathBuf, String),
}

pub fn read(path: &PathBuf) -> io::Result<VPX> {
    let file = File::open(path)?;
    let mut comp = CompoundFile::open(file)?;
    read_vpx(&mut comp)
}

pub fn read_vpx<F: Read + Write + Seek>(comp: &mut CompoundFile<F>) -> io::Result<VPX> {
    let info = tableinfo::read_tableinfo(comp)?;
    let version = version::read_version(comp)?;
    let gamedata = read_gamedata(comp)?;
    let gameitems = read_gameitems(comp, &gamedata)?;
    let images = read_images(comp, &gamedata)?;
    let sounds = read_sounds(comp, &gamedata, &version)?;
    let fonts = read_fonts(comp, &gamedata)?;
    let collections = read_collections(comp, &gamedata)?;
    Ok(VPX {
        info,
        version,
        gamedata,
        gameitems,
        images,
        sounds,
        fonts,
        collections,
    })
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
    version::write_version(comp, Version::new(1072))?;
    write_game_data(comp, &GameData::default())?;
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
        let _version = version::read_version(&mut comp);
        let gamedata = read_gamedata(&mut comp).unwrap();
        extract_script(&gamedata, &script_path).unwrap();
        ExtractResult::Extracted(script_path)
    } else {
        ExtractResult::Existed(script_path)
    }
}

pub fn importvbs(vpx_file_path: &PathBuf, extension: Option<&str>) -> std::io::Result<PathBuf> {
    let script_path = match extension {
        Some(ext) => path_for(vpx_file_path, ext),
        None => vbs_path_for(vpx_file_path),
    };
    if !script_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Script file not found: {}", script_path.display()),
        ));
    }
    let mut comp = cfb::open_rw(vpx_file_path)?;
    let mut gamedata = read_gamedata(&mut comp)?;
    let script = std::fs::read_to_string(&script_path)?;
    gamedata.set_code(script);
    write_game_data(&mut comp, &gamedata)?;
    let mac = generate_mac(&mut comp)?;
    write_mac(&mut comp, &mac)?;
    comp.flush()?;
    Ok(script_path)
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

pub fn extract_script<P: AsRef<Path>>(gamedata: &GameData, vbs_path: &P) -> Result<(), io::Error> {
    let script = &gamedata.code;
    std::fs::write(vbs_path, script)
}

pub fn read_gamedata<F: Seek + Read>(comp: &mut CompoundFile<F>) -> std::io::Result<GameData> {
    let mut game_data_vec = Vec::new();
    let game_data_path = Path::new(MAIN_SEPARATOR_STR)
        .join("GameStg")
        .join("GameData");
    let mut stream = comp.open_stream(game_data_path)?;
    stream.read_to_end(&mut game_data_vec)?;
    let gamedata = gamedata::read_all_gamedata_records(&game_data_vec[..]);
    Ok(gamedata)
}

fn write_game_data<F: Read + Write + Seek>(
    comp: &mut CompoundFile<F>,
    gamedata: &GameData,
) -> Result<(), io::Error> {
    let game_data_path = Path::new(MAIN_SEPARATOR_STR)
        .join("GameStg")
        .join("GameData");
    let mut game_data_stream = comp.create_stream(&game_data_path)?;
    let data = gamedata::write_all_gamedata_records(&gamedata);
    game_data_stream.write_all(&data)
    // this flush was required before but now it's working without
    // game_data_stream.flush()
}

fn read_gameitems<F: Read + Seek>(
    comp: &mut CompoundFile<F>,
    gamedata: &GameData,
) -> io::Result<Vec<GameItemEnum>> {
    (0..gamedata.gameitems_size)
        .map(|index| {
            let path = format!("GameStg/GameItem{}", index);
            let mut input = Vec::new();
            let mut stream = comp.open_stream(&path)?;
            stream.read_to_end(&mut input)?;
            let game_item = gameitem::read(&input);
            Ok(game_item)
        })
        .collect()
}

fn read_sounds<F: Read + Seek>(
    comp: &mut CompoundFile<F>,
    gamedata: &GameData,
    file_version: &Version,
) -> std::io::Result<Vec<SoundData>> {
    (0..gamedata.sounds_size)
        .map(|index| {
            let path = Path::new(MAIN_SEPARATOR_STR)
                .join("GameStg")
                .join(format!("Sound{}", index));
            let mut input = Vec::new();
            let mut stream = comp.open_stream(&path)?;
            stream.read_to_end(&mut input)?;
            let (_, sound) =
                sound::read(path.display().to_string(), file_version.clone(), &input).unwrap();
            Ok(sound)
        })
        .collect()
}

fn read_collections<F: Read + Seek>(
    comp: &mut CompoundFile<F>,
    gamedata: &GameData,
) -> io::Result<Vec<Collection>> {
    (0..gamedata.collections_size)
        .map(|index| {
            let path = format!("GameStg/Collection{}", index);
            let mut input = Vec::new();
            let mut stream = comp.open_stream(&path)?;
            stream.read_to_end(&mut input)?;
            Ok(collection::read(&input))
        })
        .collect()
}

fn read_images<F: Read + Seek>(
    comp: &mut CompoundFile<F>,
    gamedata: &GameData,
) -> io::Result<Vec<ImageData>> {
    (0..gamedata.images_size)
        .map(|index| {
            let path = format!("GameStg/Image{}", index);
            let mut input = Vec::new();
            let mut stream = comp.open_stream(&path)?;
            stream.read_to_end(&mut input)?;
            Ok(image::read(path, &input))
        })
        .collect()
}

fn read_fonts<F: Read + Seek>(
    comp: &mut CompoundFile<F>,
    gamedata: &GameData,
) -> io::Result<Vec<FontData>> {
    (0..gamedata.fonts_size)
        .map(|index| {
            let path = format!("GameStg/Font{}", index);
            let mut input = Vec::new();
            let mut stream = comp.open_stream(&path)?;
            stream.read_to_end(&mut input)?;

            let font = font::read(&input);
            Ok(font)
        })
        .collect()
}

pub fn diff<P: AsRef<Path>>(vpx_file_path: P) -> io::Result<String> {
    // set extension for PathBuf
    let vbs_path = vpx_file_path.as_ref().with_extension("vbs");
    let original_vbs_path = vpx_file_path.as_ref().with_extension("vbs.original.tmp");

    if vbs_path.exists() {
        match cfb::open(&vpx_file_path) {
            Ok(mut comp) => {
                let gamedata = read_gamedata(&mut comp)?;
                let script = gamedata.code;
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

        let version = version::read_version(&mut comp).unwrap();
        let tableinfo = tableinfo::read_tableinfo(&mut comp).unwrap();
        let game_data = read_gamedata(&mut comp).unwrap();

        assert_eq!(tableinfo, TableInfo::new());
        assert_eq!(version, Version::new(1072));
        let expected = GameData::default();
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
            222, 168, 237, 142, 40, 215, 175, 9, 116, 236, 50, 181, 130, 164, 254, 17,
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

    #[test]
    fn read() {
        let path = PathBuf::from("testdata/completely_blank_table_10_7_4.vpx");
        let mut comp = cfb::open(path).unwrap();
        let original = read_vpx(&mut comp).unwrap();

        let mut expected_info = TableInfo::new();
        expected_info.table_name = String::from("Visual Pinball Demo Table");
        expected_info.table_save_rev = String::from("10");
        expected_info.table_version = String::from("1.2");
        expected_info.author_website = String::from("http://www.vpforums.org/");
        expected_info.table_save_date = String::from("Tue Jul 11 15:48:49 2023");
        expected_info.table_description =
            String::from("Press C to enable manual Ball Control via the arrow keys and B");

        assert_eq!(original.version, Version::new(1072));
        assert_eq!(original.info, expected_info);
        assert_eq!(original.gamedata.collections_size, 9);
        assert_eq!(original.gamedata.images_size, 1);
        assert_eq!(original.gamedata.sounds_size, 0);
        assert_eq!(original.gamedata.fonts_size, 0);
        assert_eq!(original.gamedata.gameitems_size, 73);
        assert_eq!(original.gameitems.len(), 73);
        assert_eq!(original.images.len(), 1);
        assert_eq!(original.sounds.len(), 0);
        assert_eq!(original.fonts.len(), 0);
        assert_eq!(original.collections.len(), 9);
    }
}
