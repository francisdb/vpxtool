use crate::config::SetupConfigResult;
use crate::indexer::{IndexError, Progress};
use base64::Engine;
use clap::{arg, Arg, ArgMatches, Command};
use colored::Colorize;
use console::Emoji;
use git_version::git_version;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::error::Error;
use std::fmt::Display;
use std::fs::{metadata, File};
use std::io;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, ExitCode};
use vpin::directb2s::read;
use vpin::pov::{Customsettings, ModePov, POV};
use vpin::vpx;
use vpin::vpx::math::{dequantize_unsigned, quantize_unsigned_percent};
use vpin::vpx::{expanded, extractvbs, importvbs, tableinfo, verify, ExtractResult, VerifyResult};

pub mod config;
pub mod fixprint;
mod frontend;
pub mod indexer;

// see https://github.com/fusion-engineering/rust-git-version/issues/21
const GIT_VERSION: &str = git_version!(args = ["--tags", "--always", "--dirty=-modified"]);

const OK: Emoji = Emoji("✅", "[launch]");
const NOK: Emoji = Emoji("❌", "[crash]");

const CMD_FRONTEND: &'static str = "frontend";
const CMD_DIFF: &'static str = "diff";

const CMD_SIMPLE_FRONTEND: &'static str = "simplefrontend";

const CMD_CONFIG: &'static str = "config";
const CMD_CONFIG_SETUP: &'static str = "setup";
const CMD_CONFIG_PATH: &'static str = "path";
const CMD_CONFIG_SHOW: &'static str = "show";
const CMD_CONFIG_CLEAR: &'static str = "clear";
const CMD_CONFIG_EDIT: &'static str = "edit";

const CMD_SCRIPT: &'static str = "script";
const CMD_SCRIPT_SHOW: &'static str = "show";
const CMD_SCRIPT_EDIT: &'static str = "edit";

pub struct ProgressBarProgress {
    pb: ProgressBar,
}
impl ProgressBarProgress {
    fn new(pb: ProgressBar) -> Self {
        Self { pb }
    }
}
impl Progress for ProgressBarProgress {
    fn set_length(&self, len: u64) {
        if len > 0 {
            self.pb.set_draw_target(ProgressDrawTarget::stdout());
        } else {
            self.pb.set_draw_target(ProgressDrawTarget::hidden());
        }
        self.pb.set_length(len)
    }
    fn set_position(&self, pos: u64) {
        self.pb.set_position(pos)
    }
    fn finish_and_clear(&self) {
        self.pb.finish_and_clear()
    }
}

pub fn run() -> io::Result<ExitCode> {
    let command = build_command();
    let matches = command.get_matches_from(wild::args());
    handle_command(matches)
}

