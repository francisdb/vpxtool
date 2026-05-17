use crate::config::{ResolvedConfig, SetupConfigResult};
use crate::indexer::{DEFAULT_INDEX_FILE_NAME, IndexError, Progress};
use crate::patcher::patch_vbs_file;
use crate::{
    RemoveOnDrop, config, frontend, indexer, os_independent_file_name, path_exists, strip_cr_lf,
};
use base64::Engine;
use clap::builder::Str;
use clap::{Arg, ArgAction, ArgMatches, Command, arg};
use colored::Colorize;
use console::Emoji;
use directb2s::read;
use git_version::git_version;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use log::{LevelFilter, info};
use pinmame_nvram::dips::get_all_dip_switches;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{ExitCode, exit};
use std::time::SystemTime;
use vpin::filesystem::RealFileSystem;
use vpin::vpx;
use vpin::vpx::expanded::ExpandOptions;
use vpin::vpx::export::gltf_export::{GltfExportOptions, GltfFormat, export_gltf};
use vpin::vpx::export::obj_export::{ExportUnits, ObjExportOptions, export_obj};
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
const CMD_LOCK: &str = "lock";
const CMD_UNLOCK: &str = "unlock";
const CMD_LOCK_STATUS: &str = "lock-status";

enum LockAction {
    Lock,
    Unlock,
    Status,
}

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
const CMD_IMAGES_LIST: &str = "list";

const CMD_SOUNDS: &str = "sounds";
const CMD_SOUNDS_LIST: &str = "list";

const CMD_COLLECTIONS: &str = "collections";
const CMD_COLLECTIONS_LIST: &str = "list";

const CMD_MATERIALS: &str = "materials";
const CMD_MATERIALS_LIST: &str = "list";

const CMD_GAMEITEMS: &str = "gameitems";
const CMD_GAMEITEMS_LIST: &str = "list";

const CMD_GAMEDATA: &str = "gamedata";
const CMD_GAMEDATA_SHOW: &str = "show";

const CMD_DIPSWITCHES: &str = "dipswitches";
const CMD_DIPSWITCHES_SHOW: &str = "show";

const CMD_NVRAM: &str = "nvram";
const CMD_NVRAM_SHOW: &str = "show";

const CMD_SCORES: &str = "scores";
const CMD_SCORES_SHOW: &str = "show";

const CMD_ROMNAME: &str = "romname";

const CMD_EXPORT: &str = "export";
const CMD_EXPORT_OBJ: &str = "obj";
const CMD_EXPORT_GLTF: &str = "gltf";
const CMD_EXPORT_VPXZ: &str = "vpxz";

const CMD_INDEX: &str = "index";
const ARG_VERBOSE: &str = "VERBOSE";
const ARG_MAX_DEPTH: &str = "MAX_DEPTH";

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
    init_logging(verbose);
    handle_command(matches)
}

fn init_logging(verbose: bool) {
    let start = SystemTime::now();
    let mut builder = env_logger::Builder::from_default_env();
    // only if the RUST_LOG env var is not set
    if std::env::var("RUST_LOG").is_err() {
        builder.format(move |buf, record| {
            let elapsed = start.elapsed().unwrap();
            let warn_style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "[{:>8}.{:03}] {warn_style}{:<5}{warn_style:#} {}",
                elapsed.as_secs(),
                elapsed.subsec_millis(),
                record.level(),
                record.args()
            )
        });
        if verbose {
            builder
                .filter_level(LevelFilter::Warn)
                .filter_module("vpin", LevelFilter::Info)
                .filter_module("vpxtool", LevelFilter::Info);
        } else {
            builder.filter_level(LevelFilter::Warn);
        }
    };
    builder.init();
}

