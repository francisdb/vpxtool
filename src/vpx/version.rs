use std::{
    cmp,
    fmt::Display,
    io::{Read, Seek, Write, self},
    path::{Path, MAIN_SEPARATOR_STR},
};

use byteorder::{LittleEndian, WriteBytesExt};
use cfb::CompoundFile;
use nom::{number::complete::le_u32, IResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Version(u32);
impl Version {
    pub fn new(version: u32) -> Self {
        Version(version)
    }

    pub fn u32(&self) -> u32 {
        self.0
    }

    fn version_float(&self) -> f32 {
        (self.0 as f32) / 100f32
    }
}
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let version_float = self.version_float();
        write!(f, "{}", version_float)
    }
}
impl From<Version> for u32 {
    fn from(val: Version) -> Self {
        val.0
    }
}
impl cmp::PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// Read version
// https://github.com/vbousquet/vpx_lightmapper/blob/331a8576bb7b86668a023b304e7dd04261487106/addons/vpx_lightmapper/vlm_import.py#L328
pub fn read_version<F: Read + Seek>(comp: &mut CompoundFile<F>) -> io::Result<Version> {
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
    Ok(Version(version))
}

pub fn write_version<F: Read + Write + Seek>(
    comp: &mut CompoundFile<F>,
    version: &Version,
) -> std::io::Result<()> {
    // we expect GameStg to exist
    let version_path = Path::new(MAIN_SEPARATOR_STR)
        .join("GameStg")
        .join("Version");
    let mut stream = comp.create_stream(version_path)?;
    stream.write_u32::<LittleEndian>(version.0)
}
