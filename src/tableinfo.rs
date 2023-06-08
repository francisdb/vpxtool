use cfb::CompoundFile;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use utf16string::WStr;

#[derive(Debug)]
pub struct TableInfo {
    pub table_name: String,
    pub author_name: String,
    pub screenshot: Vec<u8>,
    pub table_blurb: String,
    pub table_rules: String,
    pub author_email: String,
    pub release_date: String,
    pub table_save_rev: String,
    pub table_version: String,
    pub author_website: String,
    pub table_save_date: String,
    pub table_description: String,
    pub properties: HashMap<String, String>,
}

pub fn read_tableinfo(comp: &mut CompoundFile<File>) -> TableInfo {
    let table_info_path = "/TableInfo";
    // println!("Reading table info at {}", table_info_path);
    let mut table_name: String = "".to_string();
    let mut author_name: String = "".to_string();
    let mut screenshot: Vec<u8> = vec![];
    let mut table_blurb: String = "".to_string();
    let mut table_rules: String = "".to_string();
    let mut author_email: String = "".to_string();
    let mut release_date: String = "".to_string();
    let mut table_save_rev: String = "".to_string();
    let mut table_version: String = "".to_string();
    let mut author_website: String = "".to_string();
    let mut table_save_date: String = "".to_string();
    let mut table_description: String = "".to_string();
    let mut properties: HashMap<String, String> = HashMap::new();

    let entries = comp.read_storage(table_info_path).unwrap();
    // read all the entries in the entrues
    let paths: Vec<String> = entries
        .filter(|entry| entry.is_stream())
        .map(|entry| entry.path().to_str().unwrap().to_owned())
        .collect();

    paths.iter().for_each(|path| match path.as_str() {
        "/TableInfo/TableName" => table_name = read_stream_string(comp, path),
        "/TableInfo/AuthorName" => author_name = read_stream_string(comp, path),
        "/TableInfo/Screenshot" => {
            // seems to be a full image file, eg if there is no jpeg data in the image this is a full png
            // but how do we know the extension?
            screenshot = read_stream_binary(comp, path)
        }
        "/TableInfo/TableBlurb" => table_blurb = read_stream_string(comp, path),
        "/TableInfo/TableRules" => table_rules = read_stream_string(comp, path),
        "/TableInfo/AuthorEmail" => author_email = read_stream_string(comp, path),
        "/TableInfo/ReleaseDate" => release_date = read_stream_string(comp, path),
        "/TableInfo/TableSaveRev" => table_save_rev = read_stream_string(comp, path),
        "/TableInfo/TableVersion" => table_version = read_stream_string(comp, path),
        "/TableInfo/AuthorWebSite" => author_website = read_stream_string(comp, path),
        "/TableInfo/TableSaveDate" => table_save_date = read_stream_string(comp, path),
        "/TableInfo/TableDescription" => table_description = read_stream_string(comp, path),
        other => {
            let key = other.replace(table_info_path, "").replacen('/', "", 1);
            properties.insert(key, read_stream_string(comp, path));
        }
    });

    TableInfo {
        table_name,
        author_name,
        screenshot,
        table_blurb,
        table_rules,
        author_email,
        release_date,
        table_save_rev,
        table_version,
        author_website,
        table_save_date,
        table_description,
        properties,
    }
}

fn read_stream_string(comp: &mut CompoundFile<File>, path: &str) -> String {
    let mut stream = comp.open_stream(path).unwrap();
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).unwrap();

    WStr::from_utf16le(&buffer).unwrap().to_utf8()
}

fn read_stream_binary(comp: &mut CompoundFile<File>, path: &str) -> Vec<u8> {
    let mut stream = comp.open_stream(path).unwrap();
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).unwrap();
    buffer
}