fn handle_command(matches: ArgMatches) -> io::Result<ExitCode> {
    match matches.subcommand() {
        Some((CMD_INFO, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_INFO_SHOW, sub_matches)) => {
                let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
                let path = path.unwrap_or("");
                let expanded_path = path_exists(path)?;
                crate::println!("showing info for {}", expanded_path.display())?;
                let info = info_gather(&expanded_path)?;
                crate::println!("{}", info)?;
                Ok(ExitCode::SUCCESS)
            }
            Some((CMD_INFO_EXTRACT, sub_matches)) => {
                let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
                let path = path.unwrap_or("");
                let expanded_path = path_exists(path)?;
                crate::println!("extracting info for {}", expanded_path.display())?;
                info_extract(&expanded_path)
            }
            Some((CMD_INFO_IMPORT, sub_matches)) => {
                let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
                let path = path.unwrap_or("");
                let expanded_path = path_exists(path)?;
                crate::println!("importing info for {}", expanded_path.display())?;
                info_import(&expanded_path)
            }
            Some((CMD_INFO_EDIT, sub_matches)) => {
                let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
                let path = path.unwrap_or("");
                let expanded_path = path_exists(path)?;
                let loaded_config = config::load_config()?;
                let config = loaded_config.as_ref().map(|c| &c.1);
                crate::println!("editing info for {}", expanded_path.display())?;
                info_edit(&expanded_path, config)?;
                Ok(ExitCode::SUCCESS)
            }
            Some((CMD_INFO_DIFF, sub_matches)) => {
                let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
                let path = path.unwrap_or("");
                let expanded_path = path_exists(path)?;
                let loaded_config = config::load_config()?;
                let config = loaded_config.as_ref().map(|c| &c.1);
                crate::println!("diffing info for {}", expanded_path.display())?;
                let diff = info_diff(&expanded_path, config)?;
                crate::println!("{}", diff)?;
                Ok(ExitCode::SUCCESS)
            }
            _ => unreachable!(),
        },
        Some((CMD_DIFF, sub_matches)) => {
            // TODO this should diff more than only the script
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = path_exists(path)?;
            let loaded_config = config::load_config()?;
            let config = loaded_config.as_ref().map(|c| &c.1);
            match script_diff(&expanded_path, config) {
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
        Some((CMD_FRONTEND, sub_matches)) => {
            let (config_path, mut config) = config::load_or_setup_config()?;
            if let Some(suggested) =
                config::stale_vpx_config_suggestion(&config.vpx_config, &config.vpx_executable)
                && frontend::warn_stale_vpx_config(&config_path, &config.vpx_config, &suggested)
            {
                if let Err(e) = config::rewrite_vpx_config(&config_path, &suggested) {
                    crate::eprintln!(
                        "{}",
                        format!("Failed to rewrite {}: {e}", config_path.display()).red()
                    )?;
                } else {
                    crate::println!("Updated vpx_config in {}", config_path.display())?;
                }
                // Use the modern path for this session regardless of the
                // rewrite outcome, so the rest of the run reads the right ini.
                config.vpx_config = suggested;
            }
            let configured_pinmame_folder = config.configured_pinmame_folder();
            let max_depth = sub_matches
                .get_one::<usize>(ARG_MAX_DEPTH)
                .copied()
                .or(config.tables_scan_max_depth);
            crate::println!("Using vpxtool config file {}", config_path.display())?;
            crate::println!("Using vpinball config file {}", config.vpx_config.display())?;
            crate::println!(
                "Using global pinmame folder {}",
                config.global_pinmame_folder().display()
            )?;
            crate::println!(
                "Using configured pinmame folder {}",
                configured_pinmame_folder
                    .as_ref()
                    .map(|f| f.display().to_string())
                    .unwrap_or_else(|| "None".to_string())
            )?;
            match frontend::frontend_index(
                &config,
                true,
                max_depth,
                configured_pinmame_folder.as_deref(),
                vec![],
            ) {
                Ok(tables) if tables.is_empty() => {
                    let warning =
                        format!("No tables found in {}", config.tables_folder.display()).red();
                    crate::eprintln!("{}", warning)?;
                    Ok(ExitCode::FAILURE)
                }
                Ok(vpx_files_with_tableinfo) => {
                    frontend::frontend(
                        &config,
                        configured_pinmame_folder.as_deref(),
                        vpx_files_with_tableinfo,
                    );
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

                let expanded_path = path_exists(path)?;
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

                let vbs_path_opt = sub_matches.get_one::<String>("VBSPATH").map(PathBuf::from);

                let expanded_path = path_exists(path)?;
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

                let expanded_vpx_path = path_exists(path)?;

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

                let expanded_path = path_exists(path)?;
                let loaded_config = config::load_config()?;
                let config = loaded_config.as_ref().map(|c| &c.1);
                let diff = script_diff(&expanded_path, config)?;
                crate::println!("{}", diff)?;
                Ok(ExitCode::SUCCESS)
            }
            Some((CMD_SCRIPT_PATCH, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("VPXPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();

                let expanded_path = path_exists(path)?;
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

            let expanded_path = path_exists(path)?;
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
                let expanded_path = path_exists(path)?;
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
            let expanded_dir_path = path_exists(dir_path)?;
            let vpx_path = match vpx_path_arg {
                Some(path) => PathBuf::from(path),
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
            let expanded_path = path_exists(path)?;
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
                let expanded_path = path_exists(path)?;
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

        Some((CMD_LOCK, sub_matches)) => run_lock(sub_matches, LockAction::Lock),
        Some((CMD_UNLOCK, sub_matches)) => run_lock(sub_matches, LockAction::Unlock),
        Some((CMD_LOCK_STATUS, sub_matches)) => run_lock(sub_matches, LockAction::Status),
        Some((CMD_NEW, sub_matches)) => {
            let path = {
                let this = sub_matches.get_one::<String>("VPXPATH").map(|v| v.as_str());
                match this {
                    Some(x) => x,
                    None => unreachable!("VPXPATH is required"),
                }
            };

            crate::println!("creating new vpx file at {}", path)?;
            new(path)?;
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
                let expanded_path = path_exists(path)?;
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
            Some((CMD_IMAGES_LIST, sub_matches)) => handle_images_list(sub_matches),
            _ => unreachable!(),
        },
        Some((CMD_SOUNDS, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_SOUNDS_LIST, sub_matches)) => handle_sounds_list(sub_matches),
            _ => unreachable!(),
        },
        Some((CMD_COLLECTIONS, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_COLLECTIONS_LIST, sub_matches)) => handle_collections_list(sub_matches),
            _ => unreachable!(),
        },
        Some((CMD_MATERIALS, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_MATERIALS_LIST, sub_matches)) => handle_materials_list(sub_matches),
            _ => unreachable!(),
        },
        Some((CMD_GAMEITEMS, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_GAMEITEMS_LIST, sub_matches)) => handle_gameitems_list(sub_matches),
            _ => unreachable!(),
        },
        Some((CMD_GAMEDATA, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_GAMEDATA_SHOW, sub_matches)) => {
                let path = sub_matches
                    .get_one::<String>("VPXPATH")
                    .map(|s| s.as_str())
                    .unwrap_or_default();
                let expanded_path = path_exists(path)?;
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
                let expanded_path = path_exists(path)?;
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
            let expanded_path = path_exists(path)?;
            if let Some(rom_name) = indexer::get_romname_from_vpx(&expanded_path)? {
                crate::println!("{rom_name}")?;
            }
            Ok(ExitCode::SUCCESS)
        }
        Some((CMD_NVRAM, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_NVRAM_SHOW, sub_matches)) => handle_nvram_show(sub_matches),
            _ => unreachable!(),
        },
        Some((CMD_SCORES, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_SCORES_SHOW, sub_matches)) => handle_scores_show(sub_matches),
            _ => unreachable!(),
        },
        Some((CMD_EXPORT, sub_matches)) => match sub_matches.subcommand() {
            Some((CMD_EXPORT_OBJ, sub_matches)) => handle_export_obj(sub_matches),
            Some((CMD_EXPORT_GLTF, sub_matches)) => handle_export_gltf(sub_matches),
            Some((CMD_EXPORT_VPXZ, sub_matches)) => handle_export_vpxz(sub_matches),
            _ => unreachable!(),
        },
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn handle_index(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let recursive = sub_matches.get_flag("RECURSIVE");
    let force = sub_matches.get_flag("FORCE");
    let max_depth_cli = sub_matches.get_one::<usize>(ARG_MAX_DEPTH).copied();
    let tables_folders_path_arg = sub_matches
        .get_one::<String>("VPXROOTPATH")
        .map(|s| s.as_str());
    let index_file_arg = sub_matches
        .get_one::<String>("INDEX_FILE")
        .map(|s| s.as_str());
    let config = config::load_config()?;

    let tables_folder_path = match tables_folders_path_arg {
        Some(path) => path_exists(path)?,
        None => match &config {
            Some((_, config)) => config.tables_folder.clone(),
            None => {
                crate::eprintln!("No VPXROOTPATH provided up and no vpxtool config file found")?;
                exit(1);
            }
        },
    };

    let tables_index_path = match index_file_arg {
        Some(path) => PathBuf::from(path),
        None => tables_folder_path.join(DEFAULT_INDEX_FILE_NAME),
    };

    let global_pinmame_folder = config.as_ref().map(|(_, c)| c.global_pinmame_folder());
    let configured_pinmame_folder = config
        .as_ref()
        .and_then(|(_, c)| c.configured_pinmame_folder());
    let max_depth = max_depth_cli.or(config.as_ref().and_then(|(_, c)| c.tables_scan_max_depth));

    crate::println!("Using tables folder {}", tables_folder_path.display())?;
    if let Some(max_depth) = max_depth {
        crate::println!("Using tables scan max depth {}", max_depth)?;
    }
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
        max_depth,
        &tables_folder_path,
        &tables_index_path,
        global_pinmame_folder.as_deref(),
        configured_pinmame_folder.as_deref(),
        &progress,
        vec![],
        force,
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
                .arg(
                    Arg::new(ARG_MAX_DEPTH)
                        .long("max-depth")
                        .value_parser(clap::value_parser!(usize))
                        .help("Maximum directory depth to scan when indexing tables"),
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
                    Arg::new(ARG_MAX_DEPTH)
                        .long("max-depth")
                        .value_parser(clap::value_parser!(usize))
                        .help("Maximum directory depth to scan when indexing tables"),
                )
                .arg(
                    Arg::new("FORCE")
                        .short('f')
                        .long("force")
                        .num_args(0)
                        .help("Force re-indexing of every table, ignoring cached entries. Use after upgrading vpxtool to pick up newly detected fields (e.g. altsound, altcolor, pup pack)."),
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
            Command::new(CMD_LOCK)
                .about("Lock a vpx file, preventing edits in vpinball")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true)),
        )
        .subcommand(
            Command::new(CMD_UNLOCK)
                .about("Unlock a vpx file")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true)),
        )
        .subcommand(
            Command::new(CMD_LOCK_STATUS)
                .about("Show the lock state of a vpx file")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true)),
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
                )
                .subcommand(
                    Command::new(CMD_IMAGES_LIST)
                        .about("List the images stored in a vpx file")
                        .long_about(
                            "List the images stored in a vpx file as aligned columns: \
                             NAME, FORMAT, WIDTH, HEIGHT, SIZE (bytes), LINKED (Y/N for \
                             screenshot-style image links), PATH (original import path). \
                             PATH is last so awk-style column extraction works on the \
                             other fields even when paths contain spaces.",
                        )
                        .arg(arg!(<VPXPATH> "The path to the vpx file").required(true)),
                ),
        )
        .subcommand(
            Command::new(CMD_SOUNDS)
                .subcommand_required(true)
                .about("Vpx sound related commands")
                .subcommand(
                    Command::new(CMD_SOUNDS_LIST)
                        .about("List the sounds stored in a vpx file")
                        .long_about(
                            "List the sounds stored in a vpx file as aligned columns: \
                             NAME, FORMAT, OUTPUT (table/backglass), PAN, FADE, VOL \
                             (raw integers from the vpx file, not the signed-percent values \
                             vpinball shows in its GUI), FREQ (sample rate, Hz), CHAN \
                             (channel count), LENGTH (seconds, WAV only, blank otherwise), \
                             SIZE (bytes), PATH (original import path). PATH is last so \
                             awk-style column extraction works on the other fields even \
                             when paths contain spaces.",
                        )
                        .arg(arg!(<VPXPATH> "The path to the vpx file").required(true)),
                ),
        )
        .subcommand(
            Command::new(CMD_COLLECTIONS)
                .subcommand_required(true)
                .about("Vpx collection related commands")
                .subcommand(
                    Command::new(CMD_COLLECTIONS_LIST)
                        .about("List the collections stored in a vpx file")
                        .long_about(
                            "List the collections stored in a vpx file as aligned columns: \
                             NAME, ITEMS (number of element names in the collection), \
                             FIRE_EVENTS, STOP_SINGLES, GROUP_ELEMENTS (all Y/N flags).",
                        )
                        .arg(arg!(<VPXPATH> "The path to the vpx file").required(true)),
                ),
        )
        .subcommand(
            Command::new(CMD_MATERIALS)
                .subcommand_required(true)
                .about("Vpx material related commands")
                .subcommand(
                    Command::new(CMD_MATERIALS_LIST)
                        .about("List the materials stored in a vpx file")
                        .long_about(
                            "List the materials stored in a vpx file as aligned columns: \
                             NAME, BASE_COLOR (RGB hex), METAL (Y/N), ROUGHNESS (0..1), \
                             OPACITY (0..1), EDGE (0..1). Supports both the 10.8+ MATR \
                             format and the pre-10.8 MATE format; columns are the fields \
                             that exist in both.",
                        )
                        .arg(arg!(<VPXPATH> "The path to the vpx file").required(true)),
                ),
        )
        .subcommand(
            Command::new(CMD_GAMEITEMS)
                .subcommand_required(true)
                .about("Vpx gameitem (table element) related commands")
                .subcommand(
                    Command::new(CMD_GAMEITEMS_LIST)
                        .about("List the gameitems stored in a vpx file")
                        .long_about(
                            "List the gameitems (vpinball calls them \"elements\" in the \
                             editor GUI) stored in a vpx file as aligned columns: NAME, \
                             TYPE, VISIBLE (Y/N/- where '-' means the variant has no \
                             visibility concept), LOCKED (Y/N/-), LAYER (editor layer name \
                             if set, otherwise the numeric layer), PART_GROUP, \
                             PHYSICS_MATERIAL, IMAGES, MATERIALS. IMAGES and MATERIALS are \
                             '--'-joined to match the format vpinball's editor uses; this \
                             makes `grep -F -- '--MyTexture'` a reliable way to find every \
                             item that references a given texture. Empty cells mean either \
                             the field is not set or the variant has no such field. For \
                             type counts: `vpxtool gameitems list table.vpx | awk 'NR>1 \
                             {print $2}' | sort | uniq -c`.",
                        )
                        .arg(arg!(<VPXPATH> "The path to the vpx file").required(true)),
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
            Command::new(CMD_NVRAM)
                .subcommand_required(true)
                .about("PinMAME NVRAM related commands")
                .subcommand(
                    Command::new(CMD_NVRAM_SHOW)
                        .about("Resolve a PinMAME NVRAM file to JSON")
                        .long_about(
                            "Resolve a PinMAME NVRAM file to JSON using the pinmame-nvram maps. \
                             PATH may be a .vpx (the .nv is located via the configured/global \
                             pinmame folders), a .nv (resolved directly), or a rom .zip (the \
                             sibling ../nvram/<stem>.nv is used).",
                        )
                        .arg(arg!(<PATH> "Path to a .vpx, .nv, or rom .zip file").required(true)),
                ),
        )
        .subcommand(
            Command::new(CMD_SCORES)
                .subcommand_required(true)
                .about("Table high-score related commands")
                .subcommand(
                    Command::new(CMD_SCORES_SHOW)
                        .about("Show high scores for a table")
                        .long_about(
                            "Show the high-score entries stored for a table. PATH accepts a \
                             .vpx, a .nv, or a rom .zip exactly like `nvram show`. PinMAME \
                             tables (.nv/.zip or .vpx with a ROM) are resolved through the \
                             pinmame-nvram maps. For rom-less .vpx tables, three non-PinMAME \
                             backends are probed in order: `VPReg.ini` (in `user/` first, \
                             then sibling) keyed by the script's cGameName; a \
                             `<cGameName>_glf.ini` sibling (GLF framework); and any \
                             `user/*.txt` / `*.txt` files containing a 5-scores-then-5- \
                             initials block (EM tables using Black's Highscore routines).\n\
                             \n\
                             Default format is an aligned LABEL / INITIALS / SCORE table \
                             with comma-grouped scores. `--format tsv` emits tab-separated \
                             rows with raw integer scores for scripting (label and initials \
                             can contain spaces, so a tab-delimited format is the reliable \
                             way to split columns).",
                        )
                        .arg(arg!(<PATH> "Path to a .vpx, .nv, or rom .zip file").required(true))
                        .arg(
                            Arg::new("FORMAT")
                                .long("format")
                                .value_parser(["table", "tsv", "pinemhi"])
                                .default_value("table")
                                .help("Output format: 'table' (aligned columns, default), 'tsv' (tab-separated, raw scores), or 'pinemhi' (section layout similar to PINemHi's output)"),
                        ),
                ),
        )
        .subcommand(
            Command::new(CMD_ROMNAME)
                .about("Prints the PinMAME ROM name from a vpx file")
                .long_about("Extracts the PinMAME ROM name from a vpx file by searching for specific patterns in the table script. If the table is not PinMAME based, no output is produced.")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true)),
        )
        .subcommand(
            Command::new(CMD_EXPORT)
                .subcommand_required(true)
                .about("Export a vpx table to a 3D model format")
                .subcommand(
                    Command::new(CMD_EXPORT_OBJ)
                        .about("Export the table as a Wavefront OBJ + MTL (with images/)")
                        .arg(arg!(<VPXPATH> "The path to the vpx file").required(true))
                        .arg(
                            Arg::new("OUTPUT_DIR")
                                .short('o')
                                .long("output-dir")
                                .num_args(1)
                                .help("Output directory. Defaults to <stem>_obj/ next to the vpx file."),
                        )
                        .arg(
                            Arg::new("UNITS")
                                .long("units")
                                .num_args(1)
                                .value_parser(["vpu", "mm", "cm", "m"])
                                .default_value("m")
                                .help("Output units for vertex positions"),
                        )
                        .arg(
                            Arg::new("VPINBALL_STRICT")
                                .long("vpinball-strict")
                                .num_args(0)
                                .help("Match vpinball's own OBJ exporter (no textures, raw VPU, duplicate newmtl blocks). Overrides --units."),
                        ),
                )
                .subcommand(
                    Command::new(CMD_EXPORT_VPXZ)
                        .about("Export the table as a .vpxz archive for the Visual Pinball mobile app")
                        .long_about("Bundles the vpx and its sidecar files (.vbs, .ini, .directb2s, .png/.jpg) into a single .vpxz archive (a renamed zip). When the table is PinMAME-based, also bundles the matching rom zip from the configured pinmame folder unless --no-rom is set.")
                        .arg(arg!(<VPXPATH> "The path to the vpx file").required(true))
                        .arg(
                            Arg::new("OUTPUT")
                                .short('o')
                                .long("output")
                                .num_args(1)
                                .help("Output .vpxz path. Defaults to <stem>.vpxz one folder up from the vpx, so re-runs don't recursively pick up the previous output."),
                        )
                        .arg(
                            Arg::new("NO_ROM")
                                .long("no-rom")
                                .num_args(0)
                                .help("Do not bundle the matching PinMAME rom zip"),
                        )
                        .arg(
                            Arg::new("FORCE")
                                .short('f')
                                .long("force")
                                .num_args(0)
                                .help("Overwrite the output file if it already exists"),
                        ),
                )
                .subcommand(
                    Command::new(CMD_EXPORT_GLTF)
                        .about("Export the table as a glTF or GLB file")
                        .arg(arg!(<VPXPATH> "The path to the vpx file").required(true))
                        .arg(
                            Arg::new("OUTPUT_DIR")
                                .short('o')
                                .long("output-dir")
                                .num_args(1)
                                .help("Output directory. Defaults to <stem>_gltf/ next to the vpx file."),
                        )
                        .arg(
                            Arg::new("FORMAT")
                                .long("format")
                                .num_args(1)
                                .value_parser(["glb", "gltf"])
                                .default_value("glb")
                                .help("Output format: glb (single binary) or gltf (json + .bin sidecar)"),
                        )
                        .arg(
                            Arg::new("UNITS")
                                .long("units")
                                .num_args(1)
                                .value_parser(["vpu", "mm", "cm", "m"])
                                .default_value("m")
                                .help("Output units for vertex positions"),
                        )
                        .arg(
                            Arg::new("INVISIBLE")
                                .long("invisible")
                                .num_args(0)
                                .help("Include invisible items using the KHR_node_visibility extension"),
                        ),
                ),
        )
}

