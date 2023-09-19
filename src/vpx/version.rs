use std::{
    cmp,
    fmt::Display,
    io::{self, Read, Seek, Write},
    path::{Path, MAIN_SEPARATOR_STR},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cfb::CompoundFile;

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

pub fn read_version<F: Read + Seek>(comp: &mut CompoundFile<F>) -> io::Result<Version> {
    let version_path = Path::new(MAIN_SEPARATOR_STR)
        .join("GameStg")
        .join("Version");
    let mut stream = comp.open_stream(version_path)?;
    let version = stream.read_u32::<LittleEndian>()?;
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
