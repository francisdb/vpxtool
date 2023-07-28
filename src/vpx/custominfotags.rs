use super::biff::{self, BiffReader, BiffWriter};

pub type CustomInfoTags = Vec<String>;

pub fn read_custominfotags(tags_data: &[u8]) -> CustomInfoTags {
    let mut reader = BiffReader::new(tags_data);
    let mut tags = CustomInfoTags::new();

    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();

        let reader: &mut BiffReader<'_> = &mut reader;

        match tag_str {
            "CUST" => {
                let tag = reader.get_string();
                tags.push(tag);
            }
            other => {
                let data = reader.get_record_data(false);
                println!("unhandled tag {} {} bytes", other, data.len());
            }
        }
    }
    tags
}

pub fn write_custominfotags(tags: &CustomInfoTags) -> Vec<u8> {
    let mut writer = BiffWriter::new();
    for tag in tags {
        writer.write_tagged_string("CUST", tag);
    }
    writer.close(true);
    writer.get_data().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_empty() {
        let game_data = CustomInfoTags::default();
        let bytes = write_custominfotags(&game_data);
        let read_game_data = read_custominfotags(&bytes);

        assert_eq!(game_data, read_game_data);
    }
}
