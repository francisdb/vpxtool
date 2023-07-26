use crate::vpx::{
    biff::{self, BiffRead, BiffReader},
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
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
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
