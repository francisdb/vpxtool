use crate::vpx::{
    biff::{self, BiffRead, BiffReader, BiffWrite},
    color::Color,
    gameitem::font::Font,
};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct TextBox {
    ver1: Vertex2D,         // VER1
    ver2: Vertex2D,         // VER2
    back_color: Color,      // CLRB
    font_color: Color,      // CLRF
    intensity_scale: f32,   // INSC
    text: String,           // TEXT
    is_timer_enabled: bool, // TMON
    timer_interval: u32,    // TMIN
    pub name: String,       // NAME
    align: u32,             // ALGN
    is_transparent: bool,   // TRNS
    is_dmd: Option<bool>,   // IDMD added in 10.2?
    font: Font,             // FONT

    // these are shared between all items
    pub is_locked: bool,                       // LOCK
    pub editor_layer: u32,                     // LAYR
    pub editor_layer_name: Option<String>,     // LANR default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>, // LVIS
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
        let mut is_dmd: Option<bool> = None;

        let mut font = Default::default();

        // these are shared between all items
        let mut is_locked: bool = false;
        let mut editor_layer: u32 = Default::default();
        let mut editor_layer_name: Option<String> = None;
        let mut editor_layer_visibility: Option<bool> = None;

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
                    is_dmd = Some(reader.get_bool());
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
                    editor_layer_name = Some(reader.get_string());
                }
                "LVIS" => {
                    editor_layer_visibility = Some(reader.get_bool());
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
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VER1", &self.ver1);
        writer.write_tagged("VER2", &self.ver2);
        writer.write_tagged_with("CLRB", &self.back_color, Color::biff_write_bgr);
        writer.write_tagged_with("CLRF", &self.font_color, Color::biff_write_bgr);
        writer.write_tagged_f32("INSC", self.intensity_scale);
        writer.write_tagged_string("TEXT", &self.text);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_u32("TMIN", self.timer_interval);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_u32("ALGN", self.align);
        writer.write_tagged_bool("TRNS", self.is_transparent);
        if let Some(is_dmd) = self.is_dmd {
            writer.write_tagged_bool("IDMD", is_dmd);
        }
        writer.write_tagged("FONT", &self.font);
        // shared
        writer.write_tagged_bool("LOCK", self.is_locked);
        writer.write_tagged_u32("LAYR", self.editor_layer);
        if let Some(editor_layer_name) = &self.editor_layer_name {
            writer.write_tagged_string("LANR", editor_layer_name);
        }
        if let Some(editor_layer_visibility) = self.editor_layer_visibility {
            writer.write_tagged_bool("LVIS", editor_layer_visibility);
        }

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
            is_dmd: Some(false),
            font: Font::new(2, 123, 456, "test font".to_string()),
            is_locked: false,
            editor_layer: 1,
            editor_layer_name: Some("test layer".to_string()),
            editor_layer_visibility: Some(true),
        };
        let mut writer = BiffWriter::new();
        TextBox::biff_write(&textbox, &mut writer);
        let textbox_read = TextBox::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(textbox, textbox_read);
    }
}
