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
use dialoguer::{FuzzySelect, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use is_executable::IsExecutable;

use crate::config::ResolvedConfig;
use crate::indexer::{IndexError, IndexedTable, Progress};
use crate::patcher::LineEndingsResult::{NoChanges, Unified};
use crate::patcher::{patch_vbs_file, unify_line_endings_vbs_file};
use crate::{
    indexer, info_diff, info_edit, info_gather, run_diff, script_diff,
    vpx::{extractvbs, vbs_path_for, ExtractResult},
    DiffColor, ProgressBarProgress,
};

const LAUNCH: Emoji = Emoji("🚀", "[launch]");
const CRASH: Emoji = Emoji("💥", "[crash]");

const SEARCH: &str = "> Search";
const RECENT: &str = "> Recent";
const SEARCH_INDEX: usize = 0;
const RECENT_INDEX: usize = 1;

enum TableOption {
    Launch,
    LaunchFullscreen,
    LaunchWindowed,
    ForceReload,
    InfoShow,
    InfoEdit,
    InfoDiff,
    ExtractVBS,
    EditVBS,
    PatchVBS,
    UnifyLineEndings,
    ShowVBSDiff,
    CreateVBSPatch,
    // ClearNVRAM,
}

impl TableOption {
    const ALL: [TableOption; 13] = [
        TableOption::Launch,
        TableOption::LaunchFullscreen,
        TableOption::LaunchWindowed,
        TableOption::ForceReload,
        TableOption::InfoShow,
        TableOption::InfoEdit,
        TableOption::InfoDiff,
        TableOption::ExtractVBS,
        TableOption::EditVBS,
        TableOption::PatchVBS,
        TableOption::UnifyLineEndings,
        TableOption::ShowVBSDiff,
        TableOption::CreateVBSPatch,
        // TableOption::ClearNVRAM,
    ];

    fn from_index(index: usize) -> Option<TableOption> {
        match index {
            0 => Some(TableOption::Launch),
            1 => Some(TableOption::LaunchFullscreen),
            2 => Some(TableOption::LaunchWindowed),
            3 => Some(TableOption::ForceReload),
            4 => Some(TableOption::InfoShow),
            5 => Some(TableOption::InfoEdit),
            6 => Some(TableOption::InfoDiff),
            7 => Some(TableOption::ExtractVBS),
            8 => Some(TableOption::EditVBS),
            9 => Some(TableOption::PatchVBS),
            10 => Some(TableOption::UnifyLineEndings),
            11 => Some(TableOption::ShowVBSDiff),
            12 => Some(TableOption::CreateVBSPatch),
            // 13 => Some(TableOption::ClearNVRAM),
            _ => None,
        }
    }

    fn display(&self) -> String {
        match self {
            TableOption::Launch => "Launch".to_string(),
            TableOption::LaunchFullscreen => "Launch fullscreen".to_string(),
            TableOption::LaunchWindowed => "Launch windowed".to_string(),
            TableOption::ForceReload => "Force reload".to_string(),
            TableOption::InfoShow => "Info > Show".to_string(),
            TableOption::InfoEdit => "Info > Edit".to_string(),
            TableOption::InfoDiff => "Info > Diff".to_string(),
            TableOption::ExtractVBS => "VBScript > Extract".to_string(),
            TableOption::EditVBS => "VBScript > Edit".to_string(),
            TableOption::PatchVBS => "VBScript > Patch typical standalone issues".to_string(),
            TableOption::UnifyLineEndings => "VBScript > Unify line endings".to_string(),
            TableOption::ShowVBSDiff => "VBScript > Diff".to_string(),
            TableOption::CreateVBSPatch => "VBScript > Create patch file".to_string(),
            // TableOption::ClearNVRAM => "Clear NVRAM".to_string(),
        }
    }
}

pub fn frontend_index(
    resolved_config: &ResolvedConfig,
    recursive: bool,
    force_reindex: Vec<PathBuf>,
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
        force_reindex,
    );
    progress.finish_and_clear();
    let index = index?;

    let mut tables: Vec<IndexedTable> = index.tables();
    tables.sort_by_key(|indexed| display_table_line(indexed).to_lowercase());
    Ok(tables)
}

