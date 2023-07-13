use super::biff::{self, BiffReader, BiffWriter};

// TODO comment here a vpx file that contains font data

#[derive(PartialEq, Debug)]
pub struct GameItem {
    pub name: String,
}

// Item types:
// 0: Wall
// 1: Flipper
// 2: Timer
// 3: Plunger
// 4: Text box
// 5: Bumper
// 6: Trigger
// 7: Light
// 8: Kicker
// 9: Decal
// 10: Gate
// 11: Spinner
// 12: Ramp
// 13: Table
// 14: Light Center
// 15: Drag Point
// 16: Collection
// 17: Reel
// 18: Light sequencer
// 19: Primitive
// 20: Flasher
// 21: Rubber
// 22: Hit Target

pub fn read(input: &[u8]) -> GameItem {
    let mut reader = BiffReader::new(input);
    let mut name: String = "".to_string();
    let mut points: Vec<Point> = vec![];
    let item_type = reader.get_32_no_remaining_update();
    println!("Item type: {}", item_type);
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
                // for a wall
                let point = load_point(&mut reader);
                points.push(point);
            }
            "FONT" => {
                let font = load_font(&mut reader);
                dbg!(font);
            }
            other => {
                //println!("Unknown tag: {}", other);
                reader.skip_tag();
                //let data = reader.get_record_data(false);
            }
        }
    }
    GameItem { name }
}

// def load_point(item_data):
//     sub_data = item_data.child_reader()
//     x = y = z = tex_coord = 0
//     smooth = False
//     point_skipped = ('LOCK', 'LAYR', 'LANR', 'LVIS', 'SLNG')
//     while not sub_data.is_eof():
//         sub_data.next()
//         if sub_data.tag == 'VCEN':
//             x = sub_data.get_float()
//             y = sub_data.get_float()
//         elif sub_data.tag == 'POSZ':
//             z = sub_data.get_float()
//         elif sub_data.tag == 'SMTH':
//             smooth = sub_data.get_bool()
//         elif sub_data.tag == 'ATEX':
//             auto_tex = sub_data.get_bool()
//         elif sub_data.tag == 'TEXC':
//             tex_coord = sub_data.get_float()
//         elif sub_data.tag in point_skipped:
//             sub_data.skip_tag()
//     item_data.skip(sub_data.pos)
//     return [x, y, z, smooth, auto_tex, tex_coord]

pub struct Point {
    x: f32,
    y: f32,
    z: f32,
    smooth: bool,
    auto_tex: bool,
    tex_coord: f32,
}

fn load_point(item_data: &mut BiffReader) -> Point {
    let mut sub_data = item_data.child_reader();
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
    item_data.skip_end_tag(pos);
    Point {
        x,
        y,
        z,
        smooth,
        auto_tex,
        tex_coord,
    }
}

#[derive(PartialEq, Debug)]
pub struct Font {
    // Font style flags
    //
    // #define TTF_STYLE_NORMAL        0x00
    // #define TTF_STYLE_BOLD          0x01
    // #define TTF_STYLE_ITALIC        0x02
    // #define TTF_STYLE_UNDERLINE     0x04
    // #define TTF_STYLE_STRIKETHROUGH 0x08
    style: u8,
    weight: u16,
    size: u32,
    name: String,
}

fn load_font(reader: &mut BiffReader) -> Font {
    let _header = reader.get_data(3); // always? 0x01, 0x0, 0x0

    let style = reader.get_u8_no_remaining_update();
    let weight = reader.get_u16_no_remaining_update();
    let size = reader.get_u32_no_remaining_update();
    let name_len = reader.get_u8_no_remaining_update();
    let name = reader.get_str_no_remaining_update(name_len as usize);
    Font {
        style,
        weight,
        size,
        name,
    }
}

fn write_font(font: &Font) -> Vec<u8> {
    let mut writer = BiffWriter::new();
    writer.write_data(&[0x01, 0x00, 0x00]);
    writer.write_u8(font.style);
    writer.write_u16(font.weight);
    writer.write_u32(font.size);
    writer.write_short_string(&font.name);
    writer.get_data().to_owned()
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn write_read_font() {
        let font = Font {
            style: 0,
            weight: 0,
            size: 0,
            name: "Arial".to_string(),
        };
        let data = write_font(&font);
        let mut reader = BiffReader::new(&data);
        let font2 = load_font(&mut reader);
        assert_eq!(font, font2);
    }
}
