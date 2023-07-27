use crate::vpx::{
    biff::{self, BiffRead, BiffReader, BiffWrite},
    color::Color,
    gameitem::font::Font,
};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct TextBox {
    ver1: Vertex2D,
    ver2: Vertex2D,
    back_color: Color,
    font_color: Color,
    intensity_scale: f32,
    text: String,
    is_timer_enabled: bool,
    timer_interval: u32,
    pub name: String,
    align: u32,
    is_transparent: bool,
    is_dmd: bool,
    font: Font,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl BiffRead for TextBox {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut ver1 = Vertex2D::default();
        let mut ver2 = Vertex2D::default();
        let mut back_color = Color::new_bgr(0x000000f);
        let mut font_color = Color::new_bgr(0xfffffff);
        let mut intensity_scale: f32 = 1.0;
        let mut text: String = Default::default();
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = Default::default();
        let mut name = Default::default();
        let mut align: u32 = Default::default();
        let mut is_transparent: bool = false;
        let mut is_dmd: bool = false;

        let mut font = Default::default();

        // these are shared between all items
        let mut is_locked: bool = false;
        let mut editor_layer: u32 = Default::default();
        let mut editor_layer_name: String = Default::default();
        let mut editor_layer_visibility: bool = true;

        loop {
            reader.next(biff::WARN);
            if reader.is_eof() {
                break;
            }
            let tag = reader.tag();
            let tag_str = tag.as_str();
            match tag_str {
                "VER1" => {
                    ver1 = Vertex2D::biff_read(reader);
                }
                "VER2" => {
                    ver2 = Vertex2D::biff_read(reader);
                }
                "CLRB" => {
                    back_color = Color::biff_read_bgr(reader);
                }
                "CLRF" => {
                    font_color = Color::biff_read_bgr(reader);
                }
                "INSC" => {
                    intensity_scale = reader.get_f32();
                }
                "TEXT" => {
                    text = reader.get_string();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "ALGN" => {
                    align = reader.get_u32();
                }
                "TRNS" => {
                    is_transparent = reader.get_bool();
                }
                "IDMD" => {
                    is_dmd = reader.get_bool();
                }

                "FONT" => {
                    font = Font::biff_read(reader);
                }
                // shared
                "LOCK" => {
                    is_locked = reader.get_bool();
                }
                "LAYR" => {
                    editor_layer = reader.get_u32();
                }
                "LANR" => {
                    editor_layer_name = reader.get_string();
                }
                "LVIS" => {
                    editor_layer_visibility = reader.get_bool();
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
        Self {
            ver1,
            ver2,
            back_color,
            font_color,
            intensity_scale,
            text,
            is_timer_enabled,
            timer_interval,
            name,
            align,
            is_transparent,
            is_dmd,
            font,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}

impl BiffWrite for TextBox {
    fn biff_write(item: &Self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VER1", &item.ver1);
        writer.write_tagged("VER2", &item.ver2);
        writer.write_tagged_with("CLRB", &item.back_color, |c, writer| {
            Color::biff_write_bgr(c, writer)
        });
        writer.write_tagged_with("CLRF", &item.font_color, |c, writer| {
            Color::biff_write_bgr(c, writer)
        });
        writer.write_tagged_f32("INSC", item.intensity_scale);
        writer.write_tagged_string("TEXT", &item.text);
        writer.write_tagged_bool("TMON", item.is_timer_enabled);
        writer.write_tagged_u32("TMIN", item.timer_interval);
        writer.write_tagged_wide_string("NAME", &item.name);
        writer.write_tagged_u32("ALGN", item.align);
        writer.write_tagged_bool("TRNS", item.is_transparent);
        writer.write_tagged_bool("IDMD", item.is_dmd);
        // shared
        writer.write_tagged_bool("LOCK", item.is_locked);
        writer.write_tagged_u32("LAYR", item.editor_layer);
        writer.write_tagged_string("LANR", &item.editor_layer_name);
        writer.write_tagged_bool("LVIS", item.editor_layer_visibility);
        writer.close(true);
    }
}

#[cfg(test)]
mod tests {
    use crate::vpx::biff::BiffWriter;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_write_read() {
        let textbox = TextBox {
            ver1: Vertex2D::new(1.0, 2.0),
            ver2: Vertex2D::new(3.0, 4.0),
            back_color: Color::new_bgr(0x1234567),
            font_color: Color::new_bgr(0xfedcba9),
            intensity_scale: 1.0,
            text: "test text".to_string(),
            is_timer_enabled: true,
            timer_interval: 3,
            name: "test timer".to_string(),
            align: 0,
            is_transparent: false,
            is_dmd: false,
            font: Font::default(),
            is_locked: false,
            editor_layer: 1,
            editor_layer_name: "test layer".to_string(),
            editor_layer_visibility: true,
        };
        let mut writer = BiffWriter::new();
        TextBox::biff_write(&textbox, &mut writer);
        let textbox_read = TextBox::biff_read(&mut BiffReader::new(&writer.get_data()));
        assert_eq!(textbox, textbox_read);
    }
}
