use std::collections::HashSet;
use std::{
    fs::File,
    io,
    io::Write,
    path::{Path, PathBuf},
    process::{exit, ExitStatus},
};

use colored::Colorize;
use console::Emoji;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use is_executable::IsExecutable;

use crate::config::ResolvedConfig;
use crate::indexer::{IndexError, IndexedTable, Progress};
use crate::{
    diff_script, indexer, patch_vbs_file, run_diff,
    vpx::{extractvbs, vbs_path_for, ExtractResult},
    DiffColor, ProgressBarProgress,
};

const LAUNCH: Emoji = Emoji("🚀", "[launch]");
const CRASH: Emoji = Emoji("💥", "[crash]");

enum TableOption {
    Launch,
    LaunchFullscreen,
    LaunchWindowed,
    ShowDetails,
    ExtractVBS,
    EditVBS,
    PatchVBS,
    ShowVBSDiff,
    CreateVBSPatch,
    // ClearNVRAM,
}

impl TableOption {
    const ALL: [TableOption; 9] = [
        TableOption::Launch,
        TableOption::LaunchFullscreen,
        TableOption::LaunchWindowed,
        TableOption::ShowDetails,
        TableOption::ExtractVBS,
        TableOption::EditVBS,
        TableOption::PatchVBS,
        TableOption::ShowVBSDiff,
        TableOption::CreateVBSPatch,
        // TableOption::ClearNVRAM,
    ];

    fn from_index(index: usize) -> Option<TableOption> {
        match index {
            0 => Some(TableOption::Launch),
            1 => Some(TableOption::LaunchFullscreen),
            2 => Some(TableOption::LaunchWindowed),
            3 => Some(TableOption::ShowDetails),
            4 => Some(TableOption::ExtractVBS),
            5 => Some(TableOption::EditVBS),
            6 => Some(TableOption::PatchVBS),
            7 => Some(TableOption::ShowVBSDiff),
            8 => Some(TableOption::CreateVBSPatch),
            // 9 => Some(TableOption::ClearNVRAM),
            _ => None,
        }
    }

    fn display(&self) -> String {
        match self {
            TableOption::Launch => "Launch".to_string(),
            TableOption::LaunchFullscreen => "Launch fullscreen".to_string(),
            TableOption::LaunchWindowed => "Launch windowed".to_string(),
            TableOption::ShowDetails => "Show details".to_string(),
            TableOption::ExtractVBS => "VBScript > Extract".to_string(),
            TableOption::EditVBS => "VBScript > Edit".to_string(),
            TableOption::PatchVBS => "VBScript > Patch typical standalone issues".to_string(),
            TableOption::ShowVBSDiff => "VBScript > Diff".to_string(),
            TableOption::CreateVBSPatch => "VBScript > Create patch file".to_string(),
            // TableOption::ClearNVRAM => "Clear NVRAM".to_string(),
        }
    }
}

pub fn frontend_index(
    resolved_config: &ResolvedConfig,
    recursive: bool,
) -> Result<Vec<IndexedTable>, IndexError> {
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
        &resolved_config.tables_folder,
        &resolved_config.tables_index_path,
        &progress,
    );
    progress.finish_and_clear();
    let index = index?;

    let mut tables: Vec<IndexedTable> = index.tables();
    tables.sort_by_key(|indexed| display_table_line(&indexed).to_lowercase());
    Ok(tables)
}

