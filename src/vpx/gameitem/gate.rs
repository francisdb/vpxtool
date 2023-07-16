use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct Gate {
    pub center: Vertex2D,            // 1
    pub length: f32,                 // 2
    pub height: f32,                 // 3
    pub rotation: f32,               // 4
    pub material: String,            // 5
    pub is_timer_enabled: bool,      // 6
    pub show_bracket: bool,          // 7
    pub is_collidable: bool,         // 8
    pub timer_interval: f32,         // 9
    pub surface: String,             // 10
    pub elasticity: f32,             // 11
    pub angle_max: f32,              // 12
    pub angle_min: f32,              // 13
    pub friction: f32,               // 14
    pub damping: f32,                // 15
    pub gravity_factor: f32,         // 16
    pub is_visible: bool,            // 17
    pub name: String,                // 18
    pub two_way: bool,               // 19
    pub is_reflection_enabled: bool, // 20
    pub gate_type: u32,              // 21

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl BiffRead for Gate {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut length: f32 = 100.0;
        let mut height: f32 = 50.0;
        let mut rotation: f32 = -90.0;
        let mut material: String = Default::default();
        let mut is_timer_enabled: bool = false;
        let mut show_bracket: bool = true;
        let mut is_collidable: bool = true;
        let mut timer_interval: f32 = Default::default();
        let mut surface: String = Default::default();
        let mut elasticity: f32 = 0.3;
        let mut angle_max: f32 = std::f32::consts::PI / 2.0;
        let mut angle_min: f32 = Default::default();
        let mut friction: f32 = 0.02;
        let mut damping: f32 = 0.985;
        let mut is_visible: bool = true;
        let mut name: String = Default::default();
        let mut two_way: bool = false;
        let mut is_reflection_enabled: bool = true;
        let mut gate_type: u32 = Default::default();
        let mut gravity_factor: f32 = 0.25;

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
                "LGTH" => {
                    length = reader.get_f32();
                }
                "HGTH" => {
                    height = reader.get_f32();
                }
                "ROTA" => {
                    rotation = reader.get_f32();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "GSUP" => {
                    show_bracket = reader.get_bool();
                }
                "GCOL" => {
                    is_collidable = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_f32();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "ELAS" => {
                    elasticity = reader.get_f32();
                }
                "GAMA" => {
                    angle_max = reader.get_f32();
                }
                "GAMI" => {
                    angle_min = reader.get_f32();
                }
                "GFRC" => {
                    friction = reader.get_f32();
                }
                "AFRC" => {
                    damping = reader.get_f32();
                }
                "GGFC" => {
                    gravity_factor = reader.get_f32();
                }
                "GVSB" => {
                    is_visible = reader.get_bool();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "TWWA" => {
                    two_way = reader.get_bool();
                }
                "REEN" => {
                    is_reflection_enabled = reader.get_bool();
                }
                "GATY" => {
                    gate_type = reader.get_u32();
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
            length,
            height,
            rotation,
            material,
            is_timer_enabled,
            show_bracket,
            is_collidable,
            timer_interval,
            surface,
            elasticity,
            angle_max,
            angle_min,
            friction,
            damping,
            gravity_factor,
            is_visible,
            name,
            two_way,
            is_reflection_enabled,
            gate_type,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}
