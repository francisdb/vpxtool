use cfb::CompoundFile;
use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::path::{Path, MAIN_SEPARATOR_STR};
use utf16string::{LittleEndian, WStr, WString};

// >    "/TableInfo/AuthorName",
// >    "/TableInfo/Screenshot",
// >    "/TableInfo/TableBlurb",
// >    "/TableInfo/TableRules",
// >    "/TableInfo/AuthorEmail",
// >    "/TableInfo/ReleaseDate",

#[derive(PartialEq, Debug)]
pub struct TableInfo {
    pub table_name: Option<String>,
    pub author_name: Option<String>,
    pub screenshot: Option<Vec<u8>>,
    pub table_blurb: Option<String>,
    pub table_rules: Option<String>,
    pub author_email: Option<String>,
    pub release_date: Option<String>,
    pub table_save_rev: Option<String>,
    pub table_version: Option<String>,
    pub author_website: Option<String>,
    pub table_save_date: Option<String>,
    pub table_description: Option<String>,
    // the keys (and ordering) for these are defined in "GameStg/CustomInfoTags"
    pub properties: HashMap<String, String>,
}
impl TableInfo {
    pub(crate) fn new() -> TableInfo {
        // current data as ISO string
        //let now: String = chrono::Local::now().to_rfc3339();
        TableInfo {
            table_name: None,
            author_name: None,
            screenshot: None,
            table_blurb: None,
            table_rules: None,
            author_email: None,
            release_date: None,
            table_save_rev: None, // added in ?
            table_version: None,
            author_website: None,
            table_save_date: None, // added in ?
            table_description: None,
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

    table_info
        .table_name
        .as_ref()
        .map(|table_name| {
            write_stream_string(
                comp,
                table_info_path.join("TableName").as_path(),
                table_name,
            )
        })
        .unwrap_or(Ok(()))?;
    table_info
        .author_name
        .as_ref()
        .map(|author_name| {
            write_stream_string(
                comp,
                table_info_path.join("AuthorName").as_path(),
                author_name,
            )
        })
        .unwrap_or(Ok(()))?;
    table_info
        .screenshot
        .as_ref()
        .map(|screenshot| {
            write_stream_binary(
                comp,
                table_info_path.join("Screenshot").as_path(),
                screenshot,
            )
        })
        .unwrap_or(Ok(()))?;
    table_info
        .table_blurb
        .as_ref()
        .map(|table_blurb| {
            write_stream_string(
                comp,
                table_info_path.join("TableBlurb").as_path(),
                table_blurb,
            )
        })
        .unwrap_or(Ok(()))?;
    table_info
        .table_rules
        .as_ref()
        .map(|table_rules| {
            write_stream_string(
                comp,
                table_info_path.join("TableRules").as_path(),
                table_rules,
            )
        })
        .unwrap_or(Ok(()))?;
    table_info
        .author_email
        .as_ref()
        .map(|author_email| {
            write_stream_string(
                comp,
                table_info_path.join("AuthorEmail").as_path(),
                author_email,
            )
        })
        .unwrap_or(Ok(()))?;

    table_info
        .release_date
        .as_ref()
        .map(|release_date| {
            write_stream_string(
                comp,
                table_info_path.join("ReleaseDate").as_path(),
                release_date,
            )
        })
        .unwrap_or(Ok(()))?;
    table_info
        .table_save_rev
        .as_ref()
        .map(|table_save_rev| {
            write_stream_string(
                comp,
                table_info_path.join("TableSaveRev").as_path(),
                table_save_rev,
            )
        })
        .unwrap_or(Ok(()))?;
    table_info
        .table_version
        .as_ref()
        .map(|table_version| {
            write_stream_string(
                comp,
                table_info_path.join("TableVersion").as_path(),
                table_version,
            )
        })
        .unwrap_or(Ok(()))?;
    table_info
        .author_website
        .as_ref()
        .map(|author_website| {
            write_stream_string(
                comp,
                table_info_path.join("AuthorWebSite").as_path(),
                author_website,
            )
        })
        .unwrap_or(Ok(()))?;
    table_info
        .table_save_date
        .as_ref()
        .map(|table_save_date| {
            write_stream_string(
                comp,
                table_info_path.join("TableSaveDate").as_path(),
                table_save_date,
            )
        })
        .unwrap_or(Ok(()))?;
    table_info
        .table_description
        .as_ref()
        .map(|table_description| {
            write_stream_string(
                comp,
                table_info_path.join("TableDescription").as_path(),
                table_description,
            )
        })
        .unwrap_or(Ok(()))?;

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
    let mut table_info = TableInfo::new();

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
                "TableName" => {
                    read_stream_string(comp, path).map(|s| table_info.table_name = Some(s))
                }
                "AuthorName" => {
                    read_stream_string(comp, path).map(|s| table_info.author_name = Some(s))
                }
                "Screenshot" => {
                    // seems to be a full image file, eg if there is no jpeg data in the image this is a full png
                    // but how do we know the extension?
                    read_stream_binary(comp, path).map(|v| table_info.screenshot = Some(v))
                }
                "TableBlurb" => {
                    read_stream_string(comp, path).map(|s| table_info.table_blurb = Some(s))
                }
                "TableRules" => {
                    read_stream_string(comp, path).map(|s| table_info.table_rules = Some(s))
                }
                "AuthorEmail" => {
                    read_stream_string(comp, path).map(|s| table_info.author_email = Some(s))
                }
                "ReleaseDate" => {
                    read_stream_string(comp, path).map(|s| table_info.release_date = Some(s))
                }
                "TableSaveRev" => {
                    read_stream_string(comp, path).map(|s| table_info.table_save_rev = Some(s))
                }
                "TableVersion" => {
                    read_stream_string(comp, path).map(|s| table_info.table_version = Some(s))
                }
                "AuthorWebSite" => {
                    read_stream_string(comp, path).map(|s| table_info.author_website = Some(s))
                }
                "TableSaveDate" => {
                    read_stream_string(comp, path).map(|s| table_info.table_save_date = Some(s))
                }
                "TableDescription" => {
                    read_stream_string(comp, path).map(|s| table_info.table_description = Some(s))
                }
                other => {
                    let str = read_stream_string(comp, path)?;
                    table_info.properties.insert(other.to_string(), str);
                    Ok(())
                }
            }
        })
        .collect();

    result.map(|_| table_info)
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
            table_name: Some("test_table_name".to_string()),
            author_name: Some("test_author_name".to_string()),
            screenshot: Some(vec![1, 2, 3]),
            table_blurb: Some("test_table_blurb".to_string()),
            table_rules: Some("test_table_rules".to_string()),
            author_email: Some("test_author_email".to_string()),
            release_date: None,
            table_save_rev: Some("test_table_save_rev".to_string()),
            table_version: Some("test_table_version".to_string()),
            author_website: Some("test_author_website".to_string()),
            table_save_date: Some("test_table_save_date".to_string()),
            table_description: Some("test_table_description".to_string()),
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