pub fn frontend(
    vpx_files_with_tableinfo: &Vec<IndexedTable>,
    roms: &HashSet<String>,
    vpinball_executable: &Path,
) {
    let mut selection_opt = None;
    loop {
        let selections = vpx_files_with_tableinfo
            .iter()
            // TODO can we expand the tuple to args?
            .map(|indexed| display_table_line_full(indexed, roms))
            .collect::<Vec<String>>();

        // TODO check FuzzySelect, requires feature to be enabled
        selection_opt = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a table to launch")
            .default(selection_opt.unwrap_or(0))
            .items(&selections[..])
            .interact_opt()
            .unwrap();

        match selection_opt {
            Some(selection) => {
                let info = vpx_files_with_tableinfo.get(selection).unwrap();
                let selected_path = &info.path;
                match choose_table_option() {
                    Some(TableOption::Launch) => {
                        launch(selected_path, vpinball_executable, None);
                    }
                    Some(TableOption::LaunchFullscreen) => {
                        launch(selected_path, vpinball_executable, Some(true));
                    }
                    Some(TableOption::LaunchWindowed) => {
                        launch(selected_path, vpinball_executable, Some(false));
                    }
                    Some(TableOption::EditVBS) => {
                        let path = vbs_path_for(selected_path);
                        if path.exists() {
                            open(path);
                        } else {
                            extractvbs(selected_path, false, None);
                            open(path);
                        }
                    }
                    Some(TableOption::ExtractVBS) => match extractvbs(selected_path, false, None) {
                        ExtractResult::Extracted(path) => {
                            prompt(format!("VBS extracted to {}", path.to_string_lossy()));
                        }
                        ExtractResult::Existed(path) => {
                            let msg = format!("VBS already exists at {}", path.to_string_lossy());
                            prompt(msg.truecolor(255, 125, 0).to_string());
                        }
                    },
                    Some(TableOption::ShowVBSDiff) => match diff_script(selected_path) {
                        Ok(diff) => {
                            prompt(diff);
                        }
                        Err(err) => {
                            let msg = format!("Unable to diff VBS: {}", err);
                            prompt(msg.truecolor(255, 125, 0).to_string());
                        }
                    },
                    Some(TableOption::PatchVBS) => {
                        let vbs_path = match extractvbs(selected_path, false, Some("vbs")) {
                            ExtractResult::Existed(path) => path,
                            ExtractResult::Extracted(path) => path,
                        };
                        match patch_vbs_file(&vbs_path) {
                            Ok(_) => {
                                prompt(format!(
                                    "Patched VBS file at {}",
                                    vbs_path.to_string_lossy()
                                ));
                            }
                            Err(err) => {
                                let msg = format!("Unable to patch VBS: {}", err);
                                prompt(msg.truecolor(255, 125, 0).to_string());
                            }
                        }
                    }
                    Some(TableOption::CreateVBSPatch) => {
                        let original_path =
                            match extractvbs(selected_path, true, Some("vbs.original")) {
                                ExtractResult::Existed(path) => path,
                                ExtractResult::Extracted(path) => path,
                            };
                        let vbs_path = vbs_path_for(selected_path);
                        let patch_path = vbs_path.with_extension("vbs.patch");

                        match run_diff(&original_path, &vbs_path, DiffColor::Never) {
                            Ok(diff) => {
                                let mut file = File::create(&patch_path).unwrap();
                                file.write_all(&diff).unwrap();
                            }
                            Err(err) => {
                                let msg = format!("Unable to diff VBS: {}", err);
                                prompt(msg.truecolor(255, 125, 0).to_string());
                            }
                        }
                    }
                    Some(TableOption::ShowDetails) => match gather_table_info(selected_path) {
                        Ok(info) => {
                            prompt(info);
                        }
                        Err(err) => {
                            let msg = format!("Unable to gather table info: {}", err);
                            prompt(msg.truecolor(255, 125, 0).to_string());
                        }
                    },
                    None => (),
                }
            }
            None => break,
        };
    }
}

fn gather_table_info(selected_path: &PathBuf) -> io::Result<String> {
    let mut vpx_file = vpin::vpx::open(selected_path)?;
    let version = vpx_file.read_version()?;
    let table_info = vpx_file.read_tableinfo()?;
    let msg = format!("version: {:#?}\n{:#?}", version, table_info);
    Ok(msg)
}

