use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::vertex3d::Vertex3D;

#[derive(Debug, PartialEq)]
pub struct HitTarget {
    pub position: Vertex3D,
    pub size: Vertex3D,
    pub rot_z: f32,
    pub image: String,
    pub target_type: i32,
    pub name: String,
    pub material: String,
    pub is_visible: bool,
    pub is_legacy: bool,
    pub use_hit_event: bool,
    pub threshold: f32,
    pub elasticity: f32,
    pub elasticity_falloff: f32,
    pub friction: f32,
    pub scatter: f32,
    pub is_collidable: bool,
    pub disable_lighting_top: f32,
    pub disable_lighting_below: f32,
    pub depth_bias: f32,
    pub is_reflection_enabled: bool,
    pub is_dropped: bool,
    pub drop_speed: f32,
    pub is_timer_enabled: bool,
    pub timer_interval: u32,
    pub raise_delay: u32,
    pub physics_material: String,
    pub overwrite_physics: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl HitTarget {
    pub const TARGET_TYPE_DROP_TARGET_BEVELED: i32 = 1;
    pub const TARGET_TYPE_DROP_TARGET_SIMPLE: i32 = 2;
    pub const TARGET_TYPE_HIT_TARGET_ROUND: i32 = 3;
    pub const TARGET_TYPE_HIT_TARGET_RECTANGLE: i32 = 4;
    pub const TARGET_TYPE_HIT_FAT_TARGET_RECTANGLE: i32 = 5;
    pub const TARGET_TYPE_HIT_FAT_TARGET_SQUARE: i32 = 6;
    pub const TARGET_TYPE_DROP_TARGET_FLAT_SIMPLE: i32 = 7;
    pub const TARGET_TYPE_HIT_FAT_TARGET_SLIM: i32 = 8;
    pub const TARGET_TYPE_HIT_TARGET_SLIM: i32 = 9;
}

impl BiffRead for HitTarget {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut position: Vertex3D = Default::default();
        let mut size = Vertex3D::new(32.0, 32.0, 32.0);
        let mut rot_z: f32 = 0.0;
        let mut image: String = Default::default();
        let mut target_type: i32 = HitTarget::TARGET_TYPE_DROP_TARGET_SIMPLE;
        let mut name: String = Default::default();
        let mut material: String = Default::default();
        let mut is_visible: bool = true;
        let mut is_legacy: bool = false;
        let mut use_hit_event: bool = true;
        let mut threshold: f32 = 2.0;
        let mut elasticity: f32 = 0.0;
        let mut elasticity_falloff: f32 = 0.0;
        let mut friction: f32 = 0.0;
        let mut scatter: f32 = 0.0;
        let mut is_collidable: bool = true;
        let mut disable_lighting_top: f32 = 0.0;
        let mut disable_lighting_below: f32 = 0.0;
        let mut depth_bias: f32 = 0.0;
        let mut is_reflection_enabled: bool = true;
        let mut is_dropped: bool = false;
        let mut drop_speed: f32 = 0.5;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = 0;
        let mut raise_delay: u32 = 100;
        let mut physics_material: String = Default::default();
        let mut overwrite_physics: bool = false;

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
                "VPOS" => {
                    position = Vertex3D::biff_read(reader);
                }
                "VSIZ" => {
                    size = Vertex3D::biff_read(reader);
                }
                "ROTZ" => {
                    rot_z = reader.get_f32();
                }
                "IMAG" => {
                    image = reader.get_string();
                }
                "TRTY" => {
                    target_type = reader.get_i32();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "TVIS" => {
                    is_visible = reader.get_bool();
                }
                "LEMO" => {
                    is_legacy = reader.get_bool();
                }
                "HTEV" => {
                    use_hit_event = reader.get_bool();
                }
                "THRS" => {
                    threshold = reader.get_f32();
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
                "DILI" => {
                    disable_lighting_top = reader.get_f32();
                }
                "DILB" => {
                    disable_lighting_below = reader.get_f32();
                }
                "REEN" => {
                    is_reflection_enabled = reader.get_bool();
                }
                "PIDB" => {
                    depth_bias = reader.get_f32();
                }
                "ISDR" => {
                    is_dropped = reader.get_bool();
                }
                "DRSP" => {
                    drop_speed = reader.get_f32();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => timer_interval = reader.get_u32(),
                "RADE" => {
                    raise_delay = reader.get_u32();
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
            position,
            size,
            rot_z,
            image,
            target_type,
            name,
            material,
            is_visible,
            is_legacy,
            use_hit_event,
            threshold,
            elasticity,
            elasticity_falloff,
            friction,
            scatter,
            is_collidable,
            disable_lighting_top,
            disable_lighting_below,
            is_reflection_enabled,
            depth_bias,
            is_dropped,
            drop_speed,
            is_timer_enabled,
            timer_interval,
            raise_delay,
            physics_material,
            overwrite_physics,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}
