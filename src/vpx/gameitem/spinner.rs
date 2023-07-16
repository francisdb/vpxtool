use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct Spinner {
    center: Vertex2D,
    rotation: f32,
    is_timer_enabled: bool,
    timer_interval: u32,
    height: f32,
    length: f32,
    damping: f32,
    angle_max: f32,
    angle_min: f32,
    elasticity: f32,
    is_visible: bool,
    show_bracket: bool,
    material: String,
    image: String,
    surface: String,
    pub name: String,
    pub is_reflection_enabled: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl BiffRead for Spinner {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut rotation: f32 = 0.0;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = 0;
        let mut height: f32 = 60.0;
        let mut length: f32 = 80.0;
        let mut damping: f32 = 0.9879;
        let mut angle_max: f32 = 0.0;
        let mut angle_min: f32 = 0.0;
        let mut elasticity: f32 = 0.3;
        let mut is_visible: bool = true;
        let mut show_bracket: bool = true;
        let mut material = Default::default();
        let mut image = Default::default();
        let mut surface = Default::default();
        let mut name = Default::default();
        let mut is_reflection_enabled: bool = false;

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
                "ROTA" => {
                    rotation = reader.get_f32();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "HIGH" => {
                    height = reader.get_f32();
                }
                "LGTH" => {
                    length = reader.get_f32();
                }
                "AFRC" => {
                    damping = reader.get_f32();
                }
                "SMAX" => {
                    angle_max = reader.get_f32();
                }
                "SMIN" => {
                    angle_min = reader.get_f32();
                }
                "SELA" => {
                    elasticity = reader.get_f32();
                }
                "SVIS" => {
                    is_visible = reader.get_bool();
                }
                "SSUP" => {
                    show_bracket = reader.get_bool();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "IMGF" => {
                    image = reader.get_string();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "REEN" => {
                    is_reflection_enabled = reader.get_bool();
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
            center,
            rotation,
            is_timer_enabled,
            timer_interval,
            height,
            length,
            damping,
            angle_max,
            angle_min,
            elasticity,
            is_visible,
            show_bracket,
            material,
            image,
            surface,
            name,
            is_reflection_enabled,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}
