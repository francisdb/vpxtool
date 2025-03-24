use vpxtool_shared::indexer::IndexedTable;

pub fn display_table_name(table: &IndexedTable) -> String {
    let file_name = table
        .path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    Some(table.table_info.table_name.to_owned())
        .filter(|s| !s.clone().unwrap_or_default().trim().is_empty())
        .map(|s| {
            match s {
                Some(name) => capitalize_first_letter(&name),
                None => capitalize_first_letter(&file_name),
            }
            // TODO we probably want to show both the file name and the table name
        })
        .unwrap_or(file_name)
}

pub fn display_file_name(table: &IndexedTable) -> String {
    table
        .path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
fn capitalize_first_letter(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
}
