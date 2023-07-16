use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct LightSequencer {
    center: Vertex2D,
    collection: String,
    pos_x: f32,
    pos_y: f32,
    update_interval: u32,
    is_timer_enabled: bool,
    timer_interval: u32,
    pub name: String,
    backglass: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl BiffRead for LightSequencer {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut collection = Default::default();
        let mut pos_x: f32 = Default::default();
        let mut pos_y: f32 = Default::default();
        let mut update_interval: u32 = 25;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = 0;
        let mut name = Default::default();
        let mut backglass: bool = false;

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
                "COLC" => {
                    collection = reader.get_string();
                }
                "CTRX" => {
                    pos_x = reader.get_f32();
                }
                "CTRY" => {
                    pos_y = reader.get_f32();
                }
                "UPTM" => {
                    update_interval = reader.get_u32();
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
                "BGLS" => {
                    backglass = reader.get_bool();
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
            collection,
            pos_x,
            pos_y,
            update_interval,
            is_timer_enabled,
            timer_interval,
            name,
            backglass,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}
