use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::font::Font;

#[derive(Debug, PartialEq)]
pub struct Decal {
    pub name: String,
    font: Font,
}

impl BiffRead for Decal {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut name = Default::default();
        let mut font = Default::default();

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
                "FONT" => {
                    font = Font::biff_read(reader);
                }
                _ => {
                    println!(
                        "Unknown tag {} for {}",
                        tag_str,
                        std::any::type_name::<Self>()
                    );
                    reader.skip_tag();
                }
            }
        }
        Decal { name, font }
    }
}
