use std::{
    ffi::OsStr,
    fs::{self, File},
    path::PathBuf,
};

use colored::Colorize;
use serde_json::json;
use walkdir::WalkDir;

use crate::{
    jsonmodel::table_json,
    tableinfo::{read_tableinfo, TableInfo},
};

pub fn find_vpx_files(recursive: bool, tables_path: &str) -> Vec<PathBuf> {
    let mut vpx_files = Vec::new();
    if recursive {
        for entry in WalkDir::new(tables_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some("vpx") = entry.path().extension().and_then(OsStr::to_str) {
                    vpx_files.push(entry.path().to_owned());
                }
            }
        }
    } else {
        for entry in fs::read_dir(tables_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                if let Some("vpx") = path.extension().and_then(OsStr::to_str) {
                    vpx_files.push(path);
                }
            }
        }
    }
    vpx_files
}

pub fn index_vpx_files(
    vpx_files: &[PathBuf],
    progress: impl Fn(u64),
) -> Vec<(PathBuf, TableInfo)> {
    // TODO tried using rayon here but it's not faster and uses up all cpu
    // use rayon::prelude::*;
    // .par_iter() instead of .iter()
    let mut vpx_files_with_tableinfo: Vec<(PathBuf, TableInfo)> = vpx_files
        .iter()
        .enumerate()
        .flat_map(|(i, vpx_file)| {
            let result = match cfb::open(vpx_file) {
                Ok(mut comp) => {
                    let table_info = read_tableinfo(&mut comp);
                    Some((vpx_file.clone(), table_info))
                }
                Err(e) => {
                    // TODO we want to return any failures instead of printing here
                    let warning =
                        format!("Not a valid vpx file {}: {}", vpx_file.display(), e).red();
                    println!("{}", warning);
                    None
                }
            };
            progress((i + 1) as u64);
            result
        })
        .collect();

    // sort by name
    vpx_files_with_tableinfo.sort_by(|(_, a), (_, b)| {
        a.table_name
            .to_lowercase()
            .cmp(&b.table_name.to_lowercase())
    });
    vpx_files_with_tableinfo
}

pub fn write_index_json(vpx_files_with_tableinfo: Vec<(PathBuf, TableInfo)>, json_path: PathBuf) {
    let json_items: Vec<serde_json::Value> = vpx_files_with_tableinfo
        .iter()
        .map(|(path, info)| {
            match table_json(info) {
                serde_json::Value::Object(mut props) => {
                    props.insert("path".to_string(), json!(path));
                    serde_json::Value::Object(props)
                }
                other => other, // This is unexpected, maybe we need to fail here, is there a cleaner solution?
            }
        })
        .collect();

    let json_tables = serde_json::Value::Array(json_items);
    let json = json!({ "tables": json_tables });

    let json_file = File::create(json_path).unwrap();
    serde_json::to_writer_pretty(json_file, &json).unwrap();
}
