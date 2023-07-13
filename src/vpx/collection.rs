use super::biff::{self, BiffReader, BiffWriter};

// TODO comment here a vpx file that contains font data

#[derive(PartialEq, Debug)]
pub struct Collection {
    pub name: String,
    pub items: Vec<String>,
    pub fire_events: bool,
    pub stop_single_events: bool,
    pub group_elements: bool,
}

pub fn read(input: &[u8]) -> Collection {
    let mut reader = BiffReader::new(input);
    let mut name: String = "".to_string();
    let mut items: Vec<String> = vec![];
    let mut fire_events: bool = false;
    let mut stop_single_events: bool = false;
    let mut group_elements: bool = false;
    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();
        match tag_str {
            "NAME" => {
                name = reader.get_wide_string();
            }
            "ITEM" => {
                let item = reader.get_wide_string();
                items.push(item);
            }
            "EVNT" => {
                fire_events = reader.get_bool();
            }
            "SSNG" => {
                stop_single_events = reader.get_bool();
            }
            "GREL" => {
                group_elements = reader.get_bool();
            }
            other => {
                println!("Unknown tag: {}", other);
                reader.skip_tag();
            }
        }
    }
    Collection {
        name,
        items,
        fire_events,
        stop_single_events,
        group_elements,
    }
}

pub fn write(collection: &Collection) -> Vec<u8> {
    let mut writer = BiffWriter::new();
    writer.write_tagged_wide_string("NAME", &collection.name);
    for item in &collection.items {
        writer.write_tagged_wide_string("ITEM", item);
    }
    writer.write_tagged_bool("EVNT", collection.fire_events);
    writer.write_tagged_bool("SSNG", collection.stop_single_events);
    writer.write_tagged_bool("GREL", collection.group_elements);
    writer.close(true);
    writer.get_data().to_owned()
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn write_read() {
        use pretty_assertions::assert_eq;

        let collection = Collection {
            name: "Test Collection".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            fire_events: true,
            stop_single_events: true,
            group_elements: true,
        };
        let data = write(&collection);
        let collection2 = read(&data);
        assert_eq!(collection, collection2);
    }
}
