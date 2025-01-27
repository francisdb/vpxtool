use log::info;
use std::path::Path;

#[derive(Debug)]
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

fn config_prefix(window_type: &WindowType) -> &'static str {
    match window_type {
        WindowType::Playfield => "Playfield",
        WindowType::PinMAME => "PinMAME",
        WindowType::FlexDMD => "FlexDMD",
        WindowType::B2SBackglass => "B2SBackglass",
        WindowType::B2SDMD => "B2SDMD",
        WindowType::PUPTopper => "PUPTopper",
        WindowType::PUPBackglass => "PUPBackglass",
        WindowType::PUPDMD => "PUPDMD",
        WindowType::PUPPlayfield => "PUPPlayfield",
        WindowType::PUPFullDMD => "PUPFullDMD",
    }
}

fn section_name(window_type: &WindowType) -> String {
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
    pub fullscreen: bool,
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub struct VPinballConfig {
    ini: ini::Ini,
}

impl VPinballConfig {
    pub fn read(ini_path: &Path) -> Result<Self, ini::Error> {
        info!("Reading vpinball ini file: {:?}", ini_path);
        let ini = ini::Ini::load_from_file(ini_path)?;
        Ok(VPinballConfig { ini })
    }

    pub fn get_pinmame_path(&self) -> Option<String> {
        if let Some(standalone_section) = self.ini.section(Some("Standalone")) {
            standalone_section.get("PinMAMEPath").map(|s| s.to_string())
        } else {
            None
        }
    }

    pub fn is_window_enabled(&self, window: WindowType) -> bool {
        match window {
            WindowType::Playfield => true,
            WindowType::B2SBackglass | WindowType::B2SDMD => {
                let section = section_name(&window);
                if let Some(ini_section) = self.ini.section(Some(section)) {
                    ini_section.get("B2SWindows") == Some("1")
                } else {
                    false
                }
            }
            WindowType::PUPDMD
            | WindowType::PUPBackglass
            | WindowType::PUPTopper
            | WindowType::PUPFullDMD
            | WindowType::PUPPlayfield => {
                let section = section_name(&window);
                if let Some(ini_section) = self.ini.section(Some(section)) {
                    ini_section.get("PUPWindows") == Some("1")
                } else {
                    false
                }
            }
            WindowType::FlexDMD | WindowType::PinMAME => {
                let section = section_name(&window);
                let prefix = config_prefix(&window);
                self.ini.section(Some(section)).is_some_and(|ini_section| {
                    ini_section.get(format!("{}Window", prefix)) == Some("1")
                })
            }
        }
    }

    pub fn get_window_info(&self, window_type: &WindowType) -> Option<WindowInfo> {
        match window_type {
            WindowType::Playfield => {
                if let Some(standalone_section) = self.ini.section(Some("Player")) {
                    // get all the values from PlayfieldXXX and fall back to the normal values
                    let fullscreen = match standalone_section.get("PlayfieldFullScreen") {
                        Some(value) => value == "1",
                        None => match standalone_section.get("FullScreen") {
                            Some(value) => value == "1",
                            None => true, // not sure if this is the correct default value for every os
                        },
                    };
                    let x = standalone_section
                        .get("PlayfieldWndX")
                        .or_else(|| standalone_section.get("WindowPosX"))
                        .and_then(|s| s.parse::<u32>().ok());

                    let y = standalone_section
                        .get("PlayfieldWndY")
                        .or_else(|| standalone_section.get("WindowPosY"))
                        .and_then(|s| s.parse::<u32>().ok());

                    let width = standalone_section
                        .get("PlayfieldWidth")
                        .or_else(|| standalone_section.get("Width"))
                        .and_then(|s| s.parse::<u32>().ok());

                    let height = standalone_section
                        .get("PlayfieldHeight")
                        .or_else(|| standalone_section.get("Height"))
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
            WindowType::B2SBackglass => self.lookup_window_info(&window_type),
            _ => None,
        }
    }

    fn lookup_window_info(&self, window_type: &WindowType) -> Option<WindowInfo> {
        let section = section_name(window_type);
        if let Some(ini_section) = self.ini.section(Some(section)) {
            let prefix = config_prefix(window_type);
            let fullscreen = ini_section
                .get(format!("{}{}", prefix, "FullScreen"))
                .map(|s| s == "1")
                .unwrap_or(false);
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
