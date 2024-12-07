use std::path::{Path, PathBuf};

use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::{env, io};

use log::info;
use std::io::Write;

const CONFIGURATION_FILE_NAME: &str = "vpxtool.cfg";

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub vpx_executable: PathBuf,
    pub tables_folder: Option<PathBuf>,
    pub editor: Option<String>,
}
impl Config {
    fn from(resolved_config: &ResolvedConfig) -> Self {
        Config {
            vpx_executable: resolved_config.vpx_executable.clone(),
            tables_folder: Some(resolved_config.tables_folder.clone()),
            editor: resolved_config.editor.clone(),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct ResolvedConfig {
    pub vpx_executable: PathBuf,
    pub tables_folder: PathBuf,
    pub tables_index_path: PathBuf,
    pub editor: Option<String>,
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
pub(crate) struct PlayfieldInfo {
    pub(crate) fullscreen: bool,
    pub(crate) x: Option<u32>,
    pub(crate) y: Option<u32>,
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
}

pub(crate) struct VPinballConfig {
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

    pub fn get_playfield_info(&self) -> Option<PlayfieldInfo> {
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

            Some(PlayfieldInfo {
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

impl ResolvedConfig {
    pub fn global_pinmame_folder(&self) -> PathBuf {
        // first we try to read the ini file
        let ini_file = self.vpinball_ini_file();
        if ini_file.exists() {
            let vpinball_config = VPinballConfig::read(&ini_file).unwrap();
            if let Some(value) = vpinball_config.get_pinmame_path() {
                // if the path exists we return it
                let path = PathBuf::from(value);
                if path.exists() {
                    return path;
                }
            }
        }

        if cfg!(target_os = "windows") {
            self.vpx_executable.parent().unwrap().join("VPinMAME")
        } else {
            dirs::home_dir().unwrap().join(".pinmame")
        }
    }

    pub fn global_pinmame_rom_folder(&self) -> PathBuf {
        self.global_pinmame_folder().join("roms")
    }

    pub fn vpinball_ini_file(&self) -> PathBuf {
        if cfg!(target_os = "windows") {
            // in the same directory as the vpx executable
            self.vpx_executable.parent().unwrap().join("VPinballX.ini")
        } else {
            dirs::home_dir()
                .unwrap()
                .join(".vpinball")
                .join("VPinballX.ini")
        }
    }
}

pub fn config_path() -> Option<PathBuf> {
    let home_directory_configuration_path = home_config_path();
    if home_directory_configuration_path.exists() {
        return Some(home_directory_configuration_path);
    } else {
        let local_configuration_path = local_config_path();
        if local_configuration_path.exists() {
            return Some(local_configuration_path);
        }
    }
    None
}

pub enum SetupConfigResult {
    Configured(PathBuf),
    Existing(PathBuf),
}

pub(crate) fn setup_config() -> io::Result<SetupConfigResult> {
    // TODO check if the config file already exists
    let existing_config_path = config_path();
    match existing_config_path {
        Some(path) => Ok(SetupConfigResult::Existing(path)),
        None => {
            // TODO avoid stdout interaction here
            println!("Warning: Failed find a config file.");
            let new_config = create_default_config()?;
            Ok(SetupConfigResult::Configured(new_config.0))
        }
    }
}

pub fn load_or_setup_config() -> io::Result<(PathBuf, ResolvedConfig)> {
    match load_config()? {
        Some(loaded) => Ok(loaded),
        None => {
            // TODO avoid stdout interaction here
            println!("Warning: Failed find a config file.");
            create_default_config()
        }
    }
}

pub fn load_config() -> io::Result<Option<(PathBuf, ResolvedConfig)>> {
    match config_path() {
        Some(config_path) => {
            let config = read_config(&config_path)?;
            Ok(Some((config_path, config)))
        }
        None => Ok(None),
    }
}

fn read_config(config_path: &Path) -> io::Result<ResolvedConfig> {
    let figment = Figment::new().merge(Toml::file(config_path));
    // TODO avoid unwrap
    let config: Config = figment.extract().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to load config file: {}", e),
        )
    })?;
    // apply defaults
    // TODO we might want to suggest the value in the config file by having it empty with a comment
    let tables_folder = config
        .tables_folder
        .unwrap_or(default_tables_root(&config.vpx_executable));
    let resolved_config = ResolvedConfig {
        vpx_executable: config.vpx_executable,
        tables_folder: tables_folder.clone(),
        tables_index_path: tables_index_path(&tables_folder),
        editor: config.editor,
    };
    Ok(resolved_config)
}

pub(crate) fn tables_index_path(tables_folder: &Path) -> PathBuf {
    tables_folder.join("vpxtool_index.json")
}

pub fn clear_config() -> io::Result<Option<PathBuf>> {
    let config_path = config_path();
    match config_path {
        Some(path) => {
            std::fs::remove_file(&path)?;
            Ok(Some(path))
        }
        None => Ok(None),
    }
}

fn local_config_path() -> PathBuf {
    Path::new(CONFIGURATION_FILE_NAME).to_path_buf()
}

fn home_config_path() -> PathBuf {
    dirs::config_dir().unwrap().join(CONFIGURATION_FILE_NAME)
}

fn create_default_config() -> io::Result<(PathBuf, ResolvedConfig)> {
    let local_configuration_path = local_config_path();
    let home_directory_configuration_path = home_config_path();
    let choices: Vec<(&str, String)> = vec![
        (
            "Home",
            home_directory_configuration_path
                .to_string_lossy()
                .to_string(),
        ),
        (
            "Local",
            local_configuration_path.to_string_lossy().to_string(),
        ),
    ];

    let selection_opt = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a configuration location:")
        .default(0)
        .items(
            &choices
                .iter()
                .map(|(choice, description)| format!("{} \x1b[90m{}\x1b[0m", choice, description))
                .collect::<Vec<_>>(),
        )
        .interact_opt()
        .unwrap();

    let config_file = if let Some(index) = selection_opt {
        let (_selected_choice, path) = (&choices[index].0, &choices[index].1);
        PathBuf::from(path)
    } else {
        unreachable!("Failed to select a configuration file path.");
    };

    let mut vpx_executable = default_vpinball_executable_detection();

    if !vpx_executable.exists() {
        println!("Warning: Failed to detect the vpinball executable.");
        print!("vpinball executable path: ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut new_executable_path = String::new();
        io::stdin()
            .read_line(&mut new_executable_path)
            .expect("Failed to read line");

        vpx_executable = PathBuf::from(new_executable_path.trim().to_string());

        if !vpx_executable.exists() {
            println!("Error: input file path wasn't found.");
            println!("Executable path is not set. ");

            std::process::exit(1);
        }
    }

    let tables_root = default_tables_root(&vpx_executable);
    let index_path = tables_index_path(&tables_root);

    let resolved_config = ResolvedConfig {
        vpx_executable,
        tables_folder: tables_root,
        tables_index_path: index_path,
        editor: None,
    };
    let config = Config::from(&resolved_config);

    // write config to config_file
    write_config(&config_file, &config)?;
    Ok((config_file, resolved_config))
}

fn write_config(config_file: &Path, config: &Config) -> io::Result<()> {
    let toml = toml::to_string(&config).unwrap();
    let mut file = File::create(config_file)?;
    file.write_all(toml.as_bytes())
}

pub fn default_tables_root(vpx_executable: &Path) -> PathBuf {
    // when on macos we assume that the tables are in ~/.vpinball/tables
    if cfg!(target_os = "macos") {
        dirs::home_dir().unwrap().join(".vpinball").join("tables")
    } else {
        vpx_executable.parent().unwrap().join("tables")
    }
}

fn default_vpinball_executable_detection() -> PathBuf {
    if cfg!(target_os = "windows") {
        // baller installer default
        let dir = PathBuf::from("c:\\vPinball\\VisualPinball");
        let exe = dir.join("VPinballX64.exe");

        // Check current directory
        let local = env::current_dir().unwrap();
        if local.join("VPinballX64.exe").exists() {
            local.join("VPinballX64.exe")
        } else if local.join("VPinballX.exe").exists() {
            local.join("VPinballX.exe")
        } else if exe.exists() {
            exe
        } else {
            dir.join("VPinballX.exe")
        }
    } else if cfg!(target_os = "macos") {
        let dmg_install =
            PathBuf::from("/Applications/VPinballX_GL.app/Contents/MacOS/VPinballX_GL");
        if dmg_install.exists() {
            dmg_install
        } else {
            let mut local = env::current_dir().unwrap();
            local = local.join("VPinballX_GL");
            local
        }
    } else {
        let mut local = env::current_dir().unwrap();
        local = local.join("VPinballX_GL");

        if local.exists() {
            local
        } else {
            let home = dirs::home_dir().unwrap();
            home.join("vpinball").join("vpinball").join("VPinballX_GL")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use testdir::testdir;

    // test that we can read a incomplete config file with missing tables_folder
    #[test]
    fn test_read_incomplete_config() -> io::Result<()> {
        // create a temporary file
        let temp_dir = testdir!();
        let config_file = temp_dir.join(CONFIGURATION_FILE_NAME);
        // write a string
        let mut file = File::create(&config_file)?;
        file.write_all(b"vpx_executable = \"/tmp/test/vpinball\"")?;

        let config = read_config(&config_file)?;

        if cfg!(target_os = "macos") {
            let expected_tables_dir = dirs::home_dir().unwrap().join(".vpinball").join("tables");
            assert_eq!(
                config,
                ResolvedConfig {
                    vpx_executable: PathBuf::from("/tmp/test/vpinball"),
                    tables_folder: expected_tables_dir.clone(),
                    tables_index_path: expected_tables_dir.join("vpxtool_index.json"),
                    editor: None,
                }
            );
        } else {
            assert_eq!(
                config,
                ResolvedConfig {
                    vpx_executable: PathBuf::from("/tmp/test/vpinball"),
                    tables_folder: PathBuf::from("/tmp/test/tables"),
                    tables_index_path: PathBuf::from("/tmp/test/tables/vpxtool_index.json"),
                    editor: None,
                }
            );
        }
        Ok(())
    }
}