fn parse_units(s: &str) -> ExportUnits {
    match s {
        "vpu" => ExportUnits::Vpu,
        "mm" => ExportUnits::Mm,
        "cm" => ExportUnits::Cm,
        "m" => ExportUnits::M,
        _ => unreachable!("clap value_parser restricts this"),
    }
}

fn handle_export_obj(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let path = sub_matches
        .get_one::<String>("VPXPATH")
        .map(|s| s.as_str())
        .unwrap_or_default();
    let expanded_path = path_exists(path)?;
    let units = parse_units(sub_matches.get_one::<String>("UNITS").unwrap());
    let strict = sub_matches.get_flag("VPINBALL_STRICT");
    let output_dir = sub_matches
        .get_one::<String>("OUTPUT_DIR")
        .map(PathBuf::from);

    let stem = expanded_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "vpx path has no usable file stem",
            )
        })?
        .to_string();
    let parent = expanded_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let out_dir = output_dir.unwrap_or_else(|| parent.join(format!("{stem}_obj")));
    if out_dir.exists() && !out_dir.is_dir() {
        return fail(format!(
            "Output path exists and is not a directory: {}",
            out_dir.display()
        ));
    }
    std::fs::create_dir_all(&out_dir)?;
    let obj_path = out_dir.join(format!("{stem}.obj"));

    let mut options = if strict {
        ObjExportOptions::vpinball_strict()
    } else {
        ObjExportOptions::default()
    };
    if !strict {
        options.units = units;
    }

    crate::println!("Reading {}", expanded_path.display())?;
    let vpx = vpx::read(&expanded_path)?;
    crate::println!(
        "Exporting OBJ to {} (units: {:?}, mode: {})",
        obj_path.display(),
        options.units,
        if strict { "vpinball-strict" } else { "default" },
    )?;
    export_obj(&vpx, &obj_path, &RealFileSystem, &options)?;
    crate::println!("Done.")?;
    Ok(ExitCode::SUCCESS)
}

