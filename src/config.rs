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

use std::io::Write;

const CONFIGURATION_FILE_NAME: &str = "vpxtool.cfg";

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub vpx_executable: PathBuf,
    pub tables_folder: Option<PathBuf>,
}
impl Config {
    fn from(resolved_config: &ResolvedConfig) -> Self {
        Config {
            vpx_executable: resolved_config.vpx_executable.clone(),
            tables_folder: Some(resolved_config.tables_folder.clone()),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct ResolvedConfig {
    pub vpx_executable: PathBuf,
    pub tables_folder: PathBuf,
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

fn read_config(config_path: &PathBuf) -> io::Result<ResolvedConfig> {
    let figment = Figment::new().merge(Toml::file(&config_path));

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
    };
    Ok(resolved_config)
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
        print!("vpinball executale path: ");
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

    let resolved_config = ResolvedConfig {
        vpx_executable: vpx_executable,
        tables_folder: tables_root,
    };
    let config = Config::from(&resolved_config);

    // write config to config_file
    write_config(&config_file, &config)?;
    Ok((config_file, resolved_config))
}

fn write_config(config_file: &PathBuf, config: &Config) -> io::Result<()> {
    let toml = toml::to_string(&config).unwrap();
    let mut file = File::create(config_file)?;
    file.write_all(toml.as_bytes())?;
    Ok(())
}

pub fn default_tables_root(vpx_executable: &PathBuf) -> PathBuf {
    // There might be a reason to keep the tables in another directory ? Disk space for example.
    // Both Linux and MacOS is case sensitive, but Windows doesn't care. So we grab it from the executable name.
    vpx_executable.parent().unwrap().join("tables")
}

pub fn default_vpinball_executable(config: &Config) -> PathBuf {
    PathBuf::from(config.vpx_executable.clone())
}

fn default_vpinball_executable_detection() -> PathBuf {
    // TODO: Improve the executable detection.
    let mut local = env::current_dir().unwrap();

    if cfg!(target_os = "windows") {
        // baller installer default
        let dir = PathBuf::from("c:\\vPinball\\VisualPinball");
        let exe = dir.join("VPinballX64.exe");

        // Check current directory
        if local.join("VPinballX64.exe").exists() {
            local.join("VPinballX64.exe")
        } else if local.join("VPinballX.exe").exists() {
            local.join("VPinballX.exe")
        } else if exe.exists() {
            exe
        } else {
            dir.join("VPinballX.exe")
        }
    } else {
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
        assert_eq!(
            config,
            ResolvedConfig {
                vpx_executable: PathBuf::from("/tmp/test/vpinball"),
                tables_folder: PathBuf::from("/tmp/test/tables"),
            }
        );
        Ok(())
    }
}