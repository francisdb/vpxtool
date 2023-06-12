use std::{
    io,
    path::{Path, PathBuf},
    process::ExitStatus,
};

use colored::Colorize;
use console::Emoji;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{indexer, tableinfo};

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
                let launch = Emoji("🚚 ", "[launch]");
                let crash = Emoji("💥 ", "[crash]");

                let (selected_path, _selected_info) =
                    vpx_files_with_tableinfo.get(selection).unwrap();

                println!("{} {}", launch, selected_path.display());
                match launch_table(selected_path, vpinball_root) {
                    Ok(status) => match status.code() {
                        Some(0) => {
                            //println!("Table exited normally");
                        }
                        Some(11) => {
                            println!("{} Table exited with segfault, you might want to report this to the vpinball team.", crash);
                        }
                        Some(139) => {
                            println!("{} Table exited with segfault, you might want to report this to the vpinball team.", crash);
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
            None => break,
        };
    }
}

fn launch_table(selected_path: &PathBuf, vpinball_root: &Path) -> io::Result<ExitStatus> {
    let executable = vpinball_root.join("vpinball").join("VPinballX_GL");

    // start process ./VPinballX_GL -play [table path]
    let mut cmd = std::process::Command::new(executable);
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