fn handle_export_gltf(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let path = sub_matches
        .get_one::<String>("VPXPATH")
        .map(|s| s.as_str())
        .unwrap_or_default();
    let expanded_path = path_exists(path)?;
    let format = match sub_matches.get_one::<String>("FORMAT").unwrap().as_str() {
        "glb" => GltfFormat::Glb,
        "gltf" => GltfFormat::Gltf,
        _ => unreachable!("clap value_parser restricts this"),
    };
    let units = parse_units(sub_matches.get_one::<String>("UNITS").unwrap());
    let export_invisible = sub_matches.get_flag("INVISIBLE");
    let output_dir = sub_matches
        .get_one::<String>("OUTPUT_DIR")
        .map(PathBuf::from);

    let stem = expanded_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "vpx path has no usable file stem",
            )
        })?
        .to_string();
    let parent = expanded_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let out_dir = output_dir.unwrap_or_else(|| parent.join(format!("{stem}_gltf")));
    if out_dir.exists() && !out_dir.is_dir() {
        return fail(format!(
            "Output path exists and is not a directory: {}",
            out_dir.display()
        ));
    }
    std::fs::create_dir_all(&out_dir)?;
    let ext = match format {
        GltfFormat::Glb => "glb",
        GltfFormat::Gltf => "gltf",
    };
    let output_path = out_dir.join(format!("{stem}.{ext}"));

    let options = GltfExportOptions {
        format,
        export_invisible_items: export_invisible,
        units,
    };

    crate::println!("Reading {}", expanded_path.display())?;
    let vpx = vpx::read(&expanded_path)?;
    crate::println!(
        "Exporting {} to {} (units: {:?}{})",
        match format {
            GltfFormat::Glb => "GLB",
            GltfFormat::Gltf => "glTF",
        },
        output_path.display(),
        options.units,
        if export_invisible {
            ", including invisible items"
        } else {
            ""
        },
    )?;
    export_gltf(&vpx, &output_path, &RealFileSystem, &options)?;
    crate::println!("Done.")?;
    Ok(ExitCode::SUCCESS)
}

