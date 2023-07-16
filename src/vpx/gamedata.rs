#![allow(dead_code)]

use super::biff::{self, BiffReader, BiffWriter};

#[derive(Debug, PartialEq)]
pub enum Record {
    PlayfieldLeft(u32),
    PlayfieldTopX(u32),
    PlayfieldRight(u32),
    PlayfieldBottom(u32),
    Code { script: String },
    Name(String),
    MaterialsSize(u32),
    ImagesSize(u32),
    SoundsSize(u32),
    FontsSize(u32),
    CollectionsSize(u32),
    GameItemsSize(u32),
    Unknown { name: String, data: Vec<u8> },
}

pub fn write_all_gamedata_records(game_data_vec: &[Record]) -> Vec<u8> {
    let mut writer = BiffWriter::new();
    // iterate over the records and write them
    for rec in game_data_vec {
        match rec {
            Record::PlayfieldLeft(n) => {
                writer.write_tagged_u32("LEFT", *n);
            }
            Record::PlayfieldTopX(n) => {
                writer.write_tagged_u32("TOPX", *n);
            }
            Record::PlayfieldRight(n) => {
                writer.write_tagged_u32("RGHT", *n);
            }
            Record::PlayfieldBottom(n) => {
                writer.write_tagged_u32("BOTM", *n);
            }
            Record::Name(s) => {
                writer.write_tagged_wide_string("NAME", s);
            }
            Record::Code { script } => {
                writer.new_tag("CODE");
                writer.write_string(script);
                // code records do not indicate their size
                writer.end_tag_no_size();
            }
            Record::MaterialsSize(n) => {
                writer.write_tagged_u32("MASI", *n);
            }
            Record::GameItemsSize(n) => {
                writer.write_tagged_u32("SEDT", *n);
            }
            Record::SoundsSize(n) => {
                writer.write_tagged_u32("SSND", *n);
            }
            Record::ImagesSize(n) => {
                writer.write_tagged_u32("SIMG", *n);
            }
            Record::FontsSize(n) => {
                writer.write_tagged_u32("SFNT", *n);
            }
            Record::CollectionsSize(n) => {
                writer.write_tagged_u32("SCOL", *n);
            }
            Record::Unknown { name, data } => {
                writer.write_tagged_data(name, data);
            }
        }
    }
    // TODO how do we get rid of this extra copy?
    writer.close(true);
    writer.get_data().to_vec()
}

pub fn read_all_gamedata_records(input: &[u8]) -> Vec<Record> {
    let mut reader = BiffReader::new(input);
    let mut records = Vec::new();
    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();
        let rec = read_record(tag_str, &mut reader);
        records.push(rec);
    }
    records
}

fn read_record(tag_str: &str, reader: &mut BiffReader<'_>) -> Record {
    let rec = match tag_str {
        "LEFT" => {
            let n = reader.get_u32();
            Record::PlayfieldLeft(n)
        }
        "TOPX" => {
            let n = reader.get_u32();
            Record::PlayfieldTopX(n)
        }
        "RGHT" => {
            let n = reader.get_u32();
            Record::PlayfieldRight(n)
        }
        "BOTM" => {
            let n = reader.get_u32();
            Record::PlayfieldBottom(n)
        }
        "NAME" => {
            let s = reader.get_wide_string();
            Record::Name(s)
        }
        "CODE" => {
            let len = reader.get_u32_no_remaining_update();
            let script = reader.get_str_no_remaining_update(len as usize);
            Record::Code { script }
        }
        "MASI" => {
            let n = reader.get_u32();
            Record::MaterialsSize(n)
        }
        "SEDT" => {
            let n = reader.get_u32();
            Record::GameItemsSize(n)
        }
        "SSND" => {
            let n = reader.get_u32();
            Record::SoundsSize(n)
        }
        "SIMG" => {
            let n = reader.get_u32();
            Record::ImagesSize(n)
        }
        "SFNT" => {
            let n = reader.get_u32();
            Record::FontsSize(n)
        }
        "SCOL" => {
            let n = reader.get_u32();
            Record::CollectionsSize(n)
        }
        other => {
            //dbg!(other);
            let data = reader.get_record_data(false);
            let tag_str = other.to_string();
            Record::Unknown {
                name: tag_str,
                data: data.to_vec(),
            }
        }
    };
    rec
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn read_write_empty() {
        let game_data: &[Record] = &[];
        let bytes = write_all_gamedata_records(game_data);
        let read_game_data = read_all_gamedata_records(&bytes);

        assert_eq!(game_data, read_game_data);
    }

    #[test]
    fn read_write() {
        let game_data: &[Record] = &[
            Record::Code {
                script: String::from("test"),
            },
            Record::Name(String::from("test2")),
            Record::Unknown {
                name: String::from("TST"),
                data: vec![4, 3, 2],
            },
        ];
        let bytes = write_all_gamedata_records(game_data);
        let read_game_data = read_all_gamedata_records(&bytes);

        assert_eq!(game_data, read_game_data);
    }
}
