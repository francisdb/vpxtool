use crate::config::{ResolvedConfig, SetupConfigResult};
use crate::indexer::{DEFAULT_INDEX_FILE_NAME, IndexError, Progress};
use crate::patcher::patch_vbs_file;
use crate::{
    RemoveOnDrop, config, expand_path, expand_path_exists, frontend, indexer,
    os_independent_file_name, path_exists, strip_cr_lf,
};
use base64::Engine;
use clap::builder::Str;
use clap::{Arg, ArgAction, ArgMatches, Command, arg};
use colored::Colorize;
use console::Emoji;
use env_logger::Env;
use git_version::git_version;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use log::LevelFilter;
use pinmame_nvram::dips::get_all_dip_switches;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{ExitCode, exit};
use std::sync::atomic::{AtomicBool, Ordering};
use vpin::directb2s::read;
use vpin::vpx;
use vpin::vpx::jsonmodel::{game_data_to_json, info_to_json};
use vpin::vpx::{ExtractResult, VerifyResult, expanded, extractvbs, importvbs, verify};

// see https://github.com/fusion-engineering/rust-git-version/issues/21
const GIT_VERSION: &str = git_version!(
    args = ["--tags", "--always", "--dirty=-modified"],
    prefix = "git:",
    cargo_prefix = "cargo:",
    fallback = "unknown"
);

const OK: Emoji = Emoji("✅", "[launch]");
const NOK: Emoji = Emoji("❌", "[crash]");

const CMD_FRONTEND: &str = "frontend";
const CMD_DIFF: &str = "diff";
const CMD_EXTRACT: &str = "extract";
const CMD_ASSEMBLE: &str = "assemble";
const CMD_EXTRACT_VBS: &str = "extractvbs";
const CMD_IMPORT_VBS: &str = "importvbs";
const CMD_PATCH: &str = "patch";
const CMD_VERIFY: &str = "verify";
const CMD_NEW: &str = "new";

const CMD_LS: &str = "ls";

const CMD_CONFIG: &str = "config";
const CMD_CONFIG_SETUP: &str = "setup";
const CMD_CONFIG_PATH: &str = "path";
const CMD_CONFIG_SHOW: &str = "show";
const CMD_CONFIG_CLEAR: &str = "clear";
const CMD_CONFIG_EDIT: &str = "edit";

const CMD_SCRIPT: &str = "script";
const CMD_SCRIPT_SHOW: &str = "show";
const CMD_SCRIPT_EXTRACT: &str = "extract";
const CMD_SCRIPT_IMPORT: &str = "import";
const CMD_SCRIPT_PATCH: &str = "patch";
const CMD_SCRIPT_EDIT: &str = "edit";
const CMD_SCRIPT_DIFF: &str = "diff";

const CMD_INFO: &str = "info";
const CMD_INFO_SHOW: &str = "show";
const CMD_INFO_EXTRACT: &str = "extract";
const CMD_INFO_IMPORT: &str = "import";
const CMD_INFO_EDIT: &str = "edit";
const CMD_INFO_DIFF: &str = "diff";

const CMD_IMAGES: &str = "images";
const CMD_IMAGES_WEBP: &str = "webp";

const CMD_GAMEDATA: &str = "gamedata";
const CMD_GAMEDATA_SHOW: &str = "show";

const CMD_DIPSWITCHES: &str = "dipswitches";
const CMD_DIPSWITCHES_SHOW: &str = "show";

const CMD_ROMNAME: &str = "romname";

const CMD_INDEX: &str = "index";
const ARG_VERBOSE: &str = "VERBOSE";
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub(crate) struct ProgressBarProgress {
    pb: ProgressBar,
}

impl ProgressBarProgress {
    pub(crate) fn new(pb: ProgressBar) -> Self {
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

    let verbose = matches.get_flag(ARG_VERBOSE);
    VERBOSE.store(verbose, Ordering::Relaxed);
    if verbose {
        init_logging();
    }

    handle_command(matches)
}

fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let mut builder = env_logger::Builder::from_env(Env::default());
        builder
            .filter_level(LevelFilter::Warn)
            .filter_module("vpin", LevelFilter::Info)
            .filter_module("vpxtool", LevelFilter::Info)
            .init();
    });
}

