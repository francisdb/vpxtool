use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct Kicker {
    center: Vertex2D,
    radius: f32,
    is_timer_enabled: bool,
    timer_interval: u32,
    material: String,
    surface: String,
    is_enabled: bool,
    pub name: String,
    kicker_type: u32,
    scatter: f32,
    hit_accuracy: f32,
    hit_height: f32,
    orientation: f32,
    fall_through: bool,
    legacy_mode: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl Kicker {
    pub const KICKER_TYPE_INVISIBLE: u32 = 0;
    pub const KICKER_TYPE_HOLE: u32 = 1;
    pub const KICKER_TYPE_CUP: u32 = 2;
    pub const KICKER_TYPE_HOLE_SIMPLE: u32 = 3;
    pub const KICKER_TYPE_WILLIAMS: u32 = 4;
    pub const KICKER_TYPE_GOTTLIEB: u32 = 5;
    pub const KICKER_TYPE_CUP2: u32 = 6;
}

impl BiffRead for Kicker {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut radius: f32 = 25.0;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = 0;
        let mut material = Default::default();
        let mut surface = Default::default();
        let mut is_enabled: bool = true;
        let mut name = Default::default();
        let mut kicker_type: u32 = Kicker::KICKER_TYPE_HOLE;
        let mut scatter: f32 = 0.0;
        let mut hit_accuracy: f32 = 0.7;
        let mut hit_height: f32 = 40.0;
        let mut orientation: f32 = 0.0;
        let mut fall_through: bool = false;
        let mut legacy_mode: bool = true;

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
                "RADI" => {
                    radius = reader.get_f32();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "EBLD" => {
                    is_enabled = reader.get_bool();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "TYPE" => {
                    kicker_type = reader.get_u32();
                }
                "KSCT" => {
                    scatter = reader.get_f32();
                }
                "KHAC" => {
                    hit_accuracy = reader.get_f32();
                }
                "KHHI" => {
                    hit_height = reader.get_f32();
                }
                "KORI" => {
                    orientation = reader.get_f32();
                }
                "FATH" => {
                    fall_through = reader.get_bool();
                }
                "LEMO" => {
                    legacy_mode = reader.get_bool();
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
            radius,
            is_timer_enabled,
            timer_interval,
            material,
            surface,
            is_enabled,
            name,
            kicker_type,
            scatter,
            hit_accuracy,
            hit_height,
            orientation,
            fall_through,
            legacy_mode,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}