fn handle_export_vpxz(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let path = sub_matches
        .get_one::<String>("VPXPATH")
        .map(|s| s.as_str())
        .unwrap_or_default();
    let expanded_path = path_exists(path)?;
    let bundle_rom = !sub_matches.get_flag("NO_ROM");
    let force = sub_matches.get_flag("FORCE");
    let output = sub_matches.get_one::<String>("OUTPUT").map(PathBuf::from);

    let output_path = match output {
        Some(p) => p,
        None => crate::vpxz::default_output_path(&expanded_path)?,
    };

    if output_path.exists() {
        if output_path.is_dir() {
            return fail(format!(
                "Output path exists and is a directory: {}",
                output_path.display()
            ));
        }
        if !force {
            let confirmed = confirm(
                format!("\"{}\" already exists.", output_path.display()),
                "Do you want to overwrite it?".to_string(),
            )?;
            if !confirmed {
                crate::println!("Aborted")?;
                return Ok(ExitCode::SUCCESS);
            }
        }
    }

    let loaded_config = config::load_config()?;
    let config = loaded_config.as_ref().map(|c| &c.1);

    let rom_name = indexer::get_romname_from_vpx(&expanded_path)?;
    let rom_zip = if bundle_rom {
        let configured = config.and_then(|c| c.configured_pinmame_folder());
        let global = config.map(|c| c.global_pinmame_folder());
        crate::vpxz::find_rom_zip(&expanded_path, configured.as_deref(), global.as_deref())?
    } else {
        None
    };

    let exclude_globs: Vec<String> = config
        .map(|c| c.vpxz_excludes.clone())
        .unwrap_or_else(config::default_vpxz_excludes);

    let parent = expanded_path
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    crate::println!("Scanning {parent} ...")?;

    let pb = ProgressBar::hidden();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{bar:.cyan/blue}] {pos}/{human_len} {msg}")
            .unwrap(),
    );
    pb.set_message("bundling");
    let progress = ProgressBarProgress::new(pb);

    let report = crate::vpxz::export_vpxz(
        &expanded_path,
        &output_path,
        &crate::vpxz::VpxzExportOptions {
            exclude_globs: &exclude_globs,
            rom_zip: rom_zip.as_deref(),
            progress: Some(&progress),
        },
    )?;

    if !report.excluded.is_empty() {
        crate::println!("Excluded {} files:", report.excluded.len())?;
        for (path, reason) in &report.excluded {
            crate::println!("  {path} [{reason}]")?;
        }
    }
    if let Some(rom_path) = &report.injected_rom {
        crate::println!("Injected rom from {}", rom_path.display())?;
    }
    if bundle_rom
        && let Some(rom_name) = rom_name.as_deref()
        && !report.rom_bundled(rom_name)
    {
        crate::println!(
            "{}",
            format!("Note: rom '{rom_name}' not found; not bundled.").truecolor(255, 125, 0)
        )?;
    }
    crate::println!(
        "Wrote {} ({} included, {} excluded)",
        report.output.display(),
        report.included.len(),
        report.excluded.len()
    )?;
    Ok(ExitCode::SUCCESS)
}

/// Resolve `path` to an nvram file. Accepts:
/// * a `.nv` file directly,
/// * a rom `.zip` (looks for `../nvram/<stem>.nv` next to it), or
/// * a `.vpx` (reads the rom name from the script and searches the configured
///   / global pinmame folders).
///
/// Returns a CLI-friendly error path on the failure cases so callers can just
/// `?` into the rest of their handler.
enum NvramResolveError {
    NotPinmame(PathBuf),
    NoNvramFor(PathBuf),
    NoNvramNextToZip(PathBuf),
    InvalidZipStem(PathBuf),
    UnsupportedExtension(PathBuf),
}

impl NvramResolveError {
    fn fail(self) -> io::Result<ExitCode> {
        match self {
            NvramResolveError::NotPinmame(p) => {
                fail(format!("Table {} is not PinMAME-based", p.display()))
            }
            NvramResolveError::NoNvramFor(p) => fail(format!(
                "No nvram file found for {} - try launching the table once",
                p.display()
            )),
            NvramResolveError::NoNvramNextToZip(p) => fail(format!(
                "No nvram file found next to rom zip {}",
                p.display()
            )),
            NvramResolveError::InvalidZipStem(p) => {
                fail(format!("rom zip has no usable file stem: {}", p.display()))
            }
            NvramResolveError::UnsupportedExtension(p) => fail(format!(
                "Unsupported file type: {} (expected .vpx, .nv, or rom .zip)",
                p.display()
            )),
        }
    }
}

fn resolve_nvram_path(expanded_path: &Path) -> io::Result<Result<PathBuf, NvramResolveError>> {
    let nvram_path = match expanded_path
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("nv") => expanded_path.to_path_buf(),
        Some("zip") => {
            let Some(stem) = expanded_path.file_stem().and_then(OsStr::to_str) else {
                return Ok(Err(NvramResolveError::InvalidZipStem(
                    expanded_path.to_path_buf(),
                )));
            };
            let candidate = expanded_path
                .parent()
                .and_then(Path::parent)
                .map(|p| p.join("nvram").join(format!("{stem}.nv")));
            match candidate.filter(|p| p.is_file()) {
                Some(p) => p,
                None => {
                    return Ok(Err(NvramResolveError::NoNvramNextToZip(
                        expanded_path.to_path_buf(),
                    )));
                }
            }
        }
        Some("vpx") => {
            if indexer::get_romname_from_vpx(expanded_path)?.is_none() {
                return Ok(Err(NvramResolveError::NotPinmame(
                    expanded_path.to_path_buf(),
                )));
            }
            let loaded_config = config::load_config()?;
            let config = loaded_config.as_ref().map(|c| &c.1);
            let configured = config.and_then(|c| c.configured_pinmame_folder());
            let global = config.map(|c| c.global_pinmame_folder());
            match indexer::find_nvram_for_vpx(
                expanded_path,
                configured.as_deref(),
                global.as_deref(),
            )? {
                Some(p) => p,
                None => {
                    return Ok(Err(NvramResolveError::NoNvramFor(
                        expanded_path.to_path_buf(),
                    )));
                }
            }
        }
        _ => {
            return Ok(Err(NvramResolveError::UnsupportedExtension(
                expanded_path.to_path_buf(),
            )));
        }
    };
    Ok(Ok(nvram_path))
}

fn handle_nvram_show(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let path = sub_matches
        .get_one::<String>("PATH")
        .map(|s| s.as_str())
        .unwrap_or_default();
    let expanded_path = path_exists(path)?;

    let nvram_path = match resolve_nvram_path(&expanded_path)? {
        Ok(p) => p,
        Err(e) => return e.fail(),
    };

    match pinmame_nvram::resolve::resolve(&nvram_path) {
        Ok(Some(resolved)) => {
            let json = serde_json::to_string_pretty(&resolved)
                .map_err(|e| io::Error::other(format!("Failed to serialize nvram json: {e}")))?;
            crate::println!("{json}")?;
            Ok(ExitCode::SUCCESS)
        }
        Ok(None) => fail(format!("No pinmame-nvram map for {}", nvram_path.display())),
        Err(e) => fail(format!(
            "Failed to resolve nvram {}: {e}",
            nvram_path.display()
        )),
    }
}

fn handle_scores_show(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let path = sub_matches
        .get_one::<String>("PATH")
        .map(|s| s.as_str())
        .unwrap_or_default();
    let format = sub_matches
        .get_one::<String>("FORMAT")
        .map(|s| s.as_str())
        .unwrap_or("table");
    let expanded_path = path_exists(path)?;

    // Resolve the input into a flat list of sections, trying PinMAME first
    // and falling back to VPReg.ini for .vpx tables that are not PinMAME.
    let sections: Vec<crate::scores::Section> = match resolve_nvram_path(&expanded_path)? {
        Ok(nvram_path) => match pinmame_nvram::resolve::resolve(&nvram_path) {
            Ok(Some(r)) => crate::scores::extract_sections(&r),
            Ok(None) => {
                return fail(format!("No pinmame-nvram map for {}", nvram_path.display()));
            }
            Err(e) => {
                return fail(format!(
                    "Failed to resolve nvram {}: {e}",
                    nvram_path.display()
                ));
            }
        },
        Err(prior) => match try_non_pinmame_fallback(&expanded_path, &prior)? {
            Some(sections) => sections,
            None => match &prior {
                // For a rom-less .vpx we probed VPReg, GLF, and EM .txt
                // before giving up - the original "not PinMAME-based" wording
                // would suggest we never tried. Surface a holistic message.
                NvramResolveError::NotPinmame(p) => {
                    return fail(format!(
                        "Could not find any high scores for {}: tried PinMAME \
                         nvram, VPReg.ini, GLF, and EM-style .txt files",
                        p.display()
                    ));
                }
                _ => return prior.fail(),
            },
        },
    };

    render_sections(&sections, format)
}

