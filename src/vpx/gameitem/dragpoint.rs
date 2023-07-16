use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::GameItem;

#[derive(Debug, PartialEq)]
pub struct DragPoint {
    x: f32,
    y: f32,
    z: f32,
    smooth: bool,
    is_slingshot: Option<bool>,
    has_auto_texture: bool,
    tex_coord: f32,

    // Somehow below items don't belong here?
    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl GameItem for DragPoint {
    fn name(&self) -> &str {
        "Unnamed DragPoint"
    }
}

impl BiffRead for DragPoint {
    fn biff_read(reader: &mut BiffReader<'_>) -> DragPoint {
        let mut sub_data = reader.child_reader();
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut tex_coord = 0.0;
        let mut smooth = false;
        let mut is_slingshot: Option<bool> = None;
        let mut has_auto_texture = false;

        // these are shared between all items
        let mut is_locked: bool = false;
        let mut editor_layer: u32 = Default::default();
        let mut editor_layer_name: String = Default::default();
        let mut editor_layer_visibility: bool = true;
        loop {
            sub_data.next(biff::WARN);
            if sub_data.is_eof() {
                break;
            }
            let tag = sub_data.tag();
            let tag_str = tag.as_str();
            match tag_str {
                "VCEN" => {
                    x = sub_data.get_f32();
                    y = sub_data.get_f32();
                }
                "POSZ" => {
                    z = sub_data.get_f32();
                }
                "SMTH" => {
                    smooth = sub_data.get_bool();
                }
                "SLNG" => {
                    is_slingshot = Some(sub_data.get_bool());
                }
                "ATEX" => {
                    has_auto_texture = sub_data.get_bool();
                }
                "TEXC" => {
                    tex_coord = sub_data.get_f32();
                }
                // shared
                "LOCK" => {
                    is_locked = sub_data.get_bool();
                }
                "LAYR" => {
                    editor_layer = sub_data.get_u32();
                }
                "LANR" => {
                    editor_layer_name = sub_data.get_string();
                }
                "LVIS" => {
                    editor_layer_visibility = sub_data.get_bool();
                }
                other => {
                    println!(
                        "Unknown tag {} for {}",
                        tag_str,
                        std::any::type_name::<Self>()
                    );
                    sub_data.skip_tag();
                }
            }
        }
        let pos = sub_data.pos();
        reader.skip_end_tag(pos);
        DragPoint {
            x,
            y,
            z,
            smooth,
            is_slingshot,
            has_auto_texture,
            tex_coord,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}
