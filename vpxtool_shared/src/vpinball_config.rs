use log::info;
use std::fmt::Display;
use std::io;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WindowType {
    Playfield,
    PinMAME,
    FlexDMD,
    B2SBackglass,
    /// FullDMD
    B2SDMD,
    PUPTopper,
    PUPBackglass,
    PUPDMD,
    PUPPlayfield,
    PUPFullDMD,
}
impl Display for WindowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowType::Playfield => write!(f, "Playfield"),
            WindowType::PinMAME => write!(f, "PinMAME"),
            WindowType::FlexDMD => write!(f, "FlexDMD"),
            WindowType::B2SBackglass => write!(f, "B2SBackglass"),
            WindowType::B2SDMD => write!(f, "B2SDMD"),
            WindowType::PUPTopper => write!(f, "PUPTopper"),
            WindowType::PUPBackglass => write!(f, "PUPBackglass"),
            WindowType::PUPDMD => write!(f, "PUPDMD"),
            WindowType::PUPPlayfield => write!(f, "PUPPlayfield"),
            WindowType::PUPFullDMD => write!(f, "PUPFullDMD"),
        }
    }
}

fn config_prefix(window_type: WindowType) -> &'static str {
    match window_type {
        WindowType::Playfield => "Playfield",
        WindowType::PinMAME => "PinMAMEWindow",
        WindowType::FlexDMD => "FlexDMDWindow",
        WindowType::B2SBackglass => "B2SBackglass",
        WindowType::B2SDMD => "B2SDMD",
        WindowType::PUPTopper => "PUPTopperWindow",
        WindowType::PUPBackglass => "PUPBackglassWindow",
        WindowType::PUPDMD => "PUPDMDWindow",
        WindowType::PUPPlayfield => "PUPPlayfieldWindow",
        WindowType::PUPFullDMD => "PUPFullDMDWindow",
    }
}

fn section_name(window_type: WindowType) -> String {
    match window_type {
        WindowType::Playfield => "Player".to_string(),
        _ => "Standalone".to_string(),
    }
}

/// FullScreen = 0
/// PlayfieldFullScreen = 0
/// WindowPosX =
/// PlayfieldWindowPosX =
/// WindowPosY =
/// PlayfieldWindowPosY =
/// Width = 540
/// PlayfieldWidth = 540
/// Height = 960
/// PlayfieldHeight = 960
///
/// Note: For macOS with hidpi screen this these are logical sizes/locations, not pixel sizes
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub fullscreen: Option<bool>,
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub struct VPinballConfig {
    ini: ini::Ini,
}

impl Default for VPinballConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl VPinballConfig {
    pub fn new() -> Self {
        VPinballConfig {
            ini: ini::Ini::new(),
        }
    }

