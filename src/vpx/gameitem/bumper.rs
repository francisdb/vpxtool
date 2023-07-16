use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::{vertex2d::Vertex2D, GameItem};

#[derive(Debug, PartialEq)]
pub struct Bumper {
    center: Vertex2D,
    radius: f32,
    is_timer_enabled: bool,
    timer_interval: i32,
    threshold: f32,
    force: f32,
    scatter: f32,
    height_scale: f32,
    ring_speed: f32,
    orientation: f32,
    ring_drop_offset: f32,
    cap_material: String,
    base_material: String,
    socket_material: String,
    ring_material: String,
    surface: String,
    name: String,
    is_cap_visible: bool,
    is_base_visible: bool,
    is_ring_visible: bool,
    is_socket_visible: bool,
    hit_event: bool,
    is_collidable: bool,
    is_reflection_enabled: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}
impl GameItem for Bumper {
    fn name(&self) -> &str {
        &self.name
    }
}

impl BiffRead for Bumper {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut radius: f32 = 45.0;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: i32 = 0;
        let mut threshold: f32 = 1.0;
        let mut force: f32 = 15.0;
        let mut scatter: f32 = 0.0;
        let mut height_scale: f32 = 90.0;
        let mut ring_speed: f32 = 0.5;
        let mut orientation: f32 = 0.0;
        let mut ring_drop_offset: f32 = 0.0;
        let mut cap_material: String = Default::default();
        let mut base_material: String = Default::default();
        let mut socket_material: String = Default::default();
        let mut ring_material: String = Default::default();
        let mut surface: String = Default::default();
        let mut name = Default::default();
        let mut is_cap_visible: bool = true;
        let mut is_base_visible: bool = true;
        let mut is_ring_visible: bool = true;
        let mut is_socket_visible: bool = true;
        let mut hit_event: bool = true;
        let mut is_collidable: bool = true;
        let mut is_reflection_enabled: bool = true;

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
                    timer_interval = reader.get_i32();
                }
                "THRS" => {
                    threshold = reader.get_f32();
                }
                "FORC" => {
                    force = reader.get_f32();
                }
                "BSCT" => {
                    scatter = reader.get_f32();
                }
                "HISC" => {
                    height_scale = reader.get_f32();
                }
                "RISP" => {
                    ring_speed = reader.get_f32();
                }
                "ORIN" => {
                    orientation = reader.get_f32();
                }
                "RDLI" => {
                    ring_drop_offset = reader.get_f32();
                }
                "MATR" => {
                    cap_material = reader.get_string();
                }
                "BAMA" => {
                    base_material = reader.get_string();
                }
                "SKMA" => {
                    socket_material = reader.get_string();
                }
                "RIMA" => {
                    ring_material = reader.get_string();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "CAVI" => {
                    is_cap_visible = reader.get_bool();
                }
                "BSVS" => {
                    is_base_visible = reader.get_bool();
                }
                "RIVS" => {
                    is_ring_visible = reader.get_bool();
                }
                "SKVS" => {
                    is_socket_visible = reader.get_bool();
                }
                "HAHE" => {
                    hit_event = reader.get_bool();
                }
                "COLI" => {
                    is_collidable = reader.get_bool();
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
            radius,
            is_timer_enabled,
            timer_interval,
            threshold,
            force,
            scatter,
            height_scale,
            ring_speed,
            orientation,
            ring_drop_offset,
            cap_material,
            base_material,
            socket_material,
            ring_material,
            surface,
            name,
            is_cap_visible,
            is_base_visible,
            is_ring_visible,
            is_socket_visible,
            hit_event,
            is_collidable,
            is_reflection_enabled,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}
