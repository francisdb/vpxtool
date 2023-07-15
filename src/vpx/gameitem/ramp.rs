use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::dragpoint::DragPoint;

#[derive(Debug, PartialEq)]
pub struct Ramp {
    pub name: String,
    points: Vec<DragPoint>,
}

impl BiffRead for Ramp {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut name = Default::default();
        let mut points: Vec<DragPoint> = Default::default();

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
                "DPNT" => {
                    let point = DragPoint::biff_read(reader);
                    points.push(point);
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
        Ramp { name, points }
    }
}