fn handle_command(matches: ArgMatches) -> io::Result<ExitCode> {
    match matches.subcommand() {
        Some((CMD_INFO, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_INFO_SHOW, sub_matches)) => {
                let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
                let path = path.unwrap_or("");
                let expanded_path = expand_path_exists(path)?;
                crate::println!("showing info for {}", expanded_path.display())?;
                let info = info_gather(&expanded_path)?;
                crate::println!("{}", info)?;
                Ok(ExitCode::SUCCESS)
            }
            Some((CMD_INFO_EXTRACT, sub_matches)) => {
                let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
                let path = path.unwrap_or("");
                let expanded_path = expand_path_exists(path)?;
                crate::println!("extracting info for {}", expanded_path.display())?;
                info_extract(&expanded_path)
            }
            Some((CMD_INFO_IMPORT, sub_matches)) => {
                let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
                let path = path.unwrap_or("");
                let expanded_path = expand_path_exists(path)?;
                crate::println!("importing info for {}", expanded_path.display())?;
                info_import(&expanded_path)
            }
            Some((CMD_INFO_EDIT, sub_matches)) => {
                let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
                let path = path.unwrap_or("");
                let expanded_path = expand_path_exists(path)?;
                let loaded_config = config::load_config()?;
                let config = loaded_config.as_ref().map(|c| &c.1);
                crate::println!("editing info for {}", expanded_path.display())?;
                info_edit(&expanded_path, config)?;
                Ok(ExitCode::SUCCESS)
            }
            Some((CMD_INFO_DIFF, sub_matches)) => {
                let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
                let path = path.unwrap_or("");
                let expanded_path = expand_path_exists(path)?;
                crate::println!("diffing info for {}", expanded_path.display())?;
                let diff = info_diff(&expanded_path)?;
                crate::println!("{}", diff)?;
                Ok(ExitCode::SUCCESS)
            }
            _ => unreachable!(),
        },
        Some((CMD_DIFF, sub_matches)) => {
            // TODO this should diff more than only the script
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = expand_path_exists(path)?;
            match script_diff(&expanded_path) {
                Ok(output) => {
                    crate::println!("{}", output)?;
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    let warning = format!("Error running diff: {e}").red();
                    crate::println!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        Some((CMD_FRONTEND, _sub_matches)) => {
            let (config_path, config) = config::load_or_setup_config()?;
            crate::println!("Using vpxtool config file {}", config_path.display())?;
            crate::println!("Using vpinball config file {}", config.vpx_config.display())?;
            crate::println!(
                "Using global pinmame folder {}",
                config.global_pinmame_folder().display()
            )?;
            crate::println!(
                "Using configured pinmame folder {}",
                config
                    .configured_pinmame_folder()
                    .map(|f| f.display().to_string())
                    .unwrap_or_else(|| "None".to_string())
            )?;
            match frontend::frontend_index(&config, true, vec![]) {
                Ok(tables) if tables.is_empty() => {
                    let warning =
                        format!("No tables found in {}", config.tables_folder.display()).red();
                    crate::eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
                Ok(vpx_files_with_tableinfo) => {
                    frontend::frontend(&config, vpx_files_with_tableinfo);
                    Ok(ExitCode::SUCCESS)
                }
                Err(IndexError::FolderDoesNotExist(path)) => {
                    let warning = format!(
                        "Configured tables folder does not exist: {}",
                        path.display()
                    )
                    .red();
                    crate::eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
                Err(IndexError::IoError(e)) => {
                    let warning = format!("Error running frontend: {e}").red();
                    crate::eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        Some((CMD_INDEX, sub_matches)) => handle_index(sub_matches),
        Some((CMD_SCRIPT, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_SCRIPT_SHOW, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("VPXPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();

                let expanded_path = expand_path_exists(path)?;
                let mut vpx_file = vpx::open(expanded_path)?;
                let game_data = vpx_file.read_gamedata()?;
                let code = game_data.code.string;

                crate::println!("{}", code)?;
                Ok(ExitCode::SUCCESS)
            }
            Some((CMD_SCRIPT_EXTRACT, sub_matches)) => handle_extractvbs(sub_matches),
            Some((CMD_SCRIPT_IMPORT, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("VPXPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();

                let vbs_path_opt = sub_matches.get_one::<String>("VBSPATH").map(|s| {
                    let path = s.as_str();
                    expand_path(path)
                });

                let expanded_path = expand_path_exists(path)?;
                match importvbs(&expanded_path, vbs_path_opt) {
                    Ok(vbs_path) => {
                        crate::println!("IMPORTED {}", vbs_path.display())?;
                        Ok(ExitCode::SUCCESS)
                    }
                    Err(e) => {
                        let warning = format!("Error importing vbs: {e}").red();
                        crate::eprintln!("{}", warning)?;
                        Ok(ExitCode::FAILURE)
                    }
                }
            }
            Some((CMD_SCRIPT_EDIT, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("VPXPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();

                let expanded_vpx_path = expand_path_exists(path)?;

                let loaded_config = config::load_config()?;
                let config = loaded_config.as_ref().map(|c| &c.1);
                let vbs_path = vpx::vbs_path_for(&expanded_vpx_path);
                if vbs_path.exists() {
                    open_or_fail(&vbs_path, config)
                } else {
                    extractvbs(&expanded_vpx_path, None, false)?;
                    open_or_fail(&vbs_path, config)
                }
            }
            Some((CMD_SCRIPT_DIFF, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("VPXPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();

                let expanded_path = expand_path_exists(path)?;
                let diff = script_diff(&expanded_path)?;
                crate::println!("{}", diff)?;
                Ok(ExitCode::SUCCESS)
            }
            Some((CMD_SCRIPT_PATCH, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("VPXPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();

                let expanded_path = expand_path_exists(path)?;
                let vbs_path = match extractvbs(&expanded_path, None, false) {
                    Ok(ExtractResult::Existed(vbs_path)) => {
                        let warning =
                            format!("EXISTED {}", vbs_path.display()).truecolor(255, 125, 0);
                        crate::println!("{}", warning)?;
                        vbs_path
                    }
                    Ok(ExtractResult::Extracted(vbs_path)) => {
                        crate::println!("CREATED {}", vbs_path.display())?;
                        vbs_path
                    }
                    Err(e) => return fail_with_error("Error extracting vbs", e),
                };

                let applied = patch_vbs_file(&vbs_path)?;
                if applied.is_empty() {
                    crate::println!("No patches applied")?;
                } else {
                    applied
                        .iter()
                        .try_for_each(|patch| crate::println!("Applied patch: {}", patch))?;
                }
                Ok(ExitCode::SUCCESS)
            }
            _ => unreachable!(),
        },
        Some((CMD_LS, sub_matches)) => {
            let path = sub_matches
                .get_one::<String>("VPXPATH")
                .map(|s| s.as_str())
                .unwrap_or_default();

            let expanded_path = expand_path_exists(path)?;
            ls(&expanded_path)?;
            Ok(ExitCode::SUCCESS)
        }
        Some((CMD_EXTRACT, sub_matches)) => {
            let force = sub_matches.get_flag("FORCE");
            let paths: Vec<&str> = sub_matches
                .get_many::<String>("VPXPATH")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect();
            paths.iter().try_for_each(|path| {
                let expanded_path = expand_path_exists(path)?;
                let ext = expanded_path.extension().map(|e| e.to_ascii_lowercase());
                match ext {
                    Some(ext) if ext == "directb2s" => {
                        crate::println!("extracting from {}", expanded_path.display())?;
                        extract_directb2s(&expanded_path)?;
                        Ok(())
                    }
                    Some(ext) if ext == "vpx" => {
                        crate::println!("extracting from {}", expanded_path.display())?;
                        extract(expanded_path.as_ref(), force)?;
                        Ok(())
                    }
                    _ => Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unknown file type: {}", expanded_path.display()),
                    )),
                }
            })?;
            Ok(ExitCode::SUCCESS)
        }
        Some((CMD_ASSEMBLE, sub_matches)) => {
            let force = sub_matches.get_flag("FORCE");
            let dir_path = sub_matches
                .get_one::<String>("DIRPATH")
                .map(|s| s.as_str())
                .unwrap_or_default();
            let vpx_path_arg = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let expanded_dir_path = expand_path_exists(dir_path)?;
            let vpx_path = match vpx_path_arg {
                Some(path) => expand_path(path),
                None => {
                    let file_name = match expanded_dir_path.file_name() {
                        Some(name) => format!("{}.vpx", name.to_string_lossy()),
                        None => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "Invalid directory path",
                            ));
                        }
                    };
                    expanded_dir_path.with_file_name(file_name)
                }
            };
            if vpx_path.exists() {
                if force {
                    std::fs::remove_file(&vpx_path)?;
                } else {
                    let confirmed = confirm(
                        format!("\"{}\" already exists.", vpx_path.display()),
                        "Do you want to overwrite it?".to_string(),
                    )?;
                    if !confirmed {
                        crate::println!("Aborted")?;
                        return Ok(ExitCode::SUCCESS);
                    }
                    std::fs::remove_file(&vpx_path)?;
                }
            }
            let result = {
                let vpx = expanded::read(&expanded_dir_path)?;
                vpx::write(&vpx_path, &vpx)
            };
            match result {
                Ok(_) => {
                    crate::println!("Successfully assembled to {}", vpx_path.display())?;
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    crate::println!("Failed to assemble: {}", e)?;
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        Some((CMD_EXTRACT_VBS, sub_matches)) => handle_extractvbs(sub_matches),
        Some((CMD_IMPORT_VBS, sub_matches)) => {
            let path: &str = sub_matches.get_one::<String>("VPXPATH").unwrap().as_str();
            let expanded_path = expand_path_exists(path)?;
            match importvbs(&expanded_path, None) {
                Ok(vbs_path) => {
                    crate::println!("IMPORTED {}", vbs_path.display())?;
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    let warning = format!("Error importing vbs: {e}").red();
                    crate::eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        Some((CMD_PATCH, sub_matches)) => {
            let vpx_path = sub_matches
                .get_one::<String>("VPXPATH")
                .map(|s| Path::new(OsStr::new(s)))
                .expect("VPXPATH is required");
            let patch_path = sub_matches
                .get_one::<String>("PATCHPATH")
                .map(|s| Path::new(OsStr::new(s)))
                .expect("PATCHPATH is required");
            let patched_vpx_path = sub_matches
                .get_one::<String>("OUTVPXPATH")
                .map(PathBuf::from)
                .unwrap_or_else(|| vpx_path.with_extension("patched.vpx"));

            if !vpx_path.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("VPXPATH not found: {}", vpx_path.display()),
                ));
            }
            if !patch_path.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("PATCHPATH not found: {}", patch_path.display()),
                ));
            }
            if patched_vpx_path.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("OUTVPXPATH already exists: {}", patched_vpx_path.display()),
                ));
            }
            let vpx_file = File::open(vpx_path)?;
            let patch_file = File::open(patch_path)?;
            let patched_vpx_file = File::create(patched_vpx_path)?;

            let mut vpx_reader = BufReader::new(vpx_file);
            let mut patch_reader = BufReader::new(patch_file);
            let mut patched_vpx_writer = BufWriter::new(patched_vpx_file);

            jojodiff::patch(&mut vpx_reader, &mut patch_reader, &mut patched_vpx_writer)?;

            patched_vpx_writer.flush()?;

            Ok(ExitCode::SUCCESS)
        }

        Some((CMD_VERIFY, sub_matches)) => {
            let paths: Vec<&str> = sub_matches
                .get_many::<String>("VPXPATH")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();
            for path in paths {
                let expanded_path = expand_path_exists(path)?;
                match verify(&expanded_path) {
                    VerifyResult::Ok(vbs_path) => {
                        crate::println!("{OK} {}", vbs_path.display())?;
                    }
                    VerifyResult::Failed(vbs_path, msg) => {
                        let warning =
                            format!("{NOK} {} {}", vbs_path.display(), msg).truecolor(255, 125, 0);
                        crate::eprintln!("{}", warning)?;
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Some((CMD_NEW, sub_matches)) => {
            let path = {
                let this = sub_matches.get_one::<String>("VPXPATH").map(|v| v.as_str());
                match this {
                    Some(x) => x,
                    None => unreachable!("VPXPATH is required"),
                }
            };

            let expanded_path = shellexpand::tilde(path);
            crate::println!("creating new vpx file at {}", expanded_path)?;
            new(expanded_path.as_ref())?;
            Ok(ExitCode::SUCCESS)
        }
        Some((CMD_CONFIG, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_CONFIG_SETUP, _)) => match config::setup_config() {
                Ok(SetupConfigResult::Configured(config_path)) => {
                    crate::println!("Created config file {}", config_path.display())?;
                    Ok(ExitCode::SUCCESS)
                }
                Ok(SetupConfigResult::Existing(config_path)) => {
                    crate::println!(
                        "Config file already exists at \"{}\"",
                        config_path.display()
                    )?;
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    crate::eprintln!("Failed to create config file: {}", e)?;
                    Ok(ExitCode::FAILURE)
                }
            },
            Some((CMD_CONFIG_PATH, _)) => match config::config_path() {
                Some(config_path) => {
                    crate::println!("{}", config_path.display())?;
                    Ok(ExitCode::SUCCESS)
                }
                None => {
                    crate::eprintln!("No config file found")?;
                    Ok(ExitCode::FAILURE)
                }
            },
            Some((CMD_CONFIG_SHOW, _)) => match config::config_path() {
                Some(config_path) => {
                    let mut file = File::open(config_path)?;
                    let mut text = String::new();
                    file.read_to_string(&mut text)?;
                    crate::println!("{}", text)?;
                    Ok(ExitCode::SUCCESS)
                }
                None => {
                    crate::eprintln!("No config file found")?;
                    Ok(ExitCode::FAILURE)
                }
            },
            Some((CMD_CONFIG_CLEAR, _)) => match config::clear_config() {
                Ok(Some(config_path)) => {
                    crate::println!("Cleared config file {}", config_path.display())?;
                    Ok(ExitCode::SUCCESS)
                }
                Ok(None) => {
                    crate::println!("No config file found")?;
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => fail_with_error("Failed to clear config file: {}", e),
            },
            Some((CMD_CONFIG_EDIT, _)) => match config::config_path() {
                Some(config_path) => {
                    match config::load_config() {
                        Ok(loaded_config) => {
                            let config = loaded_config.as_ref().map(|c| &c.1);
                            open_editor(&config_path, config)?;
                        }
                        Err(_) => {
                            // if the config is invalid, we still want to allow editing
                            open_editor(&config_path, None)?;
                        }
                    };
                    Ok(ExitCode::SUCCESS)
                }
                None => fail("No config file found"),
            },
            _ => unreachable!(),
        },
        Some((CMD_IMAGES, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_IMAGES_WEBP, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("VPXPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();
                let expanded_path = expand_path_exists(path)?;
                let mut vpx_file = vpx::open_rw(&expanded_path)?;
                let images = vpx_file.images_to_webp()?;
                if !images.is_empty() {
                    for image in images.iter() {
                        crate::println!(
                            "Updated {} from {} to {}",
                            image.name,
                            image.old_extension,
                            image.new_extension
                        )?;
                    }
                    crate::println!("Compacting vpx file")?;
                    vpx::compact(&expanded_path)?;
                } else {
                    crate::println!("No images to update")?;
                }
                Ok(ExitCode::SUCCESS)
            }
            _ => unreachable!(),
        },
        Some((CMD_GAMEDATA, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_GAMEDATA_SHOW, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("VPXPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();
                let expanded_path = expand_path_exists(path)?;
                let mut vpx_file = vpx::open(expanded_path)?;
                let game_data = vpx_file.read_gamedata()?;
                let json = game_data_to_json(&game_data);
                let pretty = serde_json::to_string_pretty(&json)?;
                crate::println!("{}", pretty)?;
                Ok(ExitCode::SUCCESS)
            }
            _ => unreachable!(),
        },
        Some((CMD_DIPSWITCHES, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_DIPSWITCHES_SHOW, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("NVRAMPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();
                let expanded_path = expand_path_exists(path)?;
                let summary = show_dip_switches(&expanded_path)?;
                crate::println!("{}", summary)?;
                Ok(ExitCode::SUCCESS)
            }
            _ => unreachable!(),
        },
        Some((CMD_ROMNAME, sub_matches)) => {
            let path = sub_matches
                .get_one::<String>("VPXPATH")
                .map(|s| s.as_str())
                .unwrap_or_default();
            let expanded_path = expand_path_exists(path)?;
            if let Some(rom_name) = indexer::get_romname_from_vpx(&expanded_path)? {
                crate::println!("{rom_name}")?;
            }
            Ok(ExitCode::SUCCESS)
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn handle_index(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let recursive = sub_matches.get_flag("RECURSIVE");
    let tables_folders_path_arg = sub_matches
        .get_one::<String>("VPXROOTPATH")
        .map(|s| s.as_str());
    let index_file_arg = sub_matches
        .get_one::<String>("INDEX_FILE")
        .map(|s| s.as_str());
    let config = config::load_config()?;

    let tables_folder_path = match tables_folders_path_arg {
        Some(path) => expand_path_exists(path)?,
        None => match &config {
            Some((_, config)) => config.tables_folder.clone(),
            None => {
                crate::eprintln!("No VPXROOTPATH provided up and no vpxtool config file found")?;
                exit(1);
            }
        },
    };

    let tables_index_path = match index_file_arg {
        Some(path) => expand_path(path),
        None => tables_folder_path.join(DEFAULT_INDEX_FILE_NAME),
    };

    let global_pinmame_folder = config.as_ref().map(|(_, c)| c.global_pinmame_folder());
    let configured_pinmame_folder = config
        .as_ref()
        .and_then(|(_, c)| c.configured_pinmame_folder());

    crate::println!("Using tables folder {}", tables_folder_path.display())?;
    match &global_pinmame_folder {
        Some(folder) => {
            crate::println!("Using global pinmame folder {}", folder.display())?;
        }
        None => {
            crate::println!("Not looking for global pinmame roms as the folder is not configured.")?
        }
    }
    match &configured_pinmame_folder {
        Some(folder) => {
            crate::println!("Using VPinballX.ini PinMAMEPath {}", folder.display())?;
        }
        None => crate::println!("VPinballX.ini PinMAMEPath not used as not configured.")?,
    }
    crate::println!("Storing index to {}", tables_index_path.display())?;

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
        global_pinmame_folder.as_deref(),
        configured_pinmame_folder.as_deref(),
        &progress,
        vec![],
    )?;
    progress.finish_and_clear();
    crate::println!("Indexed {} vpx files", index.len(),)?;
    Ok(ExitCode::SUCCESS)
}

fn build_command() -> Command {
    // to allow for non-static strings in clap
    // I had to enable the "string" module
    // is this considered a bad idea?
    Command::new("vpxtool")
        .version(GIT_VERSION)
        .author("Francis DB")
        .about("Extracts and assembles vpx files")
        .arg_required_else_help(true)
        .before_help(format!("Vpxtool {GIT_VERSION}"))
        .arg(
            Arg::new(ARG_VERBOSE)
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Enable verbose logging")
                .global(true),
        )
        .subcommand(
            Command::new(CMD_INFO)
                .subcommand_required(true)
                .about("Vpx table info related commands")
                .subcommand(
                    Command::new(CMD_INFO_SHOW)
                        .about("Show information for a vpx file")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        ),
                )
                .subcommand(
                    Command::new(CMD_INFO_EXTRACT)
                        .about("Extract information from a vpx file")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        ),
                )
                .subcommand(
                    Command::new(CMD_INFO_IMPORT)
                        .about("Import information into a vpx file")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        ),
                )
                .subcommand(
                    Command::new(CMD_INFO_EDIT)
                        .about("Edit information for a vpx file")
                        .long_about("Extracts the information from the vpx file into a json file, and opens it in the default editor.")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        ),
                )
                .subcommand(
                    Command::new(CMD_INFO_DIFF)
                        .about("Prints out a diff between the info in the vpx and the sidecar json")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        ),
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
            Command::new(CMD_INDEX)
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
                    arg!(<VPXROOTPATH> "The path to the root directory of vpx files. Defaults to what is set up in the vpxtool config file.")
                        .required(false)
                )
                .arg(
                    arg!(<INDEX_FILE> "Where the index will be written. Defaults to VPXROOTPATH/vpxtool_index.json.")
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
                    extract_script_command(CMD_SCRIPT_EXTRACT),
                )
                .subcommand(
                    Command::new(CMD_SCRIPT_IMPORT)
                        .about("Import the table vpx script")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        )
                        .arg(
                            arg!([VBSPATH] "The optional path to the vbs file to import. Defaults to the vpx file path with the extension changed to .vbs.")
                                .required(false),
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
                .subcommand(
                    Command::new(CMD_SCRIPT_DIFF)
                        .about("Prints out a diff between the script in the vpx and the sidecar vbs")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        ),
                )
                .subcommand(
                    Command::new(CMD_SCRIPT_PATCH)
                        .about("Patch the table vpx script for typical standalone issues")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        ),
                ),
        )
        .subcommand(
            Command::new(CMD_LS)
                .about("Show the vpx file contents")
                .arg(
                    arg!(<VPXPATH> "The path to the vpx file")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new(CMD_EXTRACT)
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
            extract_script_command(CMD_EXTRACT_VBS),
        )
        .subcommand(
            Command::new(CMD_IMPORT_VBS)
                .about("Imports the vbs next to it into a vpx file")
                .arg(
                    arg!(<VPXPATH> "The path(s) to the vpx file(s)")
                        .required(true)
                        .num_args(1..),
                ),
        )
        .subcommand(
            Command::new(CMD_VERIFY)
                .about("Verify the structure of a vpx file")
                .arg(
                    arg!(<VPXPATH> "The path(s) to the vpx file(s)")
                        .required(true)
                        .num_args(1..),
                ),
        )
        .subcommand(
            Command::new(CMD_ASSEMBLE)
                .about("Assembles a vpx file")
                .arg(
                    Arg::new("FORCE")
                        .short('f')
                        .long("force")
                        .num_args(0)
                        .help("Do not ask for confirmation before overwriting existing files"),
                )
                .arg(arg!(<DIRPATH> "The path to the extracted vpx structure").required(true))
                .arg(arg!([VPXPATH] "Optional path of the VPX file to assemble to. Defaults to <DIRPATH>.vpx.")),
        )
        .subcommand(
            Command::new(CMD_PATCH)
                .about("Applies a VPURemix System patch to a table")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true))
                .arg(arg!(<PATCHPATH> "The path to the dif file").required(true))
                .arg(arg!(<OUTVPXPATH> "The path to the output vpx file. Defaults to <VPXPATH>.patched.vpx").required(false))
        )
        .subcommand(
            Command::new(CMD_NEW)
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
        .subcommand(
            Command::new(CMD_IMAGES)
                .subcommand_required(true)
                .about("Vpx image related commands")
                .subcommand(
                    Command::new(CMD_IMAGES_WEBP)
                        .about("Converts lossless (bmp/png) images in a vpx file to webp")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        ),
                ),
        )
        .subcommand(
            Command::new(CMD_GAMEDATA)
                .subcommand_required(true)
                .about("Vpx gamedata related commands")
                .subcommand(
                    Command::new(CMD_GAMEDATA_SHOW)
                        .about("Show the gamedata for a vpx file")
                        .arg(
                            arg!(<VPXPATH> "The path to the vpx file")
                                .required(true),
                        ),
                ),
        )
        .subcommand(
            Command::new(CMD_DIPSWITCHES)
                .subcommand_required(true)
                .about("NVRAM file DIP switch related commands")
                .subcommand(
                    Command::new(CMD_DIPSWITCHES_SHOW)
                        .about("Show the DIP switches for a nvram file")
                        .arg(
                            arg!(<NVRAMPATH> "The path to the nvram file")
                                .required(true),
                        ),
                ),
        )
        .subcommand(
            Command::new(CMD_ROMNAME)
                .about("Prints the PinMAME ROM name from a vpx file")
                .long_about("Extracts the PinMAME ROM name from a vpx file by searching for specific patterns in the table script. If the table is not PinMAME based, no output is produced.")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true)),
        )
}

fn extract_script_command(name: impl Into<Str>) -> Command {
    Command::new(name)
        .about("Extracts the script from a vpx file.")
        .long_about("Extracts the script from a vpx file by default into a vbs file next to it. Scripts placed next to the vpx file with the same name are considered sidecar scripts and will be picked up by Visual Pinball instead of the script inside the vpx file.")
        .arg(
            Arg::new("FORCE")
                .short('f')
                .long("force")
                .num_args(0)
                .default_value("false")
                .help("Will overwrite existing .vbs file if set."),
        )
        .arg(
            arg!(<VPXPATH> "The path to the vpx file")
                .required(true),
        )
        .arg(
            arg!([VBSPATH] "The optional path to the vbs file to write. Defaults to the vpx file path with the extension changed to .vbs. This option is mutually exclusive with DIRECTORY.")
                .required(false),
        )
        .arg(
            Arg::new("OUTPUT_DIRECTORY")
                .long("output-dir")
                .num_args(1)
                .required(false)
                .help("The directory to extract the vbs file to. Only if no VBSPATH is provided"),
        )
}

fn open_or_fail(vbs_path: &Path, config: Option<&ResolvedConfig>) -> io::Result<ExitCode> {
    match open_editor(vbs_path, config) {
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
    let error = "error:".red();
    crate::eprintln!("{} {}", error, message.as_ref())?;
    Ok(ExitCode::FAILURE)
}

fn new(vpx_file_path: &str) -> io::Result<()> {
    // TODO check if file exists and prompt to overwrite / add option to force
    vpx::new_minimal_vpx(vpx_file_path)
}

fn handle_extractvbs(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let force = sub_matches.get_flag("FORCE");
    let vpx_path = sub_matches.get_one::<String>("VPXPATH").map(expand_path);
    let vbs_path = sub_matches.get_one::<String>("VBSPATH").map(expand_path);
    let directory = sub_matches
        .get_one::<String>("OUTPUT_DIRECTORY")
        .map(expand_path);
    let expanded_vpx_path = path_exists(&vpx_path.expect("should be checked by clap"))?;
    if !expanded_vpx_path.is_file() {
        return fail(format!(
            "VPXPATH not a file: {}",
            expanded_vpx_path.display()
        ));
    }
    if vbs_path.is_some() && directory.is_some() {
        return fail("Conflicting VBSPATH and DIRECTORY options, only one can be used");
    }

    let vbs_path_opt = vbs_path.or_else(|| {
        directory.map(|dir| {
            dir.join(expanded_vpx_path.file_name().unwrap_or_default())
                .with_extension("vbs")
        })
    });

    match extractvbs(&expanded_vpx_path, vbs_path_opt, force) {
        Ok(ExtractResult::Existed(vbs_path)) => {
            let warning = format!("EXISTED {}", vbs_path.display()).truecolor(255, 125, 0);
            crate::println!("{}", warning)?;
        }
        Ok(ExtractResult::Extracted(vbs_path)) => {
            crate::println!("CREATED {}", vbs_path.display())?;
        }
        Err(e) => {
            let warning = format!("Error extracting vbs: {e}").red();
            crate::eprintln!("{}", warning)?;
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn extract_directb2s(expanded_path: &PathBuf) -> io::Result<()> {
    let file = File::open(expanded_path)?;
    let reader = BufReader::new(file);
    match read(reader) {
        Ok(b2s) => {
            crate::println!("DirectB2S file version {}", b2s.version)?;
            let root_dir_path = expanded_path.with_extension("directb2s.extracted");

            let mut root_dir = std::fs::DirBuilder::new();
            root_dir.recursive(true);
            root_dir.create(&root_dir_path)?;

            crate::println!("Writing to {}", root_dir_path.display())?;
            wite_images(b2s, root_dir_path.as_path());
        }
        Err(msg) => {
            crate::println!("Failed to load {}: {}", expanded_path.display(), msg)?;
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

pub(crate) fn info_gather(vpx_file_path: &PathBuf) -> io::Result<String> {
    let mut vpx_file = vpx::open(vpx_file_path)?;
    let version = vpx_file.read_version()?;
    // GameData also has a name field that we might want to display here
    // where is this shown in the UI?
    let table_info = vpx_file.read_tableinfo()?;

    let mut buffer = String::new();

    buffer.push_str(&format!("{:>18} {}\n", "VPX Version:".green(), version));
    buffer.push_str(&format!(
        "{:>18} {}\n",
        "Table Name:".green(),
        table_info.table_name.unwrap_or("[not set]".to_string())
    ));
    buffer.push_str(&format!(
        "{:>18} {}\n",
        "Version:".green(),
        table_info.table_version.unwrap_or("[not set]".to_string())
    ));
    buffer.push_str(&format!(
        "{:>18} {}{}{}\n",
        "Author:".green(),
        Some(table_info.author_name)
            .map(|s| s.unwrap_or("[not set]".to_string()))
            .filter(|s| !s.is_empty())
            .map(|s| format!("{s} "))
            .unwrap_or_default(),
        Some(table_info.author_email)
            .map(|s| s.unwrap_or("[not set]".to_string()))
            .filter(|s| !s.is_empty())
            .map(|s| format!("{s} "))
            .unwrap_or_default(),
        Some(table_info.author_website)
            .map(|s| s.unwrap_or("[not set]".to_string()))
            .filter(|s| !s.is_empty())
            .map(|s| format!("{s} "))
            .unwrap_or_default(),
    ));
    buffer.push_str(&format!(
        "{:>18} {}\n",
        "Save revision:".green(),
        table_info.table_save_rev.unwrap_or("[not set]".to_string())
    ));
    buffer.push_str(&format!(
        "{:>18} {}\n",
        "Save date:".green(),
        table_info
            .table_save_date
            .unwrap_or("[not set]".to_string())
    ));
    buffer.push_str(&format!(
        "{:>18} {}\n",
        "Release Date:".green(),
        table_info.release_date.unwrap_or("[not set]".to_string())
    ));
    buffer.push_str(&format!(
        "{:>18} {}\n",
        "Description:".green(),
        table_info
            .table_description
            .unwrap_or("[not set]".to_string())
    ));
    buffer.push_str(&format!(
        "{:>18} {}\n",
        "Blurb:".green(),
        table_info.table_blurb.unwrap_or("[not set]".to_string())
    ));
    buffer.push_str(&format!(
        "{:>18} {}\n",
        "Rules:".green(),
        table_info.table_rules.unwrap_or("[not set]".to_string())
    ));

    for (prop, value) in &table_info.properties {
        buffer.push_str(&format!("{:>18}: {}\n", prop.green(), value));
    }

    Ok(buffer)
}

fn info_extract(vpx_file_path: &Path) -> io::Result<ExitCode> {
    let info_file_path = vpx_file_path.with_extension("info.json");
    if info_file_path.exists() {
        let confirmed = confirm(
            format!("File \"{}\" already exists", info_file_path.display()),
            "Do you want to overwrite the existing file?".to_string(),
        )?;
        if !confirmed {
            crate::println!("Aborted")?;
            return Ok(ExitCode::SUCCESS);
        }
    }
    write_info_json(vpx_file_path, &info_file_path)?;
    crate::println!("Extracted table info to {}", info_file_path.display())?;
    Ok(ExitCode::SUCCESS)
}

fn write_info_json(vpx_file_path: &Path, info_file_path: &Path) -> io::Result<()> {
    let mut vpx_file = vpx::open(vpx_file_path)?;
    let table_info = vpx_file.read_tableinfo()?;
    let custom_info_tags = vpx_file.read_custominfotags()?;
    let table_info_json = info_to_json(&table_info, &custom_info_tags);
    let info_file = File::create(info_file_path)?;
    serde_json::to_writer_pretty(info_file, &table_info_json)?;
    Ok(())
}

pub(crate) fn info_edit(
    vpx_file_path: &Path,
    config: Option<&ResolvedConfig>,
) -> io::Result<PathBuf> {
    let info_file_path = vpx_file_path.with_extension("info.json");
    if !info_file_path.exists() {
        write_info_json(vpx_file_path, &info_file_path)?;
    }
    open_editor(&info_file_path, config)?;
    Ok(info_file_path)
}

pub(crate) fn open_editor(file_to_edit: &Path, config: Option<&ResolvedConfig>) -> io::Result<()> {
    match config.iter().flat_map(|c| c.editor.clone()).next() {
        Some(editor) => open_configured_editor(file_to_edit, &editor),
        None => edit::edit_file(file_to_edit),
    }
}

fn open_configured_editor(file_to_edit: &Path, editor: &String) -> io::Result<()> {
    let mut command = std::process::Command::new(editor);
    command.arg(file_to_edit);
    command.stdout(std::process::Stdio::inherit());
    command.stderr(std::process::Stdio::inherit());
    match command.status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                let warning = format!("Failed to open editor {editor}: {status}");
                Err(io::Error::other(warning))
            }
        }
        Err(e) => {
            let warning = format!("Failed to open editor {}: {}", &editor, e);
            Err(io::Error::other(warning))
        }
    }
}

fn info_import(_vpx_file_path: &Path) -> io::Result<ExitCode> {
    // let info_file_path = vpx_file_path.with_extension("info.json");
    // if !info_file_path.exists() {
    //     let warning = format!("File \"{}\" does not exist", info_file_path.display());
    //     return Err(io::Error::new(io::ErrorKind::NotFound, warning));
    // }
    // let mut info_file = File::open(&info_file_path)?;
    // let json = serde_json::from_reader(&mut info_file).map_err(|e| {
    //     io::Error::new(
    //         io::ErrorKind::Other,
    //         format!(
    //             "Failed to parse/read json {}: {}",
    //             info_file_path.display(),
    //             e
    //         ),
    //     )
    // })?;

    // let (table_info, custom_info_tags) = json_to_info(json, None)?;
    // let mut vpx_file = vpx::open(vpx_file_path)?;
    // vpx_file.write_custominfotags(&custom_info_tags)?;
    // vpx_file.write_tableinfo(&table_info)?;
    // println!("Imported table info from {}", info_file_path.display())?;
    // Ok(ExitCode::SUCCESS)
    fail("Not yet implemented")
}

pub fn ls(vpx_file_path: &Path) -> io::Result<()> {
    expanded::extract_directory_list(vpx_file_path)
        .iter()
        .try_for_each(|file_path| crate::println!("{}", file_path))
}

pub fn confirm(msg: String, yes_no_question: String) -> io::Result<bool> {
    // TODO do we need to check for terminal here?
    //   let use_color = stdout().is_terminal();
    let warning = msg.truecolor(255, 125, 0);
    crate::println!("{}", warning)?;
    crate::println!("{} (y/n)", yes_no_question)?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim() == "y")
}

pub fn extract(vpx_file_path: &Path, yes: bool) -> io::Result<ExitCode> {
    let root_dir_path_str = vpx_file_path.with_extension("");
    let root_dir_path = Path::new(&root_dir_path_str);

    // ask for confirmation if the directory exists
    if root_dir_path.exists() && !yes {
        let confirmed = confirm(
            format!("Directory \"{}\" already exists", root_dir_path.display()),
            "Do you want to remove the existing directory?".to_string(),
        )?;
        if !confirmed {
            crate::println!("Aborted")?;
            return Ok(ExitCode::SUCCESS);
        }
    }
    if root_dir_path.exists() {
        std::fs::remove_dir_all(root_dir_path)?;
    }
    let mut root_dir = std::fs::DirBuilder::new();
    root_dir.recursive(true);
    root_dir.create(root_dir_path)?;
    let result = {
        let vpx = vpx::read(vpx_file_path)?;
        expanded::write(&vpx, &root_dir_path, VERBOSE.load(Ordering::Relaxed))
    };
    match result {
        Ok(_) => {
            crate::println!("Successfully extracted to \"{}\"", root_dir_path.display())?;
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => fail(format!("Failed to extract: {e}")),
    }
}

pub fn info_diff(vpx_file_path: &Path) -> io::Result<String> {
    let expanded_vpx_path = expand_path_exists(vpx_file_path.to_str().unwrap())?;
    let info_file_path = expanded_vpx_path.with_extension("info.json");
    if info_file_path.exists() {
        let original_info_path =
            RemoveOnDrop::new(vpx_file_path.with_extension("info.original.tmp"));
        write_info_json(&expanded_vpx_path, original_info_path.path())?;
        let diff_color = if colored::control::SHOULD_COLORIZE.should_colorize() {
            DiffColor::Always
        } else {
            DiffColor::Never
        };
        let output = run_diff(original_info_path.path(), &info_file_path, diff_color)?;
        Ok(String::from_utf8_lossy(&output).to_string())
    } else {
        let msg = format!("No sidecar info file found: {}", info_file_path.display());
        Err(io::Error::new(io::ErrorKind::NotFound, msg))
    }
}

pub fn script_diff(vpx_file_path: &Path) -> io::Result<String> {
    // set extension for PathBuf
    let vbs_path = vpx_file_path.with_extension("vbs");
    if vbs_path.exists() {
        match vpx::open(vpx_file_path) {
            Ok(mut vpx_file) => {
                let gamedata = vpx_file.read_gamedata()?;
                let script = gamedata.code;
                let original_vbs_path =
                    RemoveOnDrop::new(vpx_file_path.with_extension("vbs.original.tmp"));
                std::fs::write(original_vbs_path.path(), script.string)?;
                let diff_color = if colored::control::SHOULD_COLORIZE.should_colorize() {
                    DiffColor::Always
                } else {
                    DiffColor::Never
                };
                let output = run_diff(original_vbs_path.path(), &vbs_path, diff_color)?;
                Ok(String::from_utf8_lossy(&output).to_string())
            }
            Err(e) => {
                let msg = format!("Not a valid vpx file {}: {}", vpx_file_path.display(), e);
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
    let original_vbs_file_name_no_tmp = original_vbs_filename.to_string_lossy().replace(".tmp", "");
    let vbs_filename = vbs_path.file_name().unwrap_or(vbs_path.as_os_str());
    let result = std::process::Command::new("diff")
        .current_dir(parent)
        .arg("-u")
        .arg("-w")
        .arg(format!("--color={}", color.to_diff_arg()))
        .arg(format!("--label={original_vbs_file_name_no_tmp}"))
        .arg(original_vbs_filename)
        .arg(format!("--label={}", vbs_filename.to_string_lossy()))
        .arg(vbs_filename)
        .output()
        .map(|o| o.stdout);
    result.map_err(|e| {
        let msg = format!("Failed to run 'diff'. Is it installed on your system? {e}");
        io::Error::other(msg)
    })
}

fn show_dip_switches(nvram: &PathBuf) -> io::Result<String> {
    let mut nvram_file = OpenOptions::new().read(true).open(nvram)?;
    let switches = get_all_dip_switches(&mut nvram_file)?;

    let mut lines = Vec::new();
    for s in switches {
        lines.push(format!(
            "DIP #{}: {}",
            s.nr,
            if s.on { "ON" } else { "OFF" }
        ));
    }

    let summary = lines.join("\n");
    Ok(summary)
}
