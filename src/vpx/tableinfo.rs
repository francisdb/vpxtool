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

pub fn read_tableinfo(comp: &mut CompoundFile<File>) -> std::io::Result<TableInfo> {
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

    let result: Result<Vec<_>, _> = paths
        .iter()
        .map(|path| match path.as_str() {
            "/TableInfo/TableName" => read_stream_string(comp, path).map(|s| table_name = s),
            "/TableInfo/AuthorName" => read_stream_string(comp, path).map(|s| author_name = s),
            "/TableInfo/Screenshot" => {
                // seems to be a full image file, eg if there is no jpeg data in the image this is a full png
                // but how do we know the extension?
                screenshot = read_stream_binary(comp, path);
                Ok(())
            }
            "/TableInfo/TableBlurb" => read_stream_string(comp, path).map(|s| table_blurb = s),
            "/TableInfo/TableRules" => read_stream_string(comp, path).map(|s| table_rules = s),
            "/TableInfo/AuthorEmail" => read_stream_string(comp, path).map(|s| author_email = s),
            "/TableInfo/ReleaseDate" => read_stream_string(comp, path).map(|s| release_date = s),
            "/TableInfo/TableSaveRev" => read_stream_string(comp, path).map(|s| table_save_rev = s),
            "/TableInfo/TableVersion" => read_stream_string(comp, path).map(|s| table_version = s),
            "/TableInfo/AuthorWebSite" => {
                read_stream_string(comp, path).map(|s| author_website = s)
            }
            "/TableInfo/TableSaveDate" => {
                read_stream_string(comp, path).map(|s| table_save_date = s)
            }
            "/TableInfo/TableDescription" => {
                read_stream_string(comp, path).map(|s| table_description = s)
            }
            other => {
                let key = other.replace(table_info_path, "").replacen('/', "", 1);
                let str = read_stream_string(comp, path)?;
                properties.insert(key, str);
                Ok(())
            }
        })
        .collect();

    result.map(|_| TableInfo {
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
    })
}

fn read_stream_string(comp: &mut CompoundFile<File>, path: &str) -> Result<String, std::io::Error> {
    let mut stream = comp.open_stream(path).unwrap();
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).unwrap();

    match WStr::from_utf16le(&buffer) {
        Ok(str) => Ok(str.to_utf8()),
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error reading stream as utf16le for path: ".to_owned()
                + path
                + " "
                + " "
                + &e.to_string(),
        )),
    }
}

fn read_stream_binary(comp: &mut CompoundFile<File>, path: &str) -> Vec<u8> {
    let mut stream = comp.open_stream(path).unwrap();
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).unwrap();
    buffer
}