    pub fn read(ini_path: &Path) -> io::Result<Self> {
        info!("Reading vpinball ini file: {:?}", ini_path);
        let ini = ini::Ini::load_from_file(ini_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to read ini file: {:?}", e),
            )
        })?;
        Ok(VPinballConfig { ini })
    }

    pub fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let ini = ini::Ini::read_from(reader).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to read ini file: {:?}", e),
            )
        })?;
        Ok(VPinballConfig { ini })
    }

    pub fn write(&self, ini_path: &Path) -> io::Result<()> {
        self.ini.write_to_file(ini_path)
    }

    pub fn write_to<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.ini.write_to(writer)
    }

    pub fn get_pinmame_path(&self) -> Option<String> {
        if let Some(standalone_section) = self.ini.section(Some("Standalone")) {
            standalone_section.get("PinMAMEPath").map(|s| s.to_string())
        } else {
            None
        }
    }

    pub fn is_window_enabled(&self, window_type: WindowType) -> bool {
        match window_type {
            WindowType::Playfield => true,
            WindowType::B2SBackglass => {
                let section = section_name(window_type);
                if let Some(ini_section) = self.ini.section(Some(section)) {
                    // TODO what are the defaults here>
                    ini_section.get("B2SWindows") == Some("1")
                } else {
                    false
                }
            }
            WindowType::B2SDMD => {
                let section = section_name(window_type);
                if let Some(ini_section) = self.ini.section(Some(section)) {
                    // TODO what are the defaults here>
                    ini_section.get("B2SWindows") == Some("1")
                        && ini_section.get("B2SHideB2SDMD") == Some("0")
                } else {
                    false
                }
            }
            WindowType::PUPDMD
            | WindowType::PUPBackglass
            | WindowType::PUPTopper
            | WindowType::PUPFullDMD
            | WindowType::PUPPlayfield => {
                let section = section_name(window_type);
                if let Some(ini_section) = self.ini.section(Some(section)) {
                    ini_section.get("PUPWindows") == Some("1")
                } else {
                    false
                }
            }
            WindowType::FlexDMD | WindowType::PinMAME => {
                let section = section_name(window_type);
                let prefix = config_prefix(window_type);
                self.ini.section(Some(section)).is_some_and(|ini_section| {
                    ini_section.get(format!("{}Window", prefix)) == Some("1")
                })
            }
        }
    }

    pub fn get_window_info(&self, window_type: WindowType) -> Option<WindowInfo> {
        let section = section_name(window_type);
        match window_type {
            WindowType::Playfield => {
                if let Some(ini_section) = self.ini.section(Some(section)) {
                    // get all the values from PlayfieldXXX and fall back to the normal values
                    let fullscreen = match ini_section.get("PlayfieldFullScreen") {
                        Some("1") => Some(true),
                        Some("0") => Some(false),
                        Some(empty) if empty.trim().is_empty() => None,
                        Some(other) => {
                            log::warn!("Unexpected value for PlayfieldFullScreen: {}", other);
                            None
                        }
                        None => match ini_section.get("FullScreen") {
                            Some("1") => Some(true),
                            Some("0") => Some(false),
                            Some(empty) if empty.trim().is_empty() => None,
                            Some(other) => {
                                log::warn!("Unexpected value for FullScreen: {}", other);
                                None
                            }
                            None => None,
                        },
                    };
                    let x = ini_section
                        .get("PlayfieldWndX")
                        .or_else(|| ini_section.get("WindowPosX"))
                        .and_then(|s| s.parse::<u32>().ok());

                    let y = ini_section
                        .get("PlayfieldWndY")
                        .or_else(|| ini_section.get("WindowPosY"))
                        .and_then(|s| s.parse::<u32>().ok());

                    let width = ini_section
                        .get("PlayfieldWidth")
                        .or_else(|| ini_section.get("Width"))
                        .and_then(|s| s.parse::<u32>().ok());

                    let height = ini_section
                        .get("PlayfieldHeight")
                        .or_else(|| ini_section.get("Height"))
                        .and_then(|s| s.parse::<u32>().ok());

                    Some(WindowInfo {
                        fullscreen,
                        x,
                        y,
                        width,
                        height,
                    })
                } else {
                    None
                }
            }
            other => self.lookup_window_info(other),
        }
    }

    pub fn set_window_position(&mut self, window_type: WindowType, x: u32, y: u32) {
        let section = section_name(window_type);
        let prefix = config_prefix(window_type);
        // preferably we would write a comment but the ini crate does not support that
        // see https://github.com/zonyitoo/rust-ini/issues/77

        let x_suffix = match window_type {
            WindowType::Playfield => "WndX",
            _ => "X",
        };
        let y_suffix = match window_type {
            WindowType::Playfield => "WndY",
            _ => "Y",
        };

        let x_key = format!("{}{}", prefix, x_suffix);
        let y_key = format!("{}{}", prefix, y_suffix);

        self.ini
            .with_section(Some(&section))
            .set(x_key, x.to_string())
            .set(y_key, y.to_string());
    }

    pub fn set_window_size(&mut self, window_type: WindowType, width: u32, height: u32) {
        let section = section_name(window_type);
        let prefix = config_prefix(window_type);

        let width_key = format!("{}{}", prefix, "Width");
        let height_key = format!("{}{}", prefix, "Height");

        self.ini
            .with_section(Some(&section))
            .set(width_key, width.to_string())
            .set(height_key, height.to_string());
    }

    fn lookup_window_info(&self, window_type: WindowType) -> Option<WindowInfo> {
        let section = section_name(window_type);
        if let Some(ini_section) = self.ini.section(Some(section)) {
            let prefix = config_prefix(window_type);
            let fullscreen = ini_section
                .get(format!("{}{}", prefix, "FullScreen"))
                .map(|s| s == "1");
            let x = ini_section
                .get(format!("{}{}", prefix, "X"))
                .and_then(|s| s.parse::<u32>().ok());
            let y = ini_section
                .get(format!("{}{}", prefix, "Y"))
                .and_then(|s| s.parse::<u32>().ok());
            let width = ini_section
                .get(format!("{}{}", prefix, "Width"))
                .and_then(|s| s.parse::<u32>().ok());
            let height = ini_section
                .get(format!("{}{}", prefix, "Height"))
                .and_then(|s| s.parse::<u32>().ok());
            if x.is_none() && y.is_none() && width.is_none() && height.is_none() {
                return None;
            }
            Some(WindowInfo {
                fullscreen,
                x,
                y,
                width,
                height,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use testdir::testdir;

    #[test]
    fn test_read_vpinball_config() {
        let testdir = testdir!();
        // manually create test ini file
        let ini_path = testdir.join("test.ini");
        std::fs::write(
            &ini_path,
            r#"
[Player]
FullScreen=1
PlayfieldFullScreen=1
PlayfieldWndX=0
PlayfieldWndY=0
PlayfieldWidth=1920
PlayfieldHeight=1080
"#,
        )
        .unwrap();

        let config = VPinballConfig::read(&ini_path).unwrap();
        assert_eq!(
            config
                .get_window_info(WindowType::Playfield)
                .unwrap()
                .fullscreen,
            Some(true)
        );
        assert_eq!(
            config.get_window_info(WindowType::Playfield).unwrap().x,
            Some(0)
        );
        assert_eq!(
            config.get_window_info(WindowType::Playfield).unwrap().y,
            Some(0)
        );
        assert_eq!(
            config.get_window_info(WindowType::Playfield).unwrap().width,
            Some(1920)
        );
        assert_eq!(
            config
                .get_window_info(WindowType::Playfield)
                .unwrap()
                .height,
            Some(1080)
        );
    }

    #[test]
    fn test_write_vpinball_config() {
        let mut config = VPinballConfig::default();
        config.set_window_position(WindowType::Playfield, 100, 200);
        config.set_window_size(WindowType::Playfield, 300, 400);
        let mut cursor = io::Cursor::new(Vec::new());
        config.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        let config_read = VPinballConfig::read_from(&mut cursor).unwrap();
        assert_eq!(
            config_read
                .get_window_info(WindowType::Playfield)
                .unwrap()
                .x,
            Some(100)
        );
        assert_eq!(
            config_read
                .get_window_info(WindowType::Playfield)
                .unwrap()
                .y,
            Some(200)
        );
    }
}