pub fn frontend(
    config: &ResolvedConfig,
    mut vpx_files_with_tableinfo: Vec<IndexedTable>,
    roms: &HashSet<String>,
    vpinball_executable: &Path,
) {
    let mut main_selection_opt = None;
    loop {
        let tables: Vec<String> = vpx_files_with_tableinfo
            .iter()
            .map(|indexed| display_table_line_full(indexed, roms))
            .collect();

        let mut selections = vec![SEARCH.bold().to_string(), RECENT.bold().to_string()];
        selections.extend(tables.clone());

        main_selection_opt = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a table")
            .default(main_selection_opt.unwrap_or(0))
            .items(&selections[..])
            .interact_opt()
            .unwrap();

        match main_selection_opt {
            Some(selection) => {
                // search

                match selection {
                    SEARCH_INDEX => {
                        // show a fuzzy search
                        let selected = FuzzySelect::with_theme(&ColorfulTheme::default())
                            .with_prompt("Search a table:")
                            .items(&tables)
                            .interact_opt()
                            .unwrap();

                        if let Some(selected_index) = selected {
                            let info = vpx_files_with_tableinfo
                                .get(selected_index)
                                .unwrap()
                                .clone();
                            let info_str = display_table_line_full(&info, roms);
                            table_menu(
                                config,
                                &mut vpx_files_with_tableinfo,
                                vpinball_executable,
                                &info,
                                &info_str,
                            );
                        }
                    }
                    RECENT_INDEX => {
                        // take the last 10 most recent tables
                        let mut recent: Vec<IndexedTable> = vpx_files_with_tableinfo.clone();
                        recent.sort_by_key(|indexed| indexed.last_modified);
                        let last_modified = recent.iter().rev().take(50).collect::<Vec<_>>();
                        let last_modified_str: Vec<String> = last_modified
                            .iter()
                            .map(|indexed| display_table_line_full(indexed, roms))
                            .collect();

                        let selected = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Select a table")
                            .items(&last_modified_str)
                            .interact_opt()
                            .unwrap();

                        if let Some(selected_index) = selected {
                            let info = last_modified.get(selected_index).unwrap();
                            let info_str = display_table_line_full(info, roms);
                            table_menu(
                                config,
                                &mut vpx_files_with_tableinfo,
                                vpinball_executable,
                                info,
                                &info_str,
                            );
                        }
                    }
                    _ => {
                        let index = selection - 2;

                        let info = vpx_files_with_tableinfo.get(index).unwrap().clone();
                        let info_str = display_table_line_full(&info, roms);
                        table_menu(
                            config,
                            &mut vpx_files_with_tableinfo,
                            vpinball_executable,
                            &info,
                            &info_str,
                        );
                    }
                }
            }
            None => break,
        };
    }
}

