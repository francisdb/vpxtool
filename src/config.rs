use std::path::{Path, PathBuf};

use crate::vpinball_config::VPinballConfig;
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::{env, io};

const CONFIGURATION_FILE_NAME: &str = "vpxtool.cfg";

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Eq)]
pub struct LaunchTemplate {
    pub name: String,
    pub executable: PathBuf,
    pub arguments: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub vpx_executable: PathBuf,
    pub vpx_config: Option<PathBuf>,
    pub tables_folder: Option<PathBuf>,
    pub editor: Option<String>,
    pub launch_templates: Option<Vec<LaunchTemplate>>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ResolvedConfig {
    pub vpx_executable: PathBuf,
    pub launch_templates: Vec<LaunchTemplate>,
    pub vpx_config: PathBuf,
    pub tables_folder: PathBuf,
    pub tables_index_path: PathBuf,
    pub editor: Option<String>,
}

impl ResolvedConfig {
    pub fn global_pinmame_folder(&self) -> PathBuf {
        if cfg!(target_os = "windows") {
            self.vpx_executable.parent().unwrap().join("VPinMAME")
        } else {
            dirs::home_dir().unwrap().join(".pinmame")
        }
    }

    /// This path can be absolute or relative.
    /// In case it is relative, it will need to be resolved relative to the table vpx file.
    pub fn configured_pinmame_folder(&self) -> Option<PathBuf> {
        // first we try to read the ini file
        if self.vpx_config.exists() {
            let vpinball_config = VPinballConfig::read(&self.vpx_config).unwrap();
            if let Some(value) = vpinball_config.get_pinmame_path() {
                if value.trim().is_empty() {
                    return None;
                }
                let path = PathBuf::from(value);
                return Some(path);
            }
        }
        None
    }
}

pub fn config_path() -> Option<PathBuf> {
    let home_directory_configuration_path = home_config_path();
    if home_directory_configuration_path.exists() {
        return Some(home_directory_configuration_path);
    }
    let local_configuration_path = local_config_path();
    if local_configuration_path.exists() {
        return Some(local_configuration_path);
    }
    None
}

pub enum SetupConfigResult {
    Configured(PathBuf),
    Existing(PathBuf),
}

/// Setup the config file if it doesn't exist
///
/// This might require user input!
pub fn setup_config() -> io::Result<SetupConfigResult> {
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

/// Load the config file if it exists, otherwise create a new one
///
/// This might require user input!
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
    let vpx_config = config
        .vpx_config
        .unwrap_or_else(|| default_vpinball_ini_file(&config.vpx_executable));

    // generate launch templates if not set
    let launch_templates = config.launch_templates.unwrap_or_else(|| {
        // normal, force fullscreen, force windowed
        generate_default_launch_templates(&config.vpx_executable)
    });

    let resolved_config = ResolvedConfig {
        vpx_executable: config.vpx_executable,
        launch_templates,
        vpx_config,
        tables_folder: tables_folder.clone(),
        tables_index_path: tables_index_path(&tables_folder),
        editor: config.editor,
    };
    Ok(resolved_config)
}

fn generate_default_launch_templates(vpx_executable: &Path) -> Vec<LaunchTemplate> {
    let default_env = HashMap::from([
        ("SDL_VIDEODRIVER".to_string(), "".to_string()),
        ("SDL_RENDER_DRIVER".to_string(), "".to_string()),
    ]);

    vec![
        LaunchTemplate {
            name: "Launch".to_string(),
            executable: vpx_executable.to_owned(),
            arguments: None,
            env: Some(default_env.clone()),
        },
        LaunchTemplate {
            name: "Launch Fullscreen".to_string(),
            executable: vpx_executable.to_owned(),
            arguments: Some(vec!["-EnableTrueFullscreen".to_string()]),
            env: None,
        },
        LaunchTemplate {
            name: "Launch Windowed".to_string(),
            executable: vpx_executable.to_owned(),
            arguments: Some(vec!["-DisableTrueFullscreen".to_string()]),
            env: None,
        },
    ]
}

pub fn tables_index_path(tables_folder: &Path) -> PathBuf {
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

fn default_vpinball_ini_file(vpx_executable_path: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        // in the same directory as the vpx executable
        vpx_executable_path.parent().unwrap().join("VPinballX.ini")
    } else {
        // batocera has a specific location for the ini file
        let batocera_path = PathBuf::from("/userdata/system/configs/vpinball/VPinballX.ini");
        if batocera_path.exists() {
            return batocera_path;
        }

        // default vpinball ini file location is ~/.vpinball/VPinballX.ini
        dirs::home_dir()
            .unwrap()
            .join(".vpinball")
            .join("VPinballX.ini")
    }
}

/// Create a default config file
///
/// This requires user input!
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

