use crate::vpx::biff::{self, BiffRead, BiffReader};

#[derive(Debug, PartialEq)]
pub struct Point {
    x: f32,
    y: f32,
    z: f32,
    smooth: bool,
    auto_tex: bool,
    tex_coord: f32,
}

impl BiffRead for Point {
    fn biff_read(reader: &mut BiffReader<'_>) -> Point {
        let mut sub_data = reader.child_reader();
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut tex_coord = 0.0;
        let mut smooth = false;
        let mut auto_tex = false;
        let point_skipped = vec!["LOCK", "LAYR", "LANR", "LVIS", "SLNG"];
        loop {
            sub_data.next(biff::WARN);
            if sub_data.is_eof() {
                break;
            }
            let tag = sub_data.tag();
            let tag_str = tag.as_str();
            match tag_str {
                "VCEN" => {
                    x = sub_data.get_float();
                    y = sub_data.get_float();
                }
                "POSZ" => {
                    z = sub_data.get_float();
                }
                "SMTH" => {
                    smooth = sub_data.get_bool();
                }
                "ATEX" => {
                    auto_tex = sub_data.get_bool();
                }
                "TEXC" => {
                    tex_coord = sub_data.get_float();
                }
                other => {
                    if point_skipped.contains(&other) {
                        sub_data.skip_tag();
                    } else {
                        println!("Unknown tag: {}", other);
                        sub_data.skip_tag();
                    }
                }
            }
        }
        let pos = sub_data.pos();
        reader.skip_end_tag(pos);
        Point {
            x,
            y,
            z,
            smooth,
            auto_tex,
            tex_coord,
        }
    }
}
