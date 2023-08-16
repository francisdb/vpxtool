use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

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
    pub disable_lighting_top: f32,           // DILI
    pub disable_lighting_below: Option<f32>, // DILB (added in 10.?)
    pub depth_bias: f32,
    pub is_reflection_enabled: bool,
    pub is_dropped: bool,
    pub drop_speed: f32,
    pub is_timer_enabled: bool,
    pub timer_interval: u32,
    pub raise_delay: Option<u32>,         // RADE (added in 10.?)
    pub physics_material: Option<String>, // MAPH (added in 10.?)
    pub overwrite_physics: Option<bool>,  // OVPH (added in 10.?)

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,
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
        let mut disable_lighting_below: Option<f32> = None; //0.0;
        let mut depth_bias: f32 = 0.0;
        let mut is_reflection_enabled: bool = true;
        let mut is_dropped: bool = false;
        let mut drop_speed: f32 = 0.5;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = 0;
        let mut raise_delay: Option<u32> = None; //100;
        let mut physics_material: Option<String> = None;
        let mut overwrite_physics: Option<bool> = None; //false;

        // these are shared between all items
        let mut is_locked: bool = false;
        let mut editor_layer: u32 = Default::default();
        let mut editor_layer_name: Option<String> = None;
        let mut editor_layer_visibility: Option<bool> = None;

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
                    disable_lighting_below = Some(reader.get_f32());
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
                "RADE" => raise_delay = Some(reader.get_u32()),
                "MAPH" => physics_material = Some(reader.get_string()),
                "OVPH" => overwrite_physics = Some(reader.get_bool()),

                // shared
                "LOCK" => {
                    is_locked = reader.get_bool();
                }
                "LAYR" => {
                    editor_layer = reader.get_u32();
                }
                "LANR" => {
                    editor_layer_name = Some(reader.get_string());
                }
                "LVIS" => {
                    editor_layer_visibility = Some(reader.get_bool());
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

impl BiffWrite for HitTarget {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VPOS", &self.position);
        writer.write_tagged("VSIZ", &self.size);
        writer.write_tagged_f32("ROTZ", self.rot_z);
        writer.write_tagged_string("IMAG", &self.image);
        writer.write_tagged_i32("TRTY", self.target_type);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_string("MATR", &self.material);
        writer.write_tagged_bool("TVIS", self.is_visible);
        writer.write_tagged_bool("LEMO", self.is_legacy);
        writer.write_tagged_bool("HTEV", self.use_hit_event);
        writer.write_tagged_f32("THRS", self.threshold);
        writer.write_tagged_f32("ELAS", self.elasticity);
        writer.write_tagged_f32("ELFO", self.elasticity_falloff);
        writer.write_tagged_f32("RFCT", self.friction);
        writer.write_tagged_f32("RSCT", self.scatter);
        writer.write_tagged_bool("CLDR", self.is_collidable);
        writer.write_tagged_f32("DILI", self.disable_lighting_top);
        if let Some(disable_lighting_below) = self.disable_lighting_below {
            writer.write_tagged_f32("DILB", disable_lighting_below);
        }
        writer.write_tagged_bool("REEN", self.is_reflection_enabled);
        writer.write_tagged_f32("PIDB", self.depth_bias);
        writer.write_tagged_bool("ISDR", self.is_dropped);
        writer.write_tagged_f32("DRSP", self.drop_speed);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_u32("TMIN", self.timer_interval);
        if let Some(raise_delay) = self.raise_delay {
            writer.write_tagged_u32("RADE", raise_delay);
        }
        if let Some(physics_material) = &self.physics_material {
            writer.write_tagged_string("MAPH", physics_material);
        }
        if let Some(overwrite_physics) = self.overwrite_physics {
            writer.write_tagged_bool("OVPH", overwrite_physics);
        }
        // shared
        writer.write_tagged_bool("LOCK", self.is_locked);
        writer.write_tagged_u32("LAYR", self.editor_layer);
        if let Some(editor_layer_name) = &self.editor_layer_name {
            writer.write_tagged_string("LANR", editor_layer_name);
        }
        if let Some(editor_layer_visibility) = self.editor_layer_visibility {
            writer.write_tagged_bool("LVIS", editor_layer_visibility);
        }

        writer.close(true);
    }
}

#[cfg(test)]
mod tests {
    use crate::vpx::biff::BiffWriter;

    use super::*;
    use pretty_assertions::assert_eq;
    use rand::Rng;

    #[test]
    fn test_write_read() {
        let mut rng = rand::thread_rng();
        // values not equal to the defaults
        let hittarget = HitTarget {
            position: Vertex3D::new(rng.gen(), rng.gen(), rng.gen()),
            size: Vertex3D::new(rng.gen(), rng.gen(), rng.gen()),
            rot_z: rng.gen(),
            image: "test image".to_string(),
            target_type: rng.gen(),
            name: "test name".to_string(),
            material: "test material".to_string(),
            is_visible: rng.gen(),
            is_legacy: rng.gen(),
            use_hit_event: rng.gen(),
            threshold: rng.gen(),
            elasticity: rng.gen(),
            elasticity_falloff: rng.gen(),
            friction: rng.gen(),
            scatter: rng.gen(),
            is_collidable: rng.gen(),
            disable_lighting_top: rng.gen(),
            disable_lighting_below: rng.gen(),
            is_reflection_enabled: rng.gen(),
            depth_bias: rng.gen(),
            is_dropped: rng.gen(),
            drop_speed: rng.gen(),
            is_timer_enabled: rng.gen(),
            timer_interval: rng.gen(),
            raise_delay: rng.gen(),
            physics_material: Some("test physics material".to_string()),
            overwrite_physics: rng.gen(),
            is_locked: rng.gen(),
            editor_layer: rng.gen(),
            editor_layer_name: Some("test layer name".to_string()),
            editor_layer_visibility: rng.gen(),
        };
        let mut writer = BiffWriter::new();
        HitTarget::biff_write(&hittarget, &mut writer);
        let hittarget_read = HitTarget::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(hittarget, hittarget_read);
    }
}
