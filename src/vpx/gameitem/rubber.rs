use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::dragpoint::DragPoint;

#[derive(Debug, PartialEq)]
pub struct Rubber {
    pub height: f32,
    pub hit_height: f32,
    pub thickness: i32,
    pub hit_event: bool,
    pub material: String,
    pub is_timer_enabled: bool,
    pub timer_interval: i32,
    pub name: String,
    pub image: String,
    pub elasticity: f32,
    pub elasticity_falloff: f32,
    pub friction: f32,
    pub scatter: f32,
    pub is_collidable: bool,
    pub is_visible: bool,
    pub static_rendering: bool,
    pub show_in_editor: bool,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub is_reflection_enabled: bool,
    pub physics_material: String,
    pub overwrite_physics: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,

    points: Vec<DragPoint>,
}

impl BiffRead for Rubber {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut height: f32 = 25.0;
        let mut hit_height: f32 = 25.0;
        let mut thickness: i32 = 8;
        let mut hit_event: bool = false;
        let mut material: String = Default::default();
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: i32 = Default::default();
        let mut name: String = Default::default();
        let mut image: String = Default::default();
        let mut elasticity: f32 = Default::default();
        let mut elasticity_falloff: f32 = Default::default();
        let mut friction: f32 = Default::default();
        let mut scatter: f32 = Default::default();
        let mut is_collidable: bool = true;
        let mut is_visible: bool = true;
        let mut static_rendering: bool = true;
        let mut show_in_editor: bool = true;
        let mut rot_x: f32 = 0.0;
        let mut rot_y: f32 = 0.0;
        let mut rot_z: f32 = 0.0;
        let mut is_reflection_enabled: bool = true;
        let mut physics_material: String = Default::default();
        let mut overwrite_physics: bool = false;

        // these are shared between all items
        let mut is_locked: bool = false;
        let mut editor_layer: u32 = Default::default();
        let mut editor_layer_name: String = Default::default();
        let mut editor_layer_visibility: bool = true;

        let mut points: Vec<DragPoint> = Default::default();

        loop {
            reader.next(biff::WARN);
            if reader.is_eof() {
                break;
            }
            let tag = reader.tag();
            let tag_str = tag.as_str();
            match tag_str {
                "HTTP" => {
                    height = reader.get_f32();
                }
                "HTHI" => {
                    hit_height = reader.get_f32();
                }
                "WDTP" => {
                    thickness = reader.get_i32();
                }
                "HTEV" => {
                    hit_event = reader.get_bool();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_i32();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "IMAG" => {
                    image = reader.get_string();
                }
                "ELAS" => {
                    elasticity = reader.get_f32();
                }
                "ELFO" => {
                    elasticity_falloff = reader.get_f32();
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
                "ESTR" => {
                    static_rendering = reader.get_bool();
                }
                "ESIE" => {
                    show_in_editor = reader.get_bool();
                }
                "ROTX" => {
                    rot_x = reader.get_f32();
                }
                "ROTY" => {
                    rot_y = reader.get_f32();
                }
                "ROTZ" => {
                    rot_z = reader.get_f32();
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

                "PNTS" => {
                    // this is just a tag with no data
                }
                "DPNT" => {
                    let point = DragPoint::biff_read(reader);
                    points.push(point);
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
        Rubber {
            height,
            hit_height,
            thickness,
            hit_event,
            material,
            is_timer_enabled,
            timer_interval,
            name,
            image,
            elasticity,
            elasticity_falloff,
            friction,
            scatter,
            is_collidable,
            is_visible,
            static_rendering,
            show_in_editor,
            rot_x,
            rot_y,
            rot_z,
            is_reflection_enabled,
            physics_material,
            overwrite_physics,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
            points,
        }
    }
}