fn table_menu(
    config: &ResolvedConfig,
    vpx_files_with_tableinfo: &mut Vec<IndexedTable>,
    vpinball_executable: &Path,
    info: &IndexedTable,
    info_str: &str,
) {
    let selected_path = &info.path;
    match choose_table_option(info_str) {
        Some(TableOption::Launch) => {
            launch(selected_path, vpinball_executable, None);
        }
        Some(TableOption::LaunchFullscreen) => {
            launch(selected_path, vpinball_executable, Some(true));
        }
        Some(TableOption::LaunchWindowed) => {
            launch(selected_path, vpinball_executable, Some(false));
        }
        Some(TableOption::ForceReload) => {
            match frontend_index(config, true, vec![selected_path.clone()]) {
                Ok(index) => {
                    vpx_files_with_tableinfo.clear();
                    vpx_files_with_tableinfo.extend(index);
                }
                Err(err) => {
                    let msg = format!("Unable to reload tables: {:?}", err);
                    prompt(msg.truecolor(255, 125, 0).to_string());
                }
            }
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
        Some(TableOption::ShowVBSDiff) => match script_diff(selected_path) {
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
                Ok(applied) => {
                    if applied.is_empty() {
                        prompt("No patches applied.".to_string());
                    } else {
                        applied.iter().for_each(|patch| {
                            println!("Applied patch: {}", patch);
                        });
                        prompt(format!(
                            "Patched VBS file at {}",
                            vbs_path.to_string_lossy()
                        ));
                    }
                }
                Err(err) => {
                    let msg = format!("Unable to patch VBS: {}", err);
                    prompt(msg.truecolor(255, 125, 0).to_string());
                }
            }
        }
        Some(TableOption::UnifyLineEndings) => {
            let vbs_path = match extractvbs(selected_path, false, Some("vbs")) {
                ExtractResult::Existed(path) => path,
                ExtractResult::Extracted(path) => path,
            };
            match unify_line_endings_vbs_file(&vbs_path) {
                Ok(NoChanges) => {
                    prompt("No changes applied as file has correct line endings".to_string());
                }
                Ok(Unified) => {
                    prompt(format!(
                        "Unified line endings in VBS file at {}",
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
            let original_path = match extractvbs(selected_path, true, Some("vbs.original")) {
                ExtractResult::Existed(path) => path,
                ExtractResult::Extracted(path) => path,
            };
            let vbs_path = vbs_path_for(selected_path);
            let patch_path = vbs_path.with_extension("vbs.patch");

            match run_diff(&original_path, &vbs_path, DiffColor::Never) {
                Ok(diff) => {
                    let mut file = File::create(patch_path).unwrap();
                    file.write_all(&diff).unwrap();
                }
                Err(err) => {
                    let msg = format!("Unable to diff VBS: {}", err);
                    prompt(msg.truecolor(255, 125, 0).to_string());
                }
            }
        }
        Some(TableOption::InfoShow) => match info_gather(selected_path) {
            Ok(info) => {
                prompt(info);
            }
            Err(err) => {
                let msg = format!("Unable to gather table info: {}", err);
                prompt(msg.truecolor(255, 125, 0).to_string());
            }
        },
        Some(TableOption::InfoEdit) => match do_info_edit(selected_path) {
            Ok(path) => {
                println!("Launched editor for {}", path.display());
            }
            Err(err) => {
                let msg = format!("Unable to edit table info: {}", err);
                prompt(msg.truecolor(255, 125, 0).to_string());
            }
        },
        Some(TableOption::InfoDiff) => match info_diff(selected_path) {
            Ok(diff) => {
                prompt(diff);
            }
            Err(err) => {
                let msg = format!("Unable to diff info: {}", err);
                prompt(msg.truecolor(255, 125, 0).to_string());
            }
        },
        None => (),
    }
}

fn do_info_edit(selected_path: &PathBuf) -> io::Result<PathBuf> {
    info_edit(selected_path)
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

fn choose_table_option(table_name: &str) -> Option<TableOption> {
    // iterate over table options
    let selections = TableOption::ALL
        .iter()
        .map(|option| option.display())
        .collect::<Vec<String>>();

    let selection_opt = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(table_name)
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
                prompt(format!("{} Visual Pinball exited with segfault, you might want to report this to the vpinball team.", CRASH));
            }
            Some(139) => {
                prompt(format!("{} Visual Pinball exited with segfault, you might want to report this to the vpinball team.", CRASH));
            }
            Some(code) => {
                prompt(format!(
                    "{} Visual Pinball exited with code {}",
                    CRASH, code
                ));
            }
            None => {
                prompt("Visual Pinball exited with unknown code");
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
    let gamename_suffix = match &table.game_name {
        Some(name) => {
            let rom_found = table.local_rom_path.is_some() || roms.contains(&name.to_lowercase());
            if rom_found {
                format!(" - [{}]", name.dimmed())
            } else if table.requires_pinmame {
                format!(" - {} [{}]", Emoji("⚠️", "!"), &name)
                    .yellow()
                    .to_string()
            } else {
                format!(" - [{}]", name.dimmed())
            }
        }
        None => "".to_string(),
    };
    let b2s_suffix = match &table.b2s_path {
        Some(_) => " ▀".dimmed(),
        None => "".into(),
    };
    format!("{}{}{}", base, gamename_suffix, b2s_suffix)
}

fn capitalize_first_letter(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
}