    let mut vpx_executable = default_vpinball_executable();

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

    write_default_config(&config_file, &vpx_executable)?;

    let resolved_config = read_config(&config_file)?;
    Ok((config_file, resolved_config))
}

fn write_default_config(config_file: &Path, vpx_executable: &Path) -> io::Result<()> {
    let launch_templates = generate_default_launch_templates(vpx_executable);

    let vpx_config = default_vpinball_ini_file(vpx_executable);
    let tables_folder = default_tables_root(vpx_executable);
    let config = Config {
        vpx_executable: vpx_executable.to_owned(),
        launch_templates: Some(launch_templates),
        vpx_config: Some(vpx_config.clone()),
        tables_folder: Some(tables_folder.clone()),
        editor: None,
    };
    write_config(config_file, &config)?;
    Ok(())
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

fn default_vpinball_executable() -> PathBuf {
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
    use pretty_assertions::assert_eq;
    use testdir::testdir;

    #[cfg(target_os = "linux")]
    #[test]
    fn test_write_default_config_linux() -> io::Result<()> {
        use std::io::Read;
        let temp_dir = testdir!();
        let config_file = temp_dir.join(CONFIGURATION_FILE_NAME);
        write_default_config(&config_file, &PathBuf::from("/home/me/vpinball"))?;
        // print the config file
        let mut file = File::open(&config_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        println!("Config file contents: {}", contents);
        let config = read_config(&config_file)?;
        assert_eq!(
            config,
            ResolvedConfig {
                vpx_executable: PathBuf::from("/home/me/vpinball"),
                launch_templates: vec!(
                    LaunchTemplate {
                        name: "Launch".to_string(),
                        executable: PathBuf::from("/home/me/vpinball"),
                        arguments: None,
                        env: Some(HashMap::from([
                            ("SDL_VIDEODRIVER".to_string(), "".to_string()),
                            ("SDL_RENDER_DRIVER".to_string(), "".to_string()),
                        ])),
                    },
                    LaunchTemplate {
                        name: "Launch Fullscreen".to_string(),
                        executable: PathBuf::from("/home/me/vpinball"),
                        arguments: Some(vec!["-EnableTrueFullscreen".to_string()]),
                        env: None,
                    },
                    LaunchTemplate {
                        name: "Launch Windowed".to_string(),
                        executable: PathBuf::from("/home/me/vpinball"),
                        arguments: Some(vec!["-DisableTrueFullscreen".to_string()]),
                        env: None,
                    },
                ),

                vpx_config: dirs::home_dir().unwrap().join(".vpinball/VPinballX.ini"),
                tables_folder: PathBuf::from("/home/me/tables"),
                tables_index_path: PathBuf::from("/home/me/tables/vpxtool_index.json"),
                editor: None,
            }
        );
        Ok(())
    }

    // test that we can read an incomplete config file with missing tables_folder
    #[cfg(target_os = "linux")]
    #[test]
    fn test_read_incomplete_config_linux() -> io::Result<()> {
        let temp_dir = testdir!();
        let config_file = temp_dir.join(CONFIGURATION_FILE_NAME);
        let mut file = File::create(&config_file)?;
        file.write_all(b"vpx_executable = \"/tmp/test/vpinball\"")?;

        let config = read_config(&config_file)?;

        assert_eq!(
            config,
            ResolvedConfig {
                vpx_executable: PathBuf::from("/tmp/test/vpinball"),
                launch_templates: vec!(
                    LaunchTemplate {
                        name: "Launch".to_string(),
                        executable: PathBuf::from("/tmp/test/vpinball"),
                        arguments: None,
                        env: Some(HashMap::from([
                            ("SDL_VIDEODRIVER".to_string(), "".to_string()),
                            ("SDL_RENDER_DRIVER".to_string(), "".to_string()),
                        ])),
                    },
                    LaunchTemplate {
                        name: "Launch Fullscreen".to_string(),
                        executable: PathBuf::from("/tmp/test/vpinball"),
                        arguments: Some(vec!["-EnableTrueFullscreen".to_string()]),
                        env: None,
                    },
                    LaunchTemplate {
                        name: "Launch Windowed".to_string(),
                        executable: PathBuf::from("/tmp/test/vpinball"),
                        arguments: Some(vec!["-DisableTrueFullscreen".to_string()]),
                        env: None,
                    },
                ),
                vpx_config: dirs::home_dir().unwrap().join(".vpinball/VPinballX.ini"),
                tables_folder: PathBuf::from("/tmp/test/tables"),
                tables_index_path: PathBuf::from("/tmp/test/tables/vpxtool_index.json"),
                editor: None,
            }
        );
        Ok(())
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_read_incomplete_config_macos() -> io::Result<()> {
        let temp_dir = testdir!();
        let config_file = temp_dir.join(CONFIGURATION_FILE_NAME);
        let mut file = File::create(&config_file)?;
        file.write_all(b"vpx_executable = \"/tmp/test/vpinball\"")?;

        let config = read_config(&config_file)?;

        let expected_tables_dir = dirs::home_dir().unwrap().join(".vpinball").join("tables");
        assert_eq!(
            config,
            ResolvedConfig {
                vpx_executable: PathBuf::from("/tmp/test/vpinball"),
                launch_templates: vec!(
                    LaunchTemplate {
                        name: "Launch".to_string(),
                        executable: PathBuf::from("/tmp/test/vpinball"),
                        arguments: None,
                        env: Some(HashMap::from([
                            ("SDL_VIDEODRIVER".to_string(), "".to_string()),
                            ("SDL_RENDER_DRIVER".to_string(), "".to_string()),
                        ])),
                    },
                    LaunchTemplate {
                        name: "Launch Fullscreen".to_string(),
                        executable: PathBuf::from("/tmp/test/vpinball"),
                        arguments: Some(vec!["-EnableTrueFullscreen".to_string()]),
                        env: None,
                    },
                    LaunchTemplate {
                        name: "Launch Windowed".to_string(),
                        executable: PathBuf::from("/tmp/test/vpinball"),
                        arguments: Some(vec!["-DisableTrueFullscreen".to_string()]),
                        env: None,
                    }
                ),
                vpx_config: dirs::home_dir().unwrap().join(".vpinball/VPinballX.ini"),
                tables_folder: expected_tables_dir.clone(),
                tables_index_path: expected_tables_dir.join("vpxtool_index.json"),
                editor: None,
            }
        );
        Ok(())
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_read_incomplete_config_windows() -> io::Result<()> {
        let temp_dir = testdir!();
        let config_file = temp_dir.join(CONFIGURATION_FILE_NAME);
        let mut file = File::create(&config_file)?;
        file.write_all(b"vpx_executable = \"C:\\\\test\\\\vpinball\"")?;

        let config = read_config(&config_file)?;

        assert_eq!(
            config,
            ResolvedConfig {
                vpx_executable: PathBuf::from("C:\\test\\vpinball"),
                vpx_config: PathBuf::from("C:\\test\\VPinballX.ini"),
                tables_folder: PathBuf::from("C:\\test\\tables"),
                tables_index_path: PathBuf::from("C:\\test\\tables\\vpxtool_index.json"),
                editor: None,
                launch_templates: vec!(
                    LaunchTemplate {
                        name: "Launch".to_string(),
                        executable: PathBuf::from("C:\\test\\vpinball"),
                        arguments: None,
                        env: Some(HashMap::from([
                            ("SDL_VIDEODRIVER".to_string(), "".to_string()),
                            ("SDL_RENDER_DRIVER".to_string(), "".to_string()),
                        ])),
                    },
                    LaunchTemplate {
                        name: "Launch Fullscreen".to_string(),
                        executable: PathBuf::from("C:\\test\\vpinball"),
                        arguments: Some(vec!["-EnableTrueFullscreen".to_string()]),
                        env: None,
                    },
                    LaunchTemplate {
                        name: "Launch Windowed".to_string(),
                        executable: PathBuf::from("C:\\test\\vpinball"),
                        arguments: Some(vec!["-DisableTrueFullscreen".to_string()]),
                        env: None,
                    }
                )
            }
        );
        Ok(())
    }
}
