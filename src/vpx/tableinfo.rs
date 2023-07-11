use cfb::CompoundFile;
use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::path::{Path, MAIN_SEPARATOR_STR};
use utf16string::{LittleEndian, WStr, WString};

#[derive(PartialEq, Debug)]
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
impl TableInfo {
    pub(crate) fn new() -> TableInfo {
        // current data as ISO string
        //let now: String = chrono::Local::now().to_rfc3339();
        TableInfo {
            table_name: "".to_string(),
            author_name: "".to_string(),
            screenshot: vec![],
            table_blurb: "".to_string(),
            table_rules: "".to_string(),
            author_email: "".to_string(),
            release_date: "".to_string(),
            table_save_rev: "".to_string(),
            table_version: "".to_string(),
            author_website: "".to_string(),
            table_save_date: "".to_string(),
            table_description: "".to_string(),
            properties: HashMap::new(),
        }
    }
}

pub fn write_tableinfo<F: Read + Write + Seek>(
    comp: &mut CompoundFile<F>,
    table_info: &TableInfo,
) -> std::io::Result<()> {
    let table_info_path = Path::new(MAIN_SEPARATOR_STR).join("TableInfo");
    comp.create_storage(&table_info_path)?;
    write_stream_string(
        comp,
        table_info_path.join("TableName").as_path(),
        &table_info.table_name,
    )?;
    write_stream_string(
        comp,
        table_info_path.join("AuthorName").as_path(),
        &table_info.author_name,
    )?;
    write_stream_binary(
        comp,
        table_info_path.join("Screenshot").as_path(),
        &table_info.screenshot,
    )?;
    write_stream_string(
        comp,
        table_info_path.join("TableBlurb").as_path(),
        &table_info.table_blurb,
    )?;
    write_stream_string(
        comp,
        table_info_path.clone().join("TableRules").as_path(),
        &table_info.table_rules,
    )?;
    write_stream_string(
        comp,
        table_info_path.join("AuthorEmail").as_path(),
        &table_info.author_email,
    )?;
    write_stream_string(
        comp,
        table_info_path.join("ReleaseDate").as_path(),
        &table_info.release_date,
    )?;
    write_stream_string(
        comp,
        table_info_path.join("TableSaveRev").as_path(),
        &table_info.table_save_rev,
    )?;
    write_stream_string(
        comp,
        table_info_path.join("TableVersion").as_path(),
        &table_info.table_version,
    )?;
    write_stream_string(
        comp,
        table_info_path.join("AuthorWebSite").as_path(),
        &table_info.author_website,
    )?;
    write_stream_string(
        comp,
        table_info_path.join("TableSaveDate").as_path(),
        &table_info.table_save_date,
    )?;
    write_stream_string(
        comp,
        table_info_path.join("TableDescription").as_path(),
        &table_info.table_description,
    )?;

    // write properties
    for (key, value) in &table_info.properties {
        write_stream_string(comp, table_info_path.join(key).as_path(), value)?;
    }

    Ok(())
}

pub fn read_tableinfo<F: Read + Write + Seek>(
    comp: &mut CompoundFile<F>,
) -> std::io::Result<TableInfo> {
    // create path to table info using path separator
    let table_info_path = Path::new(MAIN_SEPARATOR_STR).join("TableInfo");
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
    let paths: Vec<_> = entries
        .filter(|entry| entry.is_stream())
        .map(|entry| entry.path().to_owned())
        .collect();

    // "/TableInfo/TableName"
    // "/TableInfo/TableDescription"

    let result: Result<Vec<_>, _> = paths
        .iter()
        .map(|path| {
            let file_name = path
                .file_name()
                .map(|s| s.to_str().unwrap_or("[not unicode]"))
                .unwrap_or("..");
            match file_name {
                "TableName" => read_stream_string(comp, path).map(|s| table_name = s),
                "AuthorName" => read_stream_string(comp, path).map(|s| author_name = s),
                "Screenshot" => {
                    // seems to be a full image file, eg if there is no jpeg data in the image this is a full png
                    // but how do we know the extension?
                    read_stream_binary(comp, path).map(|v| screenshot = v)
                }
                "TableBlurb" => read_stream_string(comp, path).map(|s| table_blurb = s),
                "TableRules" => read_stream_string(comp, path).map(|s| table_rules = s),
                "AuthorEmail" => read_stream_string(comp, path).map(|s| author_email = s),
                "ReleaseDate" => read_stream_string(comp, path).map(|s| release_date = s),
                "TableSaveRev" => read_stream_string(comp, path).map(|s| table_save_rev = s),
                "TableVersion" => read_stream_string(comp, path).map(|s| table_version = s),
                "AuthorWebSite" => read_stream_string(comp, path).map(|s| author_website = s),
                "TableSaveDate" => read_stream_string(comp, path).map(|s| table_save_date = s),
                "TableDescription" => read_stream_string(comp, path).map(|s| table_description = s),
                other => {
                    let str = read_stream_string(comp, path)?;
                    properties.insert(other.to_string(), str);
                    Ok(())
                }
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

fn read_stream_string<F: Read + Write + Seek>(
    comp: &mut CompoundFile<F>,
    path: &Path,
) -> Result<String, std::io::Error> {
    let mut stream = comp.open_stream(path).unwrap();
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).unwrap();

    match WStr::from_utf16le(&buffer) {
        Ok(str) => Ok(str.to_utf8()),
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error reading stream as utf16le for path: ".to_owned()
                + path.to_str().unwrap_or("[not unicode]")
                + " "
                + &e.to_string(),
        )),
    }
}

fn write_stream_string<F: Read + Write + Seek>(
    comp: &mut CompoundFile<F>,
    path: &Path,
    str: &String,
) -> std::io::Result<()> {
    let mut stream = comp.create_stream(path)?;
    let wide: WString<LittleEndian> = WString::from(str);
    stream.write_all(wide.as_bytes())
}

fn read_stream_binary<F: Read + Seek>(
    comp: &mut CompoundFile<F>,
    path: &Path,
) -> std::io::Result<Vec<u8>> {
    let mut stream = comp.open_stream(path)?;
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn write_stream_binary<F: Read + Write + Seek>(
    comp: &mut CompoundFile<F>,
    path: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
    let mut stream = comp.create_stream(path)?;
    stream.write_all(bytes)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_write_read() {
        let buff = Cursor::new(vec![0; 15]);
        let mut comp = CompoundFile::create(buff).unwrap();
        let table_info = TableInfo {
            table_name: "test".to_string(),
            author_name: "test".to_string(),
            screenshot: vec![1, 2, 3],
            table_blurb: "test".to_string(),
            table_rules: "test".to_string(),
            author_email: "test".to_string(),
            release_date: "test".to_string(),
            table_save_rev: "test".to_string(),
            table_version: "test".to_string(),
            author_website: "test".to_string(),
            table_save_date: "test".to_string(),
            table_description: "test".to_string(),
            properties: HashMap::from([
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "value2".to_string()),
            ]),
        };
        write_tableinfo(&mut comp, &table_info).unwrap();
        let table_info_read = read_tableinfo(&mut comp).unwrap();

        assert_eq!(table_info_read, table_info);
    }

    // #[test]
    // fn test_bad_add() {
    //     // This assert would fire and test will fail.
    //     // Please note, that private functions can be tested too!
    //     assert_eq!(bad_add(1, 2), 3);
    // }
}