fn handle_command(matches: ArgMatches) -> io::Result<ExitCode> {
    match matches.subcommand() {
        Some(("info", sub_matches)) => {
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = expand_path(path)?;
            println!("showing info for {}", expanded_path.display())?;
            let json = sub_matches.get_flag("JSON");
            info(&expanded_path, json).unwrap();
            Ok(ExitCode::SUCCESS)
        }
        Some((CMD_DIFF, sub_matches)) => {
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = expand_path(path)?;
            match diff_script(PathBuf::from(expanded_path)) {
                Ok(output) => {
                    println!("{}", output)?;
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    let warning = format!("Error running diff: {}", e).red();
                    println!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        Some((CMD_FRONTEND, _sub_matches)) => {
            let (config_path, config) = config::load_or_setup_config().unwrap();
            println!("Using config file {}", config_path.display())?;
            let roms = indexer::find_roms(&config.rom_folder()).unwrap();
            match frontend::frontend_index(&config, true) {
                Ok(tables) if tables.is_empty() => {
                    let warning =
                        format!("No tables found in {}", config.tables_folder.display()).red();
                    eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
                Ok(vpx_files_with_tableinfo) => {
                    let vpinball_executable = config.vpx_executable;
                    frontend::frontend(&vpx_files_with_tableinfo, &roms, &vpinball_executable);
                    Ok(ExitCode::SUCCESS)
                }
                Err(IndexError::FolderDoesNotExist(path)) => {
                    let warning = format!(
                        "Configured tables folder does not exist: {}",
                        path.display()
                    )
                    .red();
                    eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
                Err(IndexError::IoError(e)) => {
                    let warning = format!("Error running frontend: {}", e).red();
                    eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        Some((CMD_SIMPLE_FRONTEND, _sub_matches)) => {
            let (config_path, config) = config::load_or_setup_config().unwrap();
            println!("Using config file {}", config_path.display())?;
            let roms = indexer::find_roms(&config.rom_folder()).unwrap();
            match frontend::frontend_index(&config, true) {
                Ok(tables) if tables.is_empty() => {
                    let warning =
                        format!("No tables found in {}", config.tables_folder.display()).red();
                    eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
                Ok(vpx_files_with_tableinfo) => {
                    let vpinball_executable = config.vpx_executable;
                    frontend::frontend(&vpx_files_with_tableinfo, &roms, &vpinball_executable);
                    Ok(ExitCode::SUCCESS)
                }
                Err(IndexError::FolderDoesNotExist(path)) => {
                    let warning = format!(
                        "Configured tables folder does not exist: {}",
                        path.display()
                    )
                    .red();
                    eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
                Err(IndexError::IoError(e)) => {
                    let warning = format!("Error running frontend: {}", e).red();
                    eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        Some(("index", sub_matches)) => {
            let recursive = sub_matches.get_flag("RECURSIVE");
            let path = sub_matches
                .get_one::<String>("VPXROOTPATH")
                .map(|s| s.as_str());

            let (tables_folder_path, tables_index_path) = match path {
                Some(path) => {
                    let tables_path = expand_path(path)?;
                    let tables_index_path = config::tables_index_path(&tables_path);
                    (tables_path, tables_index_path)
                }
                None => match config::load_config().unwrap() {
                    Some((config_path, config)) => {
                        println!("Using config file {}", config_path.display())?;
                        (config.tables_folder, config.tables_index_path)
                    }
                    None => {
                        eprintln!("No VPXROOTPATH provided up and no config file found")?;
                        exit(1);
                    }
                },
            };
            let pb = ProgressBar::hidden();
            pb.set_style(
                ProgressStyle::with_template(
                    "{spinner:.green} [{bar:.cyan/blue}] {pos}/{human_len} ({eta})",
                )
                .unwrap(),
            );
            let progress = ProgressBarProgress::new(pb);
            let index = indexer::index_folder(
                recursive,
                &tables_folder_path,
                &tables_index_path,
                &progress,
            )
            .unwrap();
            progress.finish_and_clear();
            println!(
                "Indexed {} vpx files into {}",
                index.len(),
                &tables_index_path.display()
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Some((CMD_SCRIPT, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_SCRIPT_SHOW, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("VPXPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();

                let expanded_path = expand_path(path)?;
                let mut vpx_file = vpx::open(&expanded_path)?;
                let game_data = vpx_file.read_gamedata()?;
                let code = game_data.code.string;

                println!("{}", code)?;
                Ok(ExitCode::SUCCESS)
            }
            Some((CMD_SCRIPT_EDIT, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("VPXPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();

                let expanded_vpx_path = expand_path(path)?;

                let vbs_path = vpx::vbs_path_for(&expanded_vpx_path);
                if vbs_path.exists() {
                    open_or_fail(&vbs_path)
                } else {
                    extractvbs(&expanded_vpx_path, false, None);
                    open_or_fail(&vbs_path)
                }
            }
            _ => unreachable!(),
        },
        Some(("ls", sub_matches)) => {
            let path = sub_matches
                .get_one::<String>("VPXPATH")
                .map(|s| s.as_str())
                .unwrap_or_default();

            let expanded_path = expand_path(path)?;
            ls(expanded_path.as_ref())?;
            Ok(ExitCode::SUCCESS)
        }
        Some(("extract", sub_matches)) => {
            let yes = sub_matches.get_flag("FORCE");
            let paths: Vec<&str> = sub_matches
                .get_many::<String>("VPXPATH")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect();
            for path in paths {
                let expanded_path = expand_path(path)?;
                println!("extracting from {}", expanded_path.display())?;
                if expanded_path.ends_with(".directb2s") {
                    extract_directb2s(&expanded_path).unwrap();
                } else {
                    extract(expanded_path.as_ref(), yes).unwrap();
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Some(("extractvbs", sub_matches)) => {
            let overwrite = sub_matches.get_flag("OVERWRITE");
            let paths: Vec<&str> = sub_matches
                .get_many::<String>("VPXPATH")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();
            for path in paths {
                let expanded_path = PathBuf::from(expand_path(path)?);
                match extractvbs(&expanded_path, overwrite, None) {
                    ExtractResult::Existed(vbs_path) => {
                        let warning =
                            format!("EXISTED {}", vbs_path.display()).truecolor(255, 125, 0);
                        println!("{}", warning)?;
                    }
                    ExtractResult::Extracted(vbs_path) => {
                        println!("CREATED {}", vbs_path.display())?;
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Some(("importvbs", sub_matches)) => {
            let path: &str = sub_matches.get_one::<String>("VPXPATH").unwrap().as_str();
            let expanded_path = PathBuf::from(expand_path(path)?);
            match importvbs(&expanded_path, None) {
                Ok(vbs_path) => {
                    println!("IMPORTED {}", vbs_path.display())?;
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    let warning = format!("Error importing vbs: {}", e).red();
                    eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
            }
        }

        Some(("verify", sub_matches)) => {
            let paths: Vec<&str> = sub_matches
                .get_many::<String>("VPXPATH")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();
            for path in paths {
                let expanded_path = PathBuf::from(expand_path(path)?);
                match verify(&expanded_path) {
                    VerifyResult::Ok(vbs_path) => {
                        println!("{OK} {}", vbs_path.display())?;
                    }
                    VerifyResult::Failed(vbs_path, msg) => {
                        let warning =
                            format!("{NOK} {} {}", vbs_path.display(), msg).truecolor(255, 125, 0);
                        eprintln!("{}", warning)?;
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Some(("new", sub_matches)) => {
            let path = {
                let this = sub_matches.get_one::<String>("VPXPATH").map(|v| v.as_str());
                match this {
                    Some(x) => x,
                    None => unreachable!("VPXPATH is required"),
                }
            };

            let expanded_path = shellexpand::tilde(path);
            println!("creating new vpx file at {}", expanded_path)?;
            new(expanded_path.as_ref())?;
            Ok(ExitCode::SUCCESS)
        }
        Some((CMD_CONFIG, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_CONFIG_SETUP, _)) => match config::setup_config() {
                Ok(SetupConfigResult::Configured(config_path)) => {
                    println!("Created config file {}", config_path.display())?;
                    Ok(ExitCode::SUCCESS)
                }
                Ok(SetupConfigResult::Existing(config_path)) => {
                    println!("Config file already exists at {}", config_path.display())?;
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    eprintln!("Failed to create config file: {}", e)?;
                    Ok(ExitCode::FAILURE)
                }
            },
            Some((CMD_CONFIG_PATH, _)) => match config::config_path() {
                Some(config_path) => {
                    println!("{}", config_path.display())?;
                    Ok(ExitCode::SUCCESS)
                }
                None => {
                    eprintln!("No config file found")?;
                    Ok(ExitCode::FAILURE)
                }
            },
            Some((CMD_CONFIG_SHOW, _)) => match config::config_path() {
                Some(config_path) => {
                    let mut file = File::open(&config_path).unwrap();
                    let mut text = String::new();
                    file.read_to_string(&mut text).unwrap();
                    println!("{}", text)?;
                    Ok(ExitCode::SUCCESS)
                }
                None => {
                    eprintln!("No config file found")?;
                    Ok(ExitCode::FAILURE)
                }
            },
            Some((CMD_CONFIG_CLEAR, _)) => match config::clear_config() {
                Ok(Some(config_path)) => {
                    println!("Cleared config file {}", config_path.display())?;
                    Ok(ExitCode::SUCCESS)
                }
                Ok(None) => {
                    println!("No config file found")?;
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => fail_with_error("Failed to clear config file: {}", e),
            },
            Some((CMD_CONFIG_EDIT, _)) => match config::config_path() {
                Some(config_path) => {
                    let editor = std::env::var("EDITOR").expect("EDITOR not set");
                    let status = std::process::Command::new(editor)
                        .arg(config_path)
                        .status()
                        .unwrap();
                    if !status.success() {
                        fail("Failed to edit config file")
                    } else {
                        Ok(ExitCode::SUCCESS)
                    }
                }
                None => fail("No config file found"),
            },
            _ => unreachable!(),
        },
        Some(("pov", sub_matches)) => match sub_matches.subcommand() {
            Some(("extract", sub_matches)) => {
                let paths: Vec<&str> = sub_matches
                    .get_many::<String>("VPXPATH")
                    .unwrap_or_default()
                    .map(|v| v.as_str())
                    .collect::<Vec<_>>();
                for path in paths {
                    let expanded_path = PathBuf::from(expand_path(path)?);
                    match extract_pov(&expanded_path) {
                        Ok(pov_path) => {
                            println!("CREATED {}", pov_path.display()).unwrap();
                        }
                        Err(err) => {
                            fail_with_error("Error extracting pov", err).unwrap();
                        }
                    }
                }
                Ok(ExitCode::SUCCESS)
            }
            _ => unreachable!(),
        },
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn build_command() -> Command {
    // to allow for non static strings in clap
    // I had to enable the "string" module
    // is this considered a bad idea?
    Command::new("vpxtool")
        .version(GIT_VERSION)
        .author("Francis DB")
        .about("Extracts and assembles vpx files")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("info")
                .about("Show information about a vpx file")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true))
                .arg(
                    Arg::new("JSON")
                        .short('j')
                        .long("json")
                        .num_args(0)
                        .help("Output as JSON"),
                ),
        )
        .subcommand(
            Command::new(CMD_DIFF)
                .about("Prints out a diff between the vbs in the vpx and the sidecar vbs")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true))
        )
        .subcommand(
            Command::new(CMD_FRONTEND)
                .about("Text based frontend for launching vpx files")
                .arg(
                    Arg::new("RECURSIVE")
                        .short('r')
                        .long("recursive")
                        .num_args(0)
                        .help("Recursively index subdirectories")
                        .default_value("true"),
                )
        )
        .subcommand(
            Command::new(CMD_SIMPLE_FRONTEND)
                .about("Simple text based frontend for launching vpx files")
                .arg(
                    Arg::new("RECURSIVE")
                        .short('r')
                        .long("recursive")
                        .num_args(0)
                        .help("Recursively index subdirectories")
                        .default_value("true"),
                )
        )
        .subcommand(
            Command::new("index")
                .about("Indexes a directory of vpx files")
                .arg(
                    Arg::new("RECURSIVE")
                        .short('r')
                        .long("recursive")
                        .num_args(0)
                        .help("Recursively index subdirectories")
                        .default_value("true"),
                )
                .arg(
                    arg!(<VPXROOTPATH> "The path to the root directory of vpx files. Defaults to what is set up in the config file.")
                        .required(false)
                ),
        )
        .subcommand(
            Command::new(CMD_SCRIPT)
                .subcommand_required(true)
                .about("Vpx script code related commands")
                .subcommand(
                    Command::new(CMD_SCRIPT_SHOW)
                        .about("Show a vpx script")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        ),
                )
                .subcommand(
                    Command::new(CMD_SCRIPT_EDIT)
                        .about("Edit the table vpx script")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        ),
                )
        )
        .subcommand(
            Command::new("ls")
                .about("Show a vpx file content")
                .arg(
                    arg!(<VPXPATH> "The path to the vpx file")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("extract")
                .about("Extracts a vpx file")
                .arg(
                    Arg::new("FORCE")
                        .short('f')
                        .long("force")
                        .num_args(0)
                        .help("Do not ask for confirmation before overwriting existing files"),
                )
                .arg(
                    arg!(<VPXPATH> "The path(s) to the vpx file(s)")
                        .required(true)
                        .num_args(1..),
                ),
        )
        .subcommand(
            Command::new("extractvbs")
                .about("Extracts the vbs from a vpx file next to it")
                .arg(
                    Arg::new("OVERWRITE")
                        .short('o')
                        .long("overwrite")
                        .num_args(0)
                        .default_value("false")
                        .help("(Default: false) Will overwrite existing .vbs files if true, will skip the table file if false."),
                )
                .arg(
                    arg!(<VPXPATH> "The path(s) to the vpx file(s)")
                        .required(true)
                        .num_args(1..),
                ),
        )
        .subcommand(
            Command::new("importvbs")
                .about("Imports the vbs next to it into a vpx file")
                .arg(
                    arg!(<VPXPATH> "The path(s) to the vpx file(s)")
                        .required(true)
                        .num_args(1..),
                ),
        )
        .subcommand(
            Command::new("verify")
                .about("Verify the structure of a vpx file")
                .arg(
                    arg!(<VPXPATH> "The path(s) to the vpx file(s)")
                        .required(true)
                        .num_args(1..),
                ),
        )
        .subcommand(
            Command::new("pov")
                .subcommand_required(true)
                .about("Point of view file (pov) related commands")
                .subcommand(Command::new("extract")
                                .about("Extracts a the pov file from a vpx file")
                                .arg(
                                    arg!(<VPXPATH> "The path(s) to the vpx file(s)")
                                        .required(true)
                                        .num_args(1..),
                                ),
                )
        )
        .subcommand(
            Command::new("assemble")
                .about("Assembles a vpx file")
                .arg(arg!(<DIRPATH> "The path to the vpx structure").required(true)),
        )
        .subcommand(
            Command::new("new")
                .about("Creates a minimal empty new vpx file")
                .arg(arg!(<VPXPATH> "The path(s) to the vpx file").required(true)),
        )
        .subcommand(
            Command::new(CMD_CONFIG)
                .subcommand_required(true)
                .about("Vpxtool related config file")
                .subcommand(
                    Command::new(CMD_CONFIG_SETUP)
                        .about("Sets up the config file"),
                )
                .subcommand(
                    Command::new(CMD_CONFIG_PATH)
                        .about("Shows the current config file path"),
                )
                .subcommand(
                    Command::new(CMD_CONFIG_CLEAR)
                        .about("Clears the current config file"),
                )
                .subcommand(
                    Command::new(CMD_CONFIG_SHOW)
                        .about("Shows the contents of the config file"),
                )
                .subcommand(
                    Command::new(CMD_CONFIG_EDIT)
                        .about("Edits the config file using the default editor"),
                )
        )
}

fn open_or_fail(vbs_path: &PathBuf) -> io::Result<ExitCode> {
    match open::that(&vbs_path) {
        Ok(_) => Ok(ExitCode::SUCCESS),
        Err(err) => {
            let msg = format!("Unable to open {}", vbs_path.to_string_lossy());
            fail_with_error(msg, err)
        }
    }
}

fn fail_with_error(message: impl Display, err: impl Error) -> io::Result<ExitCode> {
    let warning = format!("{message}: {err}");
    fail(warning)
}

fn fail<M: AsRef<str>>(message: M) -> io::Result<ExitCode> {
    let warning = message.as_ref().red();
    eprintln!("{}", warning)?;
    Ok(ExitCode::FAILURE)
}

fn new(vpx_file_path: &str) -> io::Result<()> {
    // TODO check if file exists and prompt to overwrite / add option to force
    vpx::new_minimal_vpx(vpx_file_path)
}

fn extract_pov(vpx_path: &PathBuf) -> io::Result<PathBuf> {
    // read vpx
    // create pov
    // write pov

    let mut vpx_file = vpin::vpx::open(vpx_path)?;
    let table_info = vpx_file.read_gamedata()?;

    let pov = POV {
        desktop: ModePov {
            layout_mode: table_info.bg_view_mode_desktop,
            inclination: table_info.bg_view_mode_desktop.unwrap_or_default() as f32,
            fov: table_info.bg_fov_desktop,
            layback: table_info.bg_layback_desktop,
            lookat: Some(table_info.bg_inclination_desktop),
            rotation: table_info.bg_rotation_desktop,
            xscale: table_info.bg_scale_x_desktop,
            yscale: table_info.bg_scale_y_desktop,
            zscale: table_info.bg_scale_z_desktop,
            xoffset: table_info.bg_offset_x_desktop,
            yoffset: table_info.bg_offset_y_desktop,
            zoffset: table_info.bg_offset_z_desktop,
            view_hofs: table_info.bg_view_horizontal_offset_desktop,
            view_vofs: table_info.bg_view_vertical_offset_desktop,
            window_top_xofs: table_info.bg_window_top_x_offset_desktop,
            window_top_yofs: table_info.bg_window_top_y_offset_desktop,
            window_top_zofs: table_info.bg_window_top_z_offset_desktop,
            window_bottom_xofs: table_info.bg_window_bottom_x_offset_desktop,
            window_bottom_yofs: table_info.bg_window_bottom_y_offset_desktop,
            window_bottom_zofs: table_info.bg_window_bottom_z_offset_desktop,
        },
        fullscreen: ModePov {
            layout_mode: table_info.bg_view_mode_fullscreen,
            inclination: table_info.bg_view_mode_fullscreen.unwrap_or_default() as f32,
            fov: table_info.bg_fov_fullscreen,
            layback: table_info.bg_layback_fullscreen,
            lookat: Some(table_info.bg_inclination_fullscreen),
            rotation: table_info.bg_rotation_fullscreen,
            xscale: table_info.bg_scale_x_fullscreen,
            yscale: table_info.bg_scale_y_fullscreen,
            zscale: table_info.bg_scale_z_fullscreen,
            xoffset: table_info.bg_offset_x_fullscreen,
            yoffset: table_info.bg_offset_y_fullscreen,
            zoffset: table_info.bg_offset_z_fullscreen,
            view_hofs: table_info.bg_view_horizontal_offset_fullscreen,
            view_vofs: table_info.bg_view_vertical_offset_fullscreen,
            window_top_xofs: table_info.bg_window_top_x_offset_fullscreen,
            window_top_yofs: table_info.bg_window_top_y_offset_fullscreen,
            window_top_zofs: table_info.bg_window_top_z_offset_fullscreen,
            window_bottom_xofs: table_info.bg_window_bottom_x_offset_fullscreen,
            window_bottom_yofs: table_info.bg_window_bottom_y_offset_fullscreen,
            window_bottom_zofs: table_info.bg_window_bottom_z_offset_fullscreen,
        },
        fullsinglescreen: ModePov {
            // TODO fix these defaults
            layout_mode: table_info.bg_view_mode_full_single_screen,
            inclination: table_info
                .bg_view_mode_full_single_screen
                .unwrap_or_default() as f32,
            fov: table_info.bg_fov_full_single_screen.unwrap_or_default(),
            layback: table_info.bg_layback_full_single_screen.unwrap_or_default(),
            lookat: table_info.bg_inclination_full_single_screen,
            rotation: table_info
                .bg_rotation_full_single_screen
                .unwrap_or_default(),
            xscale: table_info.bg_scale_x_full_single_screen.unwrap_or_default(),
            yscale: table_info.bg_scale_y_full_single_screen.unwrap_or_default(),
            zscale: table_info.bg_scale_z_full_single_screen.unwrap_or_default(),
            xoffset: table_info
                .bg_offset_x_full_single_screen
                .unwrap_or_default(),
            yoffset: table_info
                .bg_offset_y_full_single_screen
                .unwrap_or_default(),
            zoffset: table_info
                .bg_offset_z_full_single_screen
                .unwrap_or_default(),
            view_hofs: table_info.bg_view_horizontal_offset_full_single_screen,
            view_vofs: table_info.bg_view_vertical_offset_full_single_screen,
            window_top_xofs: table_info.bg_window_top_x_offset_full_single_screen,
            window_top_yofs: table_info.bg_window_top_y_offset_full_single_screen,
            window_top_zofs: table_info.bg_window_top_z_offset_full_single_screen,
            window_bottom_xofs: table_info.bg_window_bottom_x_offset_full_single_screen,
            window_bottom_yofs: table_info.bg_window_bottom_y_offset_full_single_screen,
            window_bottom_zofs: table_info.bg_window_bottom_z_offset_full_single_screen,
        },
        customsettings: Some(Customsettings {
            ssaa: table_info.use_aal,
            postproc_aa: table_info.use_fxaa,
            ingame_ao: table_info.use_ao,
            sc_sp_reflect: table_info.use_ssr.unwrap_or(-1),
            fps_limiter: table_info.table_adaptive_vsync,
            overwrite_details_level: table_info.overwrite_global_detail_level as u32,
            details_level: table_info.user_detail_level,
            ball_reflection: table_info.use_reflection_for_balls,
            ball_trail: table_info.use_trail_for_balls,
            ball_trail_strength: dequantize_unsigned(8, table_info.ball_trail_strength),
            overwrite_night_day: table_info.overwrite_global_day_night as u32,
            night_day_level: quantize_unsigned_percent(table_info.global_emission_scale),
            gameplay_difficulty: table_info.global_difficulty * 100.0,
            physics_set: table_info.override_physics,
            include_flipper_physics: table_info.override_physics_flipper.unwrap_or(false) as u32,
            sound_volume: quantize_unsigned_percent(table_info.table_sound_volume),
            sound_music_volume: quantize_unsigned_percent(table_info.table_music_volume),
        }),
    };

    let pov_path = vpx_path.with_extension("pov");
    vpin::pov::save(&pov_path, &pov)?;

    Ok(pov_path)
}

fn extract_directb2s(expanded_path: &PathBuf) -> io::Result<()> {
    let file = File::open(expanded_path).unwrap();
    let reader = BufReader::new(file);
    match read(reader) {
        Ok(b2s) => {
            println!("DirectB2S file version {}", b2s.version)?;
            let root_dir_path = expanded_path.with_extension("directb2s.extracted");

            let mut root_dir = std::fs::DirBuilder::new();
            root_dir.recursive(true);
            root_dir.create(&root_dir_path).unwrap();

            println!("Writing to {}", root_dir_path.display())?;
            wite_images(b2s, root_dir_path.as_path());
        }
        Err(msg) => {
            println!("Failed to load {}: {}", expanded_path.display(), msg)?;
            exit(1);
        }
    }
    Ok(())
}

fn wite_images(b2s: vpin::directb2s::DirectB2SData, root_dir_path: &Path) {
    if let Some(backglass_off_image) = b2s.images.backglass_off_image {
        write_base64_to_file(
            root_dir_path,
            None,
            "backglassimage.img".to_string(),
            &backglass_off_image.value,
        );
    }
    if let Some(backglass_on_image) = b2s.images.backglass_on_image {
        write_base64_to_file(
            root_dir_path,
            Some(backglass_on_image.file_name),
            "backglassimage.img".to_string(),
            &backglass_on_image.value,
        );
    }
    if let Some(backglass_image) = b2s.images.backglass_image {
        write_base64_to_file(
            root_dir_path,
            Some(backglass_image.file_name),
            "backglassimage.img".to_string(),
            &backglass_image.value,
        );
    }

    if let Some(dmd_image) = b2s.images.dmd_image {
        write_base64_to_file(
            root_dir_path,
            Some(dmd_image.file_name),
            "dmdimage.img".to_string(),
            &dmd_image.value,
        );
    }
    if let Some(illumination_image) = b2s.images.illumination_image {
        write_base64_to_file(
            root_dir_path,
            None,
            "dmdimage.img".to_string(),
            &illumination_image.value,
        );
    }

    let thumbnail_image = b2s.images.thumbnail_image;
    write_base64_to_file(
        root_dir_path,
        None,
        "thumbnailimage.png".to_string(),
        &thumbnail_image.value,
    );

    for bulb in b2s.illumination.bulb.unwrap_or_default() {
        write_base64_to_file(
            root_dir_path,
            None,
            format!("{}.png", bulb.name).to_string(),
            &bulb.image,
        );
        if let Some(off_image) = bulb.off_image {
            write_base64_to_file(
                root_dir_path,
                None,
                format!("{}_off.png", bulb.name).to_string(),
                &off_image,
            );
        }
    }

    if let Some(reel) = b2s.reels {
        for reels_image in reel.images.image.iter().flatten() {
            write_base64_to_file(
                root_dir_path,
                None,
                format!("{}.png", reels_image.name).to_string(),
                &reels_image.image,
            );
        }
        for illuminated_set in reel.illuminated_images.set.iter().flatten() {
            for reels_image in &illuminated_set.illuminated_image {
                write_base64_to_file(
                    root_dir_path,
                    None,
                    format!("{}.png", reels_image.name).to_string(),
                    &reels_image.image,
                );
            }
        }
    }
}

fn write_base64_to_file(
    root_dir_path: &Path,
    original_file_path: Option<String>,
    default: String,
    base64data_with_cr_lf: &str,
) {
    // TODO bring in the other default here
    let file_name: String =
        os_independent_file_name(original_file_path.unwrap_or(default.clone())).unwrap_or(default);

    let file_path = root_dir_path.join(file_name);

    let mut file = File::create(file_path).unwrap();
    let base64data = strip_cr_lf(base64data_with_cr_lf);

    let decoded_data = base64::engine::general_purpose::STANDARD
        .decode(base64data)
        .unwrap();
    file.write_all(&decoded_data).unwrap();
}

fn strip_cr_lf(s: &str) -> String {
    s.chars().filter(|c| !c.is_ascii_whitespace()).collect()
}

fn os_independent_file_name(file_path: String) -> Option<String> {
    // we can't use path here as this uses the system path encoding
    // we might have to parse windows paths on mac/linux
    file_path
        .rsplit(|c| c == '/' || c == '\\')
        .next()
        .map(|f| f.to_string())
}

fn expand_path(path: &str) -> io::Result<PathBuf> {
    // TODO expand all instead of only tilde?
    let expanded_path = shellexpand::tilde(path);
    match metadata(expanded_path.as_ref()) {
        Ok(md) => {
            if !md.is_file() && !md.is_dir() && md.is_symlink() {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{} is not a file", expanded_path),
                ))
            } else {
                Ok(PathBuf::from(expanded_path.to_string()))
            }
        }
        Err(msg) => {
            let warning = format!("Failed to read metadata for {}: {}", expanded_path, msg);
            Err(io::Error::new(io::ErrorKind::InvalidInput, warning))
        }
    }
}

fn info(vpx_file_path: &PathBuf, json: bool) -> io::Result<()> {
    let mut vpx_file = vpx::open(vpx_file_path)?;
    let version = vpx_file.read_version()?;
    // GameData also has a name field that we might want to display here
    // where is this shown in the UI?
    let table_info = vpx_file.read_tableinfo()?;
    // TODO check the json flag

    println!("{:>18} {}", "VPX Version:".green(), version)?;
    println!(
        "{:>18} {}",
        "Table Name:".green(),
        table_info.table_name.unwrap_or("[not set]".to_string())
    )?;
    println!(
        "{:>18} {}",
        "Version:".green(),
        table_info.table_version.unwrap_or("[not set]".to_string())
    )?;
    println!(
        "{:>18} {}{}{}",
        "Author:".green(),
        Some(table_info.author_name)
            .map(|s| s.unwrap_or("[not set]".to_string()))
            .filter(|s| !s.is_empty())
            .map(|s| format!("{} ", s))
            .unwrap_or_default(),
        Some(table_info.author_email)
            .map(|s| s.unwrap_or("[not set]".to_string()))
            .filter(|s| !s.is_empty())
            .map(|s| format!("{} ", s))
            .unwrap_or_default(),
        Some(table_info.author_website)
            .map(|s| s.unwrap_or("[not set]".to_string()))
            .filter(|s| !s.is_empty())
            .map(|s| format!("{} ", s))
            .unwrap_or_default(),
    )?;
    println!(
        "{:>18} {}",
        "Save revision:".green(),
        table_info.table_save_rev.unwrap_or("[not set]".to_string())
    )?;
    println!(
        "{:>18} {}",
        "Save date:".green(),
        table_info
            .table_save_date
            .unwrap_or("[not set]".to_string())
    )?;
    println!(
        "{:>18} {}",
        "Release Date:".green(),
        table_info.release_date.unwrap_or("[not set]".to_string())
    )?;
    println!(
        "{:>18} {}",
        "Description:".green(),
        table_info
            .table_description
            .unwrap_or("[not set]".to_string())
    )?;
    println!(
        "{:>18} {}",
        "Blurb:".green(),
        table_info.table_blurb.unwrap_or("[not set]".to_string())
    )?;
    println!(
        "{:>18} {}",
        "Rules:".green(),
        table_info.table_rules.unwrap_or("[not set]".to_string())
    )?;
    // other properties
    table_info
        .properties
        .iter()
        .map(|(prop, value)| println!("{:>18}: {}", prop.green(), value))
        .collect::<io::Result<()>>()?;

    Ok(())
}

pub fn ls(vpx_file_path: &Path) -> io::Result<()> {
    expanded::extract_directory_list(vpx_file_path)
        .iter()
        .map(|file_path| println!("{}", file_path))
        .collect()
}

pub fn extract(vpx_file_path: &Path, yes: bool) -> io::Result<ExitCode> {
    let root_dir_path_str = vpx_file_path.with_extension("");
    let root_dir_path = Path::new(&root_dir_path_str);

    let mut root_dir = std::fs::DirBuilder::new();
    root_dir.recursive(true);
    // ask for confirmation if the directory exists
    if root_dir_path.exists() && !yes {
        // TODO do we need to check for terminal here?
        //   let use_color = stdout().is_terminal();
        let warning =
            format!("Directory {} already exists", root_dir_path.display()).truecolor(255, 125, 0);
        println!("{}", warning)?;
        println!("Do you want to continue exporting? (y/n)")?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim() != "y" {
            println!("Aborting")?;
            Ok(ExitCode::SUCCESS)
        } else {
            match expanded::extract(vpx_file_path, root_dir_path) {
                Ok(_) => {
                    println!("Successfully extracted to {}", root_dir_path.display())?;
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    println!("Failed to extract: {}", e)?;
                    Ok(ExitCode::FAILURE)
                }
            }
        }
    } else {
        match expanded::extract(vpx_file_path, root_dir_path) {
            Ok(_) => {
                println!("Successfully extracted to {}", root_dir_path.display())?;
                Ok(ExitCode::SUCCESS)
            }
            Err(e) => {
                println!("Failed to extract: {}", e)?;
                Ok(ExitCode::FAILURE)
            }
        }
    }
}

pub fn diff_script<P: AsRef<Path>>(vpx_file_path: P) -> io::Result<String> {
    // set extension for PathBuf
    let vbs_path = vpx_file_path.as_ref().with_extension("vbs");
    let original_vbs_path = vpx_file_path.as_ref().with_extension("vbs.original.tmp");

    if vbs_path.exists() {
        match vpx::open(&vpx_file_path) {
            Ok(mut vpx_file) => {
                let gamedata = vpx_file.read_gamedata()?;
                let script = gamedata.code;
                std::fs::write(&original_vbs_path, script.string)?;
                let diff_color = if colored::control::SHOULD_COLORIZE.should_colorize() {
                    DiffColor::Always
                } else {
                    DiffColor::Never
                };
                let output = run_diff(&original_vbs_path, &vbs_path, diff_color)?;

                if original_vbs_path.exists() {
                    std::fs::remove_file(original_vbs_path)?;
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
