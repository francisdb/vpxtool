use crate::vpx::{
    biff::{self, BiffRead, BiffReader, BiffWrite},
    color::Color,
};

use super::{font::Font, vertex2d::Vertex2D, GameItem};

#[derive(Debug, PartialEq)]
pub struct Decal {
    pub center: Vertex2D,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub image: String,
    pub surface: String,
    pub name: String,
    pub text: String,
    pub decal_type: u32,
    pub material: String,
    pub color: Color,
    pub sizing_type: u32,
    pub vertical_text: bool,
    pub backglass: bool,

    font: Font,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,
}

impl Decal {
    pub const DECAL_TYPE_TEXT: u32 = 0;
    pub const DECAL_TYPE_IMAGE: u32 = 1;

    pub const SIZING_TYPE_AUTO_SIZE: u32 = 0;
    pub const SIZING_TYPE_AUTO_WIDTH: u32 = 1;
    pub const SIZING_TYPE_MANUAL_SIZE: u32 = 2;
}

impl GameItem for Decal {
    fn name(&self) -> &str {
        &self.name
    }
}

impl BiffRead for Decal {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut width: f32 = 100.0;
        let mut height: f32 = 100.0;
        let mut rotation: f32 = 0.0;
        let mut image = Default::default();
        let mut surface = Default::default();
        let mut name = Default::default();
        let mut text = Default::default();
        let mut decal_type: u32 = Decal::DECAL_TYPE_IMAGE;
        let mut material: String = Default::default();
        let mut color = Color::new_bgr(0x000000);
        let mut sizing_type: u32 = Decal::SIZING_TYPE_MANUAL_SIZE;
        let mut vertical_text: bool = false;
        let mut backglass: bool = false;

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
                "VCEN" => {
                    center = Vertex2D::biff_read(reader);
                }
                "WDTH" => {
                    width = reader.get_f32();
                }
                "HIGH" => {
                    height = reader.get_f32();
                }
                "ROTA" => {
                    rotation = reader.get_f32();
                }
                "IMAG" => {
                    image = reader.get_string();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "TEXT" => {
                    text = reader.get_string();
                }
                "TYPE" => {
                    decal_type = reader.get_u32();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "COLR" => {
                    color = Color::biff_read_bgr(reader);
                }
                "SIZE" => {
                    sizing_type = reader.get_u32();
                }
                "VERT" => {
                    vertical_text = reader.get_bool();
                }
                "BGLS" => {
                    backglass = reader.get_bool();
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
        Decal {
            center,
            width,
            height,
            rotation,
            image,
            surface,
            name,
            text,
            decal_type,
            material,
            color,
            sizing_type,
            vertical_text,
            backglass,
            font,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}

impl BiffWrite for Decal {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VCEN", &self.center);
        writer.write_tagged_f32("WDTH", self.width);
        writer.write_tagged_f32("HIGH", self.height);
        writer.write_tagged_f32("ROTA", self.rotation);
        writer.write_tagged_string("IMAG", &self.image);
        writer.write_tagged_string("SURF", &self.surface);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_string("TEXT", &self.text);
        writer.write_tagged_u32("TYPE", self.decal_type);
        writer.write_tagged_string("MATR", &self.material);
        writer.write_tagged_with("COLR", &self.color, Color::biff_write_bgr);
        writer.write_tagged_u32("SIZE", self.sizing_type);
        writer.write_tagged_bool("VERT", self.vertical_text);
        writer.write_tagged_bool("BGLS", self.backglass);

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
        // values not equal to the defaults
        let decal = Decal {
            center: Vertex2D::new(1.0, 2.0),
            width: 3.0,
            height: 4.0,
            rotation: 5.0,
            image: "image".to_owned(),
            surface: "surface".to_owned(),
            name: "name".to_owned(),
            text: "text".to_owned(),
            decal_type: 1,
            material: "material".to_owned(),
            color: Color::new_bgr(0x010203),
            sizing_type: 2,
            vertical_text: true,
            backglass: true,
            font: Font::default(),
            is_locked: true,
            editor_layer: 3,
            editor_layer_name: Some("editor_layer_name".to_owned()),
            editor_layer_visibility: Some(false),
        };
        let mut writer = BiffWriter::new();
        Decal::biff_write(&decal, &mut writer);
        let decal_read = Decal::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(decal, decal_read);
    }
}
