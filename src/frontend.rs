use std::{
    fs::File,
    io::{Result, Write},
    path::{Path, PathBuf},
    process::{exit, ExitStatus},
};

use colored::Colorize;
use console::Emoji;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    indexer, tableinfo,
    vpx::{self, extractvbs, vbs_path_for, version, ExtractResult},
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
    CreateVBSPatch,
    // ClearNVRAM,
}

impl TableOption {
    const ALL: [TableOption; 7] = [
        TableOption::LaunchFullscreen,
        TableOption::LaunchWindowed,
        TableOption::ShowDetails,
        TableOption::ExtractVBS,
        TableOption::EditVBS,
        TableOption::ShowVBSDiff,
        TableOption::CreateVBSPatch,
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
            6 => Some(TableOption::CreateVBSPatch),
            // 7 => Some(TableOption::ClearNVRAM),
            _ => None,
        }
    }

    fn display(&self) -> String {
        match self {
            TableOption::LaunchFullscreen => "Launch Fullscreen".to_string(),
            TableOption::LaunchWindowed => "Launch Windowed".to_string(),
            TableOption::ShowDetails => "Show Details".to_string(),
            TableOption::ExtractVBS => "VBScript > Extract".to_string(),
            TableOption::EditVBS => "VBScript > Edit".to_string(),
            TableOption::ShowVBSDiff => "VBScript > Diff".to_string(),
            TableOption::CreateVBSPatch => "VBScript > Create Patch".to_string(),
            // TableOption::ClearNVRAM => "Clear NVRAM".to_string(),
        }
    }
}

pub fn frontend_index(
    tables_path: PathBuf,
    recursive: bool,
) -> Vec<(PathBuf, tableinfo::TableInfo)> {
    println!("Indexing {}", &tables_path.display());
    let vpx_files = indexer::find_vpx_files(recursive, &tables_path).unwrap();
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
    vpx_files_with_tableinfo
        .sort_by_key(|(path1, info1)| display_table_line(path1, info1).to_lowercase());
    vpx_files_with_tableinfo
}

pub fn frontend(
    vpx_files_with_tableinfo: Vec<(PathBuf, tableinfo::TableInfo)>,
    vpinball_executable: &Path,
) {
    let mut selection_opt = None;
    loop {
        let selections = vpx_files_with_tableinfo
            .iter()
            // TODO can we expand the tuple to args?
            .map(|(path, info)| display_table_line(path, info))
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
                let (selected_path, _selected_info) =
                    vpx_files_with_tableinfo.get(selection).unwrap();

                match choose_table_option() {
                    Some(TableOption::LaunchFullscreen) => {
                        launch(selected_path, vpinball_executable, true);
                    }
                    Some(TableOption::LaunchWindowed) => {
                        launch(selected_path, vpinball_executable, false);
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
                    Some(TableOption::ShowVBSDiff) => match vpx::diff_script(selected_path) {
                        Ok(diff) => {
                            prompt(diff);
                        }
                        Err(err) => {
                            let msg = format!("Unable to diff VBS: {}", err);
                            prompt(msg.truecolor(255, 125, 0).to_string());
                        }
                    },
                    Some(TableOption::CreateVBSPatch) => {
                        let original_path =
                            match vpx::extractvbs(selected_path, true, Some("vbs.original")) {
                                ExtractResult::Existed(path) => path,
                                ExtractResult::Extracted(path) => path,
                            };
                        let vbs_path = vbs_path_for(selected_path);
                        let patch_path = vbs_path.with_extension("vbs.patch");

                        match vpx::run_diff(&original_path, &vbs_path, vpx::DiffColor::Never) {
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

fn gather_table_info(selected_path: &PathBuf) -> Result<String> {
    let mut comp = cfb::open(selected_path)?;
    let version = version::read_version(&mut comp)?;
    let table_info = tableinfo::read_tableinfo(&mut comp)?;
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

fn launch(selected_path: &PathBuf, vpinball_executable: &Path, fullscreen: bool) {
    println!("{} {}", LAUNCH, selected_path.display());
    match launch_table(selected_path, vpinball_executable, fullscreen) {
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
    println!("{CRASH} {}", msg);
    exit(1);
}

fn launch_table(
    selected_path: &PathBuf,
    vpinball_executable: &Path,
    fullscreen: bool,
) -> Result<ExitStatus> {
    // start process ./VPinballX_GL -play [table path]
    let mut cmd = std::process::Command::new(vpinball_executable);
    if fullscreen {
        cmd.arg("-EnableTrueFullscreen");
    } else {
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

fn capitalize_first_letter(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
}
