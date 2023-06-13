use std::{
    io,
    path::{Path, PathBuf},
    process::ExitStatus,
};

use colored::Colorize;
use console::Emoji;
use dialoguer::{theme::ColorfulTheme, Confirm};
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    indexer, tableinfo,
    vpx::{self, extractvbs, vbs_path_for, ExtractResult},
};

const LAUNCH: Emoji = Emoji("🚀", "[launch]");
const CRASH: Emoji = Emoji("💥", "[crash]");

enum TableOption {
    LaunchFullscreen,
    LaunchWindowed,
    ShowDetails,
    ExtractVBS,
    EditVBS,
    ShowVBSDiff,
    // ClearNVRAM,
}

impl TableOption {
    const ALL: [TableOption; 6] = [
        TableOption::LaunchFullscreen,
        TableOption::LaunchWindowed,
        TableOption::ShowDetails,
        TableOption::ExtractVBS,
        TableOption::EditVBS,
        TableOption::ShowVBSDiff,
        // TableOption::ClearNVRAM,
    ];

    fn from_index(index: usize) -> Option<TableOption> {
        match index {
            0 => Some(TableOption::LaunchFullscreen),
            1 => Some(TableOption::LaunchWindowed),
            2 => Some(TableOption::ShowDetails),
            3 => Some(TableOption::ExtractVBS),
            4 => Some(TableOption::EditVBS),
            5 => Some(TableOption::ShowVBSDiff),
            // 6 => Some(TableOption::ClearNVRAM),
            _ => None,
        }
    }

    fn display(&self) -> String {
        match self {
            TableOption::LaunchFullscreen => "Launch Fullscreen".to_string(),
            TableOption::LaunchWindowed => "Launch Windowed".to_string(),
            TableOption::ShowDetails => "Show Details".to_string(),
            TableOption::ExtractVBS => "VBScript - Extract".to_string(),
            TableOption::EditVBS => "VBScript - Edit".to_string(),
            TableOption::ShowVBSDiff => "VBScript - Diff".to_string(),
            // TableOption::ClearNVRAM => "Clear NVRAM".to_string(),
        }
    }
}

pub fn frontend_index(
    tables_path: String,
    recursive: bool,
) -> Vec<(PathBuf, tableinfo::TableInfo)> {
    println!("Indexing {}", tables_path);
    let vpx_files = indexer::find_vpx_files(recursive, &tables_path);
    let pb = ProgressBar::new(vpx_files.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{bar:.cyan/blue}] {pos}/{human_len} ({eta})",
        )
        .unwrap(),
    );
    let mut vpx_files_with_tableinfo = indexer::index_vpx_files(&vpx_files, |pos: u64| {
        pb.set_position(pos);
    });
    pb.finish_and_clear();

    // TODO this is a second sort, does not make a lot of sense to do the first one
    vpx_files_with_tableinfo.sort_by_key(|(path1, info1)| display_table_line(path1, info1));
    vpx_files_with_tableinfo
}

pub fn frontend(
    vpx_files_with_tableinfo: Vec<(PathBuf, tableinfo::TableInfo)>,
    vpinball_root: &Path,
) {
    let mut selection_opt = None;
    loop {
        let selections = vpx_files_with_tableinfo
            .iter()
            // TODO can we expand the tuple to args?
            .map(|(path, info)| display_table_line(path, info))
            .collect::<Vec<String>>();

        selection_opt = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a table to launch")
            .default(selection_opt.unwrap_or(0))
            .items(&selections[..])
            .interact_opt()
            .unwrap();

        match selection_opt {
            Some(selection) => {
                let (selected_path, _selected_info) =
                    vpx_files_with_tableinfo.get(selection).unwrap();

                match choose_table_option() {
                    Some(TableOption::LaunchFullscreen) => {
                        launch(selected_path, vpinball_root, true);
                    }
                    Some(TableOption::LaunchWindowed) => {
                        launch(selected_path, vpinball_root, false);
                    }
                    Some(TableOption::EditVBS) => {
                        let path = vbs_path_for(&selected_path.to_string_lossy());
                        if path.exists() {
                            open(path);
                        } else {
                            extractvbs(&selected_path.to_string_lossy(), false);
                            open(path);
                        }
                    }
                    Some(TableOption::ExtractVBS) => {
                        match extractvbs(&selected_path.to_string_lossy(), false) {
                            ExtractResult::Extracted(path) => {
                                prompt(format!("VBS extracted to {}", path.to_string_lossy()));
                            }
                            ExtractResult::Existed(path) => {
                                let msg =
                                    format!("VBS already exists at {}", path.to_string_lossy());
                                prompt(msg.truecolor(255, 125, 0).to_string());
                            }
                        }
                    }
                    Some(TableOption::ShowVBSDiff) => {
                        match vpx::diff(&selected_path.to_string_lossy()) {
                            Ok(diff) => {
                                prompt(diff);
                            }
                            Err(err) => {
                                let msg = format!("Unable to diff VBS: {}", err);
                                prompt(msg.truecolor(255, 125, 0).to_string());
                            }
                        }
                    }
                    Some(TableOption::ShowDetails) => {
                        prompt("Not implemented");
                    }
                    None => (),
                }
            }
            None => break,
        };
    }
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

fn launch(selected_path: &PathBuf, vpinball_root: &Path, fullscreen: bool) {
    println!("{} {}", LAUNCH, selected_path.display());
    match launch_table(selected_path, vpinball_root, fullscreen) {
        Ok(status) => match status.code() {
            Some(0) => {
                //println!("Table exited normally");
            }
            Some(11) => {
                println!("{} Table exited with segfault, you might want to report this to the vpinball team.", CRASH);
            }
            Some(139) => {
                println!("{} Table exited with segfault, you might want to report this to the vpinball team.", CRASH);
            }
            Some(code) => {
                println!("Table exited with code {}", code);
            }
            None => {
                println!("Table exited with unknown code");
            }
        },
        Err(e) => {
            println!("Error launching table: {:?}", e);
        }
    }
}

fn launch_table(
    selected_path: &PathBuf,
    vpinball_root: &Path,
    fullscreen: bool,
) -> io::Result<ExitStatus> {
    let executable = vpinball_root.join("vpinball").join("VPinballX_GL");

    // start process ./VPinballX_GL -play [table path]
    let mut cmd = std::process::Command::new(executable);
    if !fullscreen {
        cmd.arg("-DisableTrueFullscreen");
    }
    cmd.arg("-play");
    cmd.arg(selected_path);
    let mut child = cmd.spawn()?;
    let result = child.wait()?;
    Ok(result)
}

fn display_table_line(path: &Path, info: &tableinfo::TableInfo) -> String {
    let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
    Some(info.table_name.to_owned())
        .filter(|s| !s.is_empty())
        .map(|s| format!("{} {}", s, (format!("({})", file_name)).dimmed()))
        .unwrap_or(file_name)
}