/// If `expanded_path` is a `.vpx` that PinMAME resolution couldn't handle,
/// probe the non-PinMAME score storage backends in order: VPReg first
/// (by far the most common rom-less storage; one shared `user/VPReg.ini`
/// keyed by `[<cGameName>]`), then GLF (a `<cGameName>_glf.ini` sibling
/// from the vpx-glf framework, far less common in the wild).
///
/// Returns `Ok(None)` when no backend matched (non-vpx input, non-fallthrough
/// error, no `cGameName` in the script, or no scores anywhere) so the caller
/// falls back to the original PinMAME error message. Returns `Err` only when
/// a backend file *exists* but is malformed - that's a real failure, not a
/// "this backend doesn't apply" signal.
fn try_non_pinmame_fallback(
    expanded_path: &Path,
    prior_err: &NvramResolveError,
) -> io::Result<Option<Vec<crate::scores::Section>>> {
    let is_vpx = expanded_path
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|e| e.eq_ignore_ascii_case("vpx"));
    let should_try = is_vpx
        && matches!(
            prior_err,
            NvramResolveError::NotPinmame(_) | NvramResolveError::NoNvramFor(_)
        );
    if !should_try {
        return Ok(None);
    }
    let Some(game_name) = indexer::get_gamename_from_vpx(expanded_path)? else {
        return Ok(None);
    };
    let vpx_parent = expanded_path.parent().unwrap_or(Path::new("."));

    // VPReg: `user/VPReg.ini` first because standalone vpinball writes there
    // by default; sibling as a fallback for older layouts.
    let vpreg_candidates = [
        vpx_parent.join("user").join("VPReg.ini"),
        vpx_parent.join("VPReg.ini"),
    ];
    for candidate in &vpreg_candidates {
        if !candidate.is_file() {
            continue;
        }
        match crate::scores::vpreg::read_sections(candidate, &game_name) {
            Ok(sections) => return Ok(Some(sections)),
            Err(crate::scores::vpreg::LookupError::SectionNotFound)
            | Err(crate::scores::vpreg::LookupError::SectionHasNoScores) => continue,
            Err(crate::scores::vpreg::LookupError::ParseFailed(msg)) => {
                return Err(io::Error::other(format!(
                    "Failed to parse {}: {msg}",
                    candidate.display()
                )));
            }
        }
    }

    // GLF: `<cGameName>_glf.ini` sibling of the .vpx.
    let glf_path = vpx_parent.join(format!("{game_name}_glf.ini"));
    if glf_path.is_file() {
        match crate::scores::glf::read_sections(&glf_path) {
            Ok(sections) => return Ok(Some(sections)),
            // GLF file present but no usable scores - return None so the
            // caller surfaces the original PinMAME error.
            Err(crate::scores::glf::LookupError::NoHighScoresSection)
            | Err(crate::scores::glf::LookupError::EmptyHighScores) => {}
            Err(crate::scores::glf::LookupError::ParseFailed(msg)) => {
                return Err(io::Error::other(format!(
                    "Failed to parse {}: {msg}",
                    glf_path.display()
                )));
            }
        }
    }

    // EM-style `.txt` (Black's Highscore routines): per-table file with
    // non-canonical name and a variable header. Glob `user/*.txt` (the
    // standard standalone-vpinball location) and then `*.txt` in the table
    // folder, sniffing each for the 5-scores-then-5-initials block. First
    // parse-success wins.
    if let Some(sections) = try_emhs_glob(vpx_parent)? {
        return Ok(Some(sections));
    }

    Ok(None)
}

/// Probe candidate EM-style score `.txt` files in `user/` then the table
/// folder root. Returns the first file whose content yields a valid score
/// block; `Ok(None)` when none match (or directories don't exist).
fn try_emhs_glob(vpx_parent: &Path) -> io::Result<Option<Vec<crate::scores::Section>>> {
    for dir in [vpx_parent.join("user"), vpx_parent.to_path_buf()] {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        // Sort so the probe order is reproducible across runs (read_dir's
        // iteration order is OS- and inode-dependent otherwise).
        let mut txts: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.extension()
                        .and_then(OsStr::to_str)
                        .is_some_and(|e| e.eq_ignore_ascii_case("txt"))
            })
            .collect();
        txts.sort();
        for candidate in txts {
            match crate::scores::emhs::read_sections(&candidate) {
                Ok(sections) => return Ok(Some(sections)),
                Err(crate::scores::emhs::LookupError::PatternNotFound) => continue,
                Err(crate::scores::emhs::LookupError::ReadFailed(msg)) => {
                    return Err(io::Error::other(format!(
                        "Failed to read {}: {msg}",
                        candidate.display()
                    )));
                }
            }
        }
    }
    Ok(None)
}