fn open(path: PathBuf) {
    open::that(&path)
        .unwrap_or_else(|err| prompt(format!("Unable to open {} {err}", path.to_string_lossy())));
}

fn prompt<S: Into<String>>(msg: S) {
    Input::<String>::new()
        .with_prompt(format!("{} - Press enter to continue.", msg.into()))
        .default("".to_string())
        .show_default(false)
        .interact()
        .unwrap();
}

fn choose_table_option() -> Option<TableOption> {
    // iterate over table options
    let selections = TableOption::ALL
        .iter()
        .map(|option| option.display())
        .collect::<Vec<String>>();

    let selection_opt = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose an option")
        .default(0)
        .items(&selections[..])
        .interact_opt()
        .unwrap();

    selection_opt.and_then(TableOption::from_index)
}

fn launch(selected_path: &PathBuf, vpinball_executable: &Path, fullscreen: Option<bool>) {
    println!("{} {}", LAUNCH, selected_path.display());

    if !vpinball_executable.is_executable() {
        report_and_exit(format!(
            "Unable to launch table, {} is not executable",
            vpinball_executable.display()
        ));
    }

    match launch_table(selected_path, vpinball_executable, fullscreen) {
        Ok(status) => match status.code() {
            Some(0) => {
                //println!("Table exited normally");
            }
            Some(11) => {
                eprintln!("{} Table exited with segfault, you might want to report this to the vpinball team.", CRASH);
            }
            Some(139) => {
                eprintln!("{} Table exited with segfault, you might want to report this to the vpinball team.", CRASH);
            }
            Some(code) => {
                eprintln!("Table exited with code {}", code);
            }
            None => {
                eprintln!("Table exited with unknown code");
            }
        },
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                report_and_exit(format!(
                    "Unable to launch table, vpinball executable not found at {}",
                    vpinball_executable.display()
                ));
            } else {
                report_and_exit(format!("Unable to launch table: {:?}", e));
            }
        }
    }
}

fn report_and_exit(msg: String) -> ! {
    eprintln!("{CRASH} {}", msg);
    exit(1);
}

fn launch_table(
    selected_path: &PathBuf,
    vpinball_executable: &Path,
    fullscreen: Option<bool>,
) -> io::Result<ExitStatus> {
    // start process ./VPinballX_GL -play [table path]
    let mut cmd = std::process::Command::new(vpinball_executable);
    match fullscreen {
        Some(true) => {
            cmd.arg("-EnableTrueFullscreen");
        }
        Some(false) => {
            cmd.arg("-DisableTrueFullscreen");
        }
        None => (),
    }
    cmd.arg("-play");
    cmd.arg(selected_path);
    let mut child = cmd.spawn()?;
    let result = child.wait()?;
    Ok(result)
}

fn display_table_line(table: &IndexedTable) -> String {
    let file_name = table
        .path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    Some(table.table_info.table_name.to_owned())
        .filter(|s| !s.clone().unwrap_or_default().is_empty())
        .map(|s| {
            format!(
                "{} {}",
                capitalize_first_letter(s.unwrap_or_default().as_str()),
                (format!("({})", file_name)).dimmed()
            )
        })
        .unwrap_or(file_name)
}

fn display_table_line_full(table: &IndexedTable, roms: &HashSet<String>) -> String {
    let base = display_table_line(table);
    let suffix = match &table.game_name {
        Some(name) => {
            let rom_found = roms.contains(&name.to_lowercase());
            if rom_found {
                format!(" - [{}]", name.dimmed())
            } else {
                if table.requires_pinmame {
                    format!(" - {} [{}]", Emoji("⚠️", "!"), &name)
                        .yellow()
                        .to_string()
                } else {
                    format!(" - [{}]", name.dimmed())
                }
            }
        }
        None => "".to_string(),
    };
    format!("{}{}", base, suffix)
}

fn capitalize_first_letter(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
}
