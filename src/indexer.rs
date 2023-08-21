use std::{
    ffi::OsStr,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use colored::Colorize;
use serde_json::json;
use walkdir::WalkDir;

use crate::{
    jsonmodel::table_json,
    tableinfo::{read_tableinfo, TableInfo},
};

pub fn find_vpx_files<P: AsRef<Path>>(recursive: bool, tables_path: P) -> io::Result<Vec<PathBuf>> {
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
        let dirs = fs::read_dir(tables_path)?;
        dirs.map(|entry| {
            let dir_entry = entry?;
            let path = dir_entry.path();
            if path.is_file() {
                if let Some("vpx") = path.extension().and_then(OsStr::to_str) {
                    vpx_files.push(path);
                }
            }
            Ok(())
        })
        .collect::<io::Result<()>>()?;
    }
    Ok(vpx_files)
}

pub fn index_vpx_files(vpx_files: &[PathBuf], progress: impl Fn(u64)) -> Vec<(PathBuf, TableInfo)> {
    // TODO tried using rayon here but it's not faster and uses up all cpu
    // use rayon::prelude::*;
    // .par_iter() instead of .iter()
    let mut vpx_files_with_tableinfo: Vec<(PathBuf, TableInfo)> = vpx_files
        .iter()
        .enumerate()
        .flat_map(|(i, vpx_file)| {
            let result = cfb::open(vpx_file).and_then(|mut comp| read_tableinfo(&mut comp));
            let optional = match result {
                Ok(table_info) => Some((vpx_file.clone(), table_info)),
                Err(e) => {
                    // TODO we want to return any failures instead of printing here
                    let warning =
                        format!("Not a valid vpx file {}: {}", vpx_file.display(), e).red();
                    println!("{}", warning);
                    None
                }
            };
            progress((i + 1) as u64);
            optional
        })
        .collect();

    // sort by name
    vpx_files_with_tableinfo.sort_by(|(_, a), (_, b)| {
        // TODO get rid of clone() here
        let a_lower = a
            .table_name
            .clone()
            .unwrap_or("".to_string())
            .to_lowercase();
        let b_lower = b
            .table_name
            .clone()
            .unwrap_or("".to_string())
            .to_lowercase();
        a_lower.cmp(&b_lower)
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