/// Render a resolved `Vec<Section>` in the requested format. Common rendering
/// path shared by every backend (PinMAME nvram, GLF, VPReg.ini).
fn render_sections(sections: &[crate::scores::Section], format: &str) -> io::Result<ExitCode> {
    match format {
        "tsv" => {
            // Header + raw rows, tab-separated. Scores stay as raw integers
            // so scripts can `awk -F$'\t' '$3>=1000000'` directly; the
            // trailing UNITS column lets scripts that care format time-like
            // scores themselves. Locale-independent by design.
            crate::println!("{}", crate::scores::HEADERS.join("\t"))?;
            for section in sections {
                for row in &section.rows {
                    crate::println!("{}", row.join("\t"))?;
                }
            }
        }
        "pinemhi" => {
            // Section-based layout matching PINemHi's output shape: separate
            // GRAND CHAMPION block when present, one HIGH SCORES block for
            // ranked entries, one section per mode champion.
            // Branch on the locale type rather than using a `&dyn Format` -
            // num-format's ToFormattedString requires Sized. Windows lacks
            // SystemLocale (see Cargo.toml note) so it always uses Locale::en.
            #[cfg(not(windows))]
            let rendered = if let Some(sys) = readable_system_locale() {
                crate::scores::render_pinemhi(sections, &sys)
            } else {
                crate::scores::render_pinemhi(sections, &num_format::Locale::en)
            };
            #[cfg(windows)]
            let rendered = crate::scores::render_pinemhi(sections, &num_format::Locale::en);
            crate::print!("{}", rendered)?;
        }
        _ => {
            // Human table view: drop the trailing UNITS column after using
            // it to format the SCORE column (e.g. seconds -> mm:ss).
            let mut rows: Vec<Vec<String>> = sections
                .iter()
                .flat_map(|s| s.rows.iter().cloned())
                .collect();
            #[cfg(not(windows))]
            if let Some(sys) = readable_system_locale() {
                crate::scores::pretty_score_column(&mut rows, &sys);
            } else {
                crate::scores::pretty_score_column(&mut rows, &num_format::Locale::en);
            }
            #[cfg(windows)]
            crate::scores::pretty_score_column(&mut rows, &num_format::Locale::en);
            let visible_headers = ["LABEL", "INITIALS", "SCORE"];
            let aligns = [ColAlign::Left, ColAlign::Left, ColAlign::Right];
            let visible_rows: Vec<Vec<String>> = rows
                .into_iter()
                .map(|mut r| {
                    r.truncate(3);
                    r
                })
                .collect();
            print_aligned_table(&visible_headers, &aligns, &visible_rows)?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// Read the user's `LC_ALL` / `LANG` / `LC_NUMERIC` to pick a thousands
/// separator, but **fall back to `Locale::en` (comma) when the system locale
/// has no separator at all** - the POSIX `C` / `POSIX` locales specify
/// `thousands_sep=""`, which would render `52000000` instead of `52,000,000`.
/// PINemHi makes the same readability-over-strict-POSIX call, so we match.
///
/// Windows-only note: `num-format`'s `with-system-locale` Windows backend
/// fails to build (https://github.com/bcmyers/num-format/issues/43), so
/// this function is not compiled there; the call sites fall back to
/// `Locale::en` directly.
#[cfg(not(windows))]
fn readable_system_locale() -> Option<num_format::SystemLocale> {
    let sys = num_format::SystemLocale::default().ok()?;
    if sys.separator().is_empty() {
        return None;
    }
    Some(sys)
}

/// Right- vs left-aligned column. The last column is printed without trailing
/// padding regardless, so paths with spaces don't break awk-style splitting
/// on the preceding fields.
#[derive(Clone, Copy)]
enum ColAlign {
    Left,
    Right,
}

fn print_aligned_table(
    headers: &[&str],
    aligns: &[ColAlign],
    rows: &[Vec<String>],
) -> io::Result<()> {
    assert_eq!(headers.len(), aligns.len());
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        assert_eq!(row.len(), headers.len());
        for (i, cell) in row.iter().enumerate() {
            if cell.len() > widths[i] {
                widths[i] = cell.len();
            }
        }
    }
    let mut buf = String::new();
    let mut emit = |cells: &[&str]| -> io::Result<()> {
        buf.clear();
        let last = cells.len() - 1;
        for (i, cell) in cells.iter().enumerate() {
            if i > 0 {
                buf.push_str("  ");
            }
            if i == last {
                buf.push_str(cell);
            } else {
                match aligns[i] {
                    ColAlign::Left => buf.push_str(&format!("{:<w$}", cell, w = widths[i])),
                    ColAlign::Right => buf.push_str(&format!("{:>w$}", cell, w = widths[i])),
                }
            }
        }
        crate::println!("{}", buf)
    };
    emit(headers)?;
    for row in rows {
        let refs: Vec<&str> = row.iter().map(String::as_str).collect();
        emit(&refs)?;
    }
    Ok(())
}

fn handle_images_list(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let path = sub_matches
        .get_one::<String>("VPXPATH")
        .map(|s| s.as_str())
        .unwrap_or_default();
    let expanded_path = path_exists(path)?;
    let mut vpx_file = vpx::open(&expanded_path)?;
    let images = vpx_file.read_images()?;

    let rows: Vec<Vec<String>> = images
        .iter()
        .map(|image| {
            let size_bytes = if let Some(jpeg) = &image.jpeg {
                jpeg.data.len()
            } else if let Some(bits) = &image.bits {
                bits.lzw_compressed_data.len()
            } else {
                0
            };
            vec![
                image.name.clone(),
                image.ext(),
                image.width.to_string(),
                image.height.to_string(),
                size_bytes.to_string(),
                if image.is_link() { "Y" } else { "N" }.to_string(),
                image.path.clone(),
            ]
        })
        .collect();

    let headers = [
        "NAME", "FORMAT", "WIDTH", "HEIGHT", "SIZE", "LINKED", "PATH",
    ];
    let aligns = [
        ColAlign::Left,
        ColAlign::Left,
        ColAlign::Right,
        ColAlign::Right,
        ColAlign::Right,
        ColAlign::Left,
        ColAlign::Left,
    ];
    print_aligned_table(&headers, &aligns, &rows)?;
    Ok(ExitCode::SUCCESS)
}

fn handle_sounds_list(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let path = sub_matches
        .get_one::<String>("VPXPATH")
        .map(|s| s.as_str())
        .unwrap_or_default();
    let expanded_path = path_exists(path)?;
    let mut vpx_file = vpx::open(&expanded_path)?;
    let sounds = vpx_file.read_sounds()?;

    let rows: Vec<Vec<String>> = sounds
        .iter()
        .map(|sound| {
            let ext = sound
                .path
                .rsplit('.')
                .next()
                .filter(|e| !e.contains(['/', '\\']))
                .unwrap_or("")
                .to_ascii_lowercase();
            let is_wav = ext.is_empty() || ext == "wav";
            let format = if ext.is_empty() {
                "wav".to_string()
            } else {
                ext
            };
            let output = match sound.output_target {
                vpin::vpx::sound::OutputTarget::Table => "table",
                vpin::vpx::sound::OutputTarget::Backglass => "backglass",
            };
            // For WAV (PCM) we can derive the playback duration from the raw
            // sample bytes; for non-WAV containers the data is compressed and
            // we'd have to parse the container, so leave it blank.
            let length = if is_wav && sound.wave_form.avg_bytes_per_sec > 0 {
                format!(
                    "{:.2}",
                    sound.data.len() as f64 / sound.wave_form.avg_bytes_per_sec as f64
                )
            } else {
                String::new()
            };
            vec![
                sound.name.clone(),
                format,
                output.to_string(),
                sound.balance.to_string(),
                sound.fade.to_string(),
                sound.volume.to_string(),
                sound.wave_form.samples_per_sec.to_string(),
                sound.wave_form.channels.to_string(),
                length,
                sound.data.len().to_string(),
                sound.path.clone(),
            ]
        })
        .collect();

    let headers = [
        "NAME", "FORMAT", "OUTPUT", "PAN", "FADE", "VOL", "FREQ", "CHAN", "LENGTH", "SIZE", "PATH",
    ];
    let aligns = [
        ColAlign::Left,  // NAME
        ColAlign::Left,  // FORMAT
        ColAlign::Left,  // OUTPUT
        ColAlign::Right, // PAN
        ColAlign::Right, // FADE
        ColAlign::Right, // VOL
        ColAlign::Right, // FREQ
        ColAlign::Right, // CHAN
        ColAlign::Right, // LENGTH
        ColAlign::Right, // SIZE
        ColAlign::Left,  // PATH
    ];
    print_aligned_table(&headers, &aligns, &rows)?;
    Ok(ExitCode::SUCCESS)
}

fn handle_collections_list(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let path = sub_matches
        .get_one::<String>("VPXPATH")
        .map(|s| s.as_str())
        .unwrap_or_default();
    let expanded_path = path_exists(path)?;
    let mut vpx_file = vpx::open(&expanded_path)?;
    let collections = vpx_file.read_collections()?;

    let rows: Vec<Vec<String>> = collections
        .iter()
        .map(|c| {
            vec![
                c.name.clone(),
                c.items.len().to_string(),
                if c.fire_events { "Y" } else { "N" }.to_string(),
                if c.stop_single_events { "Y" } else { "N" }.to_string(),
                if c.group_elements { "Y" } else { "N" }.to_string(),
            ]
        })
        .collect();

    let headers = [
        "NAME",
        "ITEMS",
        "FIRE_EVENTS",
        "STOP_SINGLES",
        "GROUP_ELEMENTS",
    ];
    let aligns = [
        ColAlign::Left,
        ColAlign::Right,
        ColAlign::Left,
        ColAlign::Left,
        ColAlign::Left,
    ];
    print_aligned_table(&headers, &aligns, &rows)?;
    Ok(ExitCode::SUCCESS)
}

fn handle_materials_list(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let path = sub_matches
        .get_one::<String>("VPXPATH")
        .map(|s| s.as_str())
        .unwrap_or_default();
    let expanded_path = path_exists(path)?;
    let mut vpx_file = vpx::open(&expanded_path)?;
    let gamedata = vpx_file.read_gamedata()?;

    // Build a uniform Vec<Vec<String>> from whichever material format is
    // present. 10.8+ uses the MATR storage (`gamedata.materials`); older
    // files use the legacy MATE storage (`gamedata.materials_old`).
    let rows: Vec<Vec<String>> = if let Some(materials) = gamedata.materials.as_ref() {
        materials
            .iter()
            .map(|m| {
                vec![
                    m.name.clone(),
                    format!(
                        "#{:02X}{:02X}{:02X}",
                        m.base_color.r, m.base_color.g, m.base_color.b
                    ),
                    if m.type_ == vpin::vpx::material::MaterialType::Metal {
                        "Y"
                    } else {
                        "N"
                    }
                    .to_string(),
                    format!("{:.3}", m.roughness),
                    format!("{:.3}", m.opacity),
                    format!("{:.3}", m.edge),
                ]
            })
            .collect()
    } else {
        gamedata
            .materials_old
            .iter()
            .map(|m| {
                vec![
                    m.name.clone(),
                    format!(
                        "#{:02X}{:02X}{:02X}",
                        m.base_color.r, m.base_color.g, m.base_color.b
                    ),
                    if m.is_metal { "Y" } else { "N" }.to_string(),
                    format!("{:.3}", m.roughness),
                    format!("{:.3}", m.opacity),
                    format!("{:.3}", m.edge),
                ]
            })
            .collect()
    };

    let headers = [
        "NAME",
        "BASE_COLOR",
        "METAL",
        "ROUGHNESS",
        "OPACITY",
        "EDGE",
    ];
    let aligns = [
        ColAlign::Left,
        ColAlign::Left,
        ColAlign::Left,
        ColAlign::Right,
        ColAlign::Right,
        ColAlign::Right,
    ];
    print_aligned_table(&headers, &aligns, &rows)?;
    Ok(ExitCode::SUCCESS)
}

fn handle_gameitems_list(sub_matches: &ArgMatches) -> io::Result<ExitCode> {
    let path = sub_matches
        .get_one::<String>("VPXPATH")
        .map(|s| s.as_str())
        .unwrap_or_default();
    let expanded_path = path_exists(path)?;
    let mut vpx_file = vpx::open(&expanded_path)?;
    let gameitems = vpx_file.read_gameitems()?;

    let rows: Vec<Vec<String>> = gameitems
        .iter()
        .map(|item| {
            let visible = match item.is_visible() {
                Some(true) => "Y",
                Some(false) => "N",
                None => "-",
            };
            let locked = match item.is_locked() {
                Some(true) => "Y",
                Some(false) => "N",
                None => "-",
            };
            // Prefer the editor layer name; fall back to the numeric layer if
            // unnamed, empty if neither is set.
            let layer = match item.editor_layer_name() {
                Some(name) if !name.is_empty() => name.clone(),
                _ => item
                    .editor_layer()
                    .map(|n| n.to_string())
                    .unwrap_or_default(),
            };
            vec![
                item.name().to_string(),
                item.type_name(),
                visible.to_string(),
                locked.to_string(),
                layer,
                item.part_group_name().unwrap_or("").to_string(),
                item.physics_material().unwrap_or("").to_string(),
                item.images().join("--"),
                item.materials().join("--"),
            ]
        })
        .collect();

    let headers = [
        "NAME",
        "TYPE",
        "VISIBLE",
        "LOCKED",
        "LAYER",
        "PART_GROUP",
        "PHYSICS_MATERIAL",
        "IMAGES",
        "MATERIALS",
    ];
    let aligns = [
        ColAlign::Left,
        ColAlign::Left,
        ColAlign::Left,
        ColAlign::Left,
        ColAlign::Left,
        ColAlign::Left,
        ColAlign::Left,
        ColAlign::Left,
        ColAlign::Left,
    ];
    print_aligned_table(&headers, &aligns, &rows)?;
    Ok(ExitCode::SUCCESS)
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
    let vpx_path = sub_matches.get_one::<String>("VPXPATH").map(PathBuf::from);
    let vbs_path = sub_matches.get_one::<String>("VBSPATH").map(PathBuf::from);
    let directory = sub_matches
        .get_one::<String>("OUTPUT_DIRECTORY")
        .map(PathBuf::from);
    let expanded_vpx_path = path_exists(vpx_path.expect("should be checked by clap"))?;
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

fn wite_images(b2s: directb2s::DirectB2SData, root_dir_path: &Path) {
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
        let options = ExpandOptions::default();
        expanded::write(&vpx, &root_dir_path, &options)
    };
    match result {
        Ok(_) => {
            crate::println!("Successfully extracted to \"{}\"", root_dir_path.display())?;
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => fail(format!("Failed to extract: {e}")),
    }
}

pub fn info_diff(vpx_file_path: &Path, config: Option<&ResolvedConfig>) -> io::Result<String> {
    let expanded_vpx_path = path_exists(vpx_file_path)?;
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
        let output = run_diff(
            original_info_path.path(),
            &info_file_path,
            diff_color,
            config,
        )?;
        Ok(String::from_utf8_lossy(&output).to_string())
    } else {
        let msg = format!("No sidecar info file found: {}", info_file_path.display());
        Err(io::Error::new(io::ErrorKind::NotFound, msg))
    }
}

pub fn script_diff(vpx_file_path: &Path, config: Option<&ResolvedConfig>) -> io::Result<String> {
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
                let output = run_diff(original_vbs_path.path(), &vbs_path, diff_color, config)?;
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
    config: Option<&ResolvedConfig>,
) -> Result<Vec<u8>, io::Error> {
    let original_vbs_filename = original_vbs_path
        .file_name()
        .unwrap_or(original_vbs_path.as_os_str());
    let original_vbs_file_name_no_tmp = original_vbs_filename.to_string_lossy().replace(".tmp", "");
    let vbs_filename = vbs_path.file_name().unwrap_or(vbs_path.as_os_str());
    let diff = config
        .and_then(|resolved| resolved.diff.as_deref())
        .unwrap_or("diff");
    let mut command = std::process::Command::new(diff);
    match vbs_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            command.current_dir(parent);
        }
        _ => {}
    }
    command
        .arg("-u")
        .arg("-w")
        .arg(format!("--color={}", color.to_diff_arg()))
        .arg(format!("--label={original_vbs_file_name_no_tmp}"))
        .arg(original_vbs_filename)
        .arg(format!("--label={}", vbs_filename.to_string_lossy()))
        .arg(vbs_filename);
    info!("Running command: {:?}", &command);
    let result = command.output().map(|o| o.stdout);
    result.map_err(|e| {
        let msg = format!("Failed to run diff '{diff}'. Is it installed on your system? {e}");
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

fn run_lock(sub_matches: &ArgMatches, action: LockAction) -> io::Result<ExitCode> {
    let path = sub_matches
        .get_one::<String>("VPXPATH")
        .expect("VPXPATH is required");
    let expanded = path_exists(path)?;
    apply_lock_action(&expanded, &action)?;
    Ok(ExitCode::SUCCESS)
}

fn apply_lock_action(path: &Path, action: &LockAction) -> io::Result<()> {
    match action {
        LockAction::Status => {
            let mut vpx = vpx::open(path)?;
            let state = if vpx.is_locked()? {
                "locked"
            } else {
                "unlocked"
            };
            crate::println!("{}: {}", path.display(), state)?;
        }
        LockAction::Lock => {
            let mut vpx = vpx::open_rw(path)?;
            if vpx.lock()? {
                crate::println!("Locked {}", path.display())?;
            } else {
                crate::println!("Already locked: {}", path.display())?;
            }
        }
        LockAction::Unlock => {
            let mut vpx = vpx::open_rw(path)?;
            if vpx.unlock()? {
                crate::println!("Unlocked {}", path.display())?;
            } else {
                crate::println!("Already unlocked: {}", path.display())?;
            }
        }
    }
    Ok(())
}
