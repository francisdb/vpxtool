use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::dragpoint::DragPoint;

#[derive(Debug, PartialEq)]
pub struct Ramp {
    pub height_bottom: f32,             // 1
    pub height_top: f32,                // 2
    pub width_bottom: f32,              // 3
    pub width_top: f32,                 // 4
    pub material: String,               // 5
    pub is_timer_enabled: bool,         // 6
    pub timer_interval: u32,            // 7
    pub ramp_type: u32,                 // 8
    pub name: String,                   // 9
    pub image: String,                  // 10
    pub image_alignment: u32,           // 11
    pub image_walls: bool,              // 12
    pub left_wall_height: f32,          // 13
    pub right_wall_height: f32,         // 14
    pub left_wall_height_visible: f32,  // 15
    pub right_wall_height_visible: f32, // 16
    pub hit_event: bool,                // 17
    pub threshold: f32,                 // 18
    pub elasticity: f32,                // 19
    pub friction: f32,                  // 20
    pub scatter: f32,                   // 21
    pub is_collidable: bool,            // 22
    pub is_visible: bool,               // 23
    pub depth_bias: f32,                // 24
    pub wire_diameter: f32,             // 25
    pub wire_distance_x: f32,           // 26
    pub wire_distance_y: f32,           // 27
    pub is_reflection_enabled: bool,    // 28
    pub physics_material: String,       // 29
    pub overwrite_physics: bool,        // 30

    drag_points: Vec<DragPoint>,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl Ramp {
    pub const RAMP_IMAGE_ALIGNMENT_MODE_WORLD: u32 = 0;
    pub const RAMP_IMAGE_ALIGNMENT_MODE_WRAP: u32 = 1;

    pub const RAMP_TYPE_FLAT: u32 = 0;
    pub const RAMP_TYPE_4_WIRE: u32 = 1;
    pub const RAMP_TYPE_2_WIRE: u32 = 2;
    pub const RAMP_TYPE_3_WIRE_LEFT: u32 = 3;
    pub const RAMP_TYPE_3_WIRE_RIGHT: u32 = 4;
    pub const RAMP_TYPE_1_WIRE: u32 = 5;
}

impl BiffRead for Ramp {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut height_bottom: f32 = 0.0;
        let mut height_top: f32 = 50.0;
        let mut width_bottom: f32 = 75.0;
        let mut width_top: f32 = 60.0;
        let mut material: String = Default::default();
        let mut is_timer_enabled: bool = Default::default();
        let mut timer_interval: u32 = Default::default();
        let mut ramp_type: u32 = Ramp::RAMP_TYPE_FLAT;
        let mut name = Default::default();
        let mut image = Default::default();
        let mut image_alignment: u32 = Ramp::RAMP_IMAGE_ALIGNMENT_MODE_WORLD;
        let mut image_walls: bool = true;
        let mut left_wall_height: f32 = 62.0;
        let mut right_wall_height: f32 = 62.0;
        let mut left_wall_height_visible: f32 = 30.0;
        let mut right_wall_height_visible: f32 = 30.0;
        let mut hit_event: bool = Default::default();
        let mut threshold: f32 = Default::default();
        let mut elasticity: f32 = Default::default();
        let mut friction: f32 = Default::default();
        let mut scatter: f32 = Default::default();
        let mut is_collidable: bool = true;
        let mut is_visible: bool = true;
        let mut depth_bias: f32 = 0.0;
        let mut wire_diameter: f32 = 8.0;
        let mut wire_distance_x: f32 = 38.0;
        let mut wire_distance_y: f32 = 88.0;
        let mut is_reflection_enabled: bool = true;
        let mut physics_material: String = Default::default();
        let mut overwrite_physics: bool = true;

        let mut drag_points: Vec<DragPoint> = Default::default();

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
                "HTBT" => {
                    height_bottom = reader.get_f32();
                }
                "HTTP" => {
                    height_top = reader.get_f32();
                }
                "WDBT" => {
                    width_bottom = reader.get_f32();
                }
                "WDTP" => {
                    width_top = reader.get_f32();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "TYPE" => {
                    ramp_type = reader.get_u32();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "IMAG" => {
                    image = reader.get_string();
                }
                "ALGN" => {
                    image_alignment = reader.get_u32();
                }
                "IMGW" => {
                    image_walls = reader.get_bool();
                }
                "WLHL" => {
                    left_wall_height = reader.get_f32();
                }
                "WLHR" => {
                    right_wall_height = reader.get_f32();
                }
                "WVHL" => {
                    left_wall_height_visible = reader.get_f32();
                }
                "WVHR" => {
                    right_wall_height_visible = reader.get_f32();
                }
                "HTEV" => {
                    hit_event = reader.get_bool();
                }
                "THRS" => {
                    threshold = reader.get_f32();
                }
                "ELAS" => {
                    elasticity = reader.get_f32();
                }
                "RFCT" => {
                    friction = reader.get_f32();
                }
                "RSCT" => {
                    scatter = reader.get_f32();
                }
                "CLDR" => {
                    is_collidable = reader.get_bool();
                }
                "RVIS" => {
                    is_visible = reader.get_bool();
                }
                "RAMP" => {
                    ramp_type = reader.get_u32();
                }
                "RADB" => {
                    depth_bias = reader.get_f32();
                }
                "RADI" => {
                    wire_diameter = reader.get_f32();
                }
                "RADX" => {
                    wire_distance_x = reader.get_f32();
                }
                "RADY" => {
                    wire_distance_y = reader.get_f32();
                }
                "REEN" => {
                    is_reflection_enabled = reader.get_bool();
                }
                "MAPH" => {
                    physics_material = reader.get_string();
                }
                "OVPH" => {
                    overwrite_physics = reader.get_bool();
                }
                "PNTS" => {
                    // this is just a tag with no data
                }
                "DPNT" => {
                    let point = DragPoint::biff_read(reader);
                    drag_points.push(point);
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
        Ramp {
            height_bottom,
            height_top,
            width_bottom,
            width_top,
            material,
            is_timer_enabled,
            timer_interval,
            ramp_type,
            name,
            image,
            image_alignment,
            image_walls,
            left_wall_height,
            right_wall_height,
            left_wall_height_visible,
            right_wall_height_visible,
            hit_event,
            threshold,
            elasticity,
            friction,
            scatter,
            is_collidable,
            is_visible,
            depth_bias,
            wire_diameter,
            wire_distance_x,
            wire_distance_y,
            is_reflection_enabled,
            physics_material,
            overwrite_physics,
            drag_points,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}
