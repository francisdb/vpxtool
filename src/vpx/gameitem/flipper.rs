use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

use super::{vertex2d::Vertex2D, GameItem};

#[derive(Debug, PartialEq)]
pub struct Flipper {
    center: Vertex2D,
    base_radius: f32,
    end_radius: f32,
    flipper_radius_max: f32,
    return_: f32,
    start_angle: f32,
    end_angle: f32,
    override_physics: u32,
    mass: f32,
    is_timer_enabled: bool,
    timer_interval: u32,
    surface: String,
    material: String,
    name: String,
    rubber_material: String,
    rthk: f32,                     // RTHK deprecated?
    rubber_thickness: Option<f32>, // RTHF (added in 10.?)
    rhgt: f32,                     // RHGT deprecated?
    rubber_height: Option<f32>,    // RHGF (added in 10.?)
    rwdt: f32,                     // RWDT deprecated?
    rubber_width: Option<f32>,     // RHGF (added in 10.?)
    strength: f32,
    elasticity: f32,
    elasticity_falloff: f32,
    friction: f32,
    ramp_up: f32,
    scatter: Option<f32>,              // SCTR (added in 10.?)
    torque_damping: Option<f32>,       // TODA (added in 10.?)
    torque_damping_angle: Option<f32>, // TDAA (added in 10.?)
    flipper_radius_min: f32,
    is_visible: bool,
    is_enabled: bool,
    height: f32,
    image: String,
    is_reflection_enabled: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,
}

impl GameItem for Flipper {
    fn name(&self) -> &str {
        &self.name
    }
}

impl BiffRead for Flipper {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut base_radius: f32 = 21.5;
        let mut end_radius: f32 = 13.0;
        let mut flipper_radius_max: f32 = 130.0;
        let mut return_: f32 = 0.058;
        let mut start_angle: f32 = 121.0;
        let mut end_angle: f32 = 70.0;
        let mut override_physics: u32 = 0;
        let mut mass: f32 = 1.0;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = 0;
        let mut surface: String = Default::default();
        let mut material: String = Default::default();
        let mut name: String = Default::default();
        let mut rubber_material: String = Default::default();
        let mut rthk: f32 = 0.0;
        let mut rubber_thickness: Option<f32> = None; //7.0;
        let mut rhgt: f32 = 0.0;
        let mut rubber_height: Option<f32> = None; //19.0;
        let mut rwdt: f32 = 0.0;
        let mut rubber_width: Option<f32> = None; //24.0;
        let mut strength: f32 = 2200.0;
        let mut elasticity: f32 = 0.8;
        let mut elasticity_falloff: f32 = 0.43;
        let mut friction: f32 = 0.6;
        let mut ramp_up: f32 = 3.0;
        let mut scatter: Option<f32> = None; //0.0;
        let mut torque_damping: Option<f32> = None; //0.75;
        let mut torque_damping_angle: Option<f32> = None; //6.0;
        let mut flipper_radius_min: f32 = 0.0;
        let mut is_visible: bool = true;
        let mut is_enabled: bool = true;
        let mut height: f32 = 50.0;
        let mut image: String = Default::default();
        let mut is_reflection_enabled: bool = true;

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
                "VCEN" => {
                    center = Vertex2D::biff_read(reader);
                }
                "BASR" => {
                    base_radius = reader.get_f32();
                }
                "ENDR" => {
                    end_radius = reader.get_f32();
                }
                "FLPR" => {
                    flipper_radius_max = reader.get_f32();
                }
                "FRTN" => {
                    return_ = reader.get_f32();
                }
                "ANGS" => {
                    start_angle = reader.get_f32();
                }
                "ANGE" => {
                    end_angle = reader.get_f32();
                }
                "OVRP" => {
                    override_physics = reader.get_u32();
                }
                "FORC" => {
                    mass = reader.get_f32();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "RUMA" => {
                    rubber_material = reader.get_string();
                }
                "RTHK" => {
                    rthk = reader.get_f32();
                }
                "RTHF" => {
                    rubber_thickness = Some(reader.get_f32());
                }
                "RHGT" => {
                    rhgt = reader.get_f32();
                }
                "RHGF" => {
                    rubber_height = Some(reader.get_f32());
                }
                "RWDT" => {
                    rwdt = reader.get_f32();
                }
                "RWDF" => {
                    rubber_width = Some(reader.get_f32());
                }
                "STRG" => {
                    strength = reader.get_f32();
                }
                "ELAS" => {
                    elasticity = reader.get_f32();
                }
                "ELFO" => {
                    elasticity_falloff = reader.get_f32();
                }
                "FRIC" => {
                    friction = reader.get_f32();
                }
                "RPUP" => {
                    ramp_up = reader.get_f32();
                }
                "SCTR" => {
                    scatter = Some(reader.get_f32());
                }
                "TODA" => {
                    torque_damping = Some(reader.get_f32());
                }
                "TDAA" => {
                    torque_damping_angle = Some(reader.get_f32());
                }
                "VSBL" => {
                    is_visible = reader.get_bool();
                }
                "ENBL" => {
                    is_enabled = reader.get_bool();
                }
                "FRMN" => {
                    flipper_radius_min = reader.get_f32();
                }
                "FHGT" => {
                    height = reader.get_f32();
                }
                "IMAG" => {
                    image = reader.get_string();
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
            center,
            base_radius,
            end_radius,
            flipper_radius_max,
            return_,
            start_angle,
            end_angle,
            override_physics,
            strength,
            is_timer_enabled,
            timer_interval,
            surface,
            material,
            name,
            rubber_material,
            rthk,
            rubber_thickness,
            rhgt,
            rubber_height,
            rwdt,
            rubber_width,
            elasticity,
            friction,
            ramp_up,
            scatter,
            torque_damping,
            torque_damping_angle,
            is_visible,
            is_enabled,
            flipper_radius_min,
            height,
            image,
            is_reflection_enabled,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
            mass,
            elasticity_falloff,
        }
    }
}

impl BiffWrite for Flipper {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VCEN", &self.center);
        writer.write_tagged_f32("BASR", self.base_radius);
        writer.write_tagged_f32("ENDR", self.end_radius);
        writer.write_tagged_f32("FLPR", self.flipper_radius_max);
        writer.write_tagged_f32("FRTN", self.return_);
        writer.write_tagged_f32("ANGS", self.start_angle);
        writer.write_tagged_f32("ANGE", self.end_angle);
        writer.write_tagged_u32("OVRP", self.override_physics);
        writer.write_tagged_f32("FORC", self.mass);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_u32("TMIN", self.timer_interval);
        writer.write_tagged_string("SURF", &self.surface);
        writer.write_tagged_string("MATR", &self.material);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_string("RUMA", &self.rubber_material);
        writer.write_tagged_f32("RTHK", self.rthk);
        if let Some(rubber_thickness) = self.rubber_thickness {
            writer.write_tagged_f32("RTHF", rubber_thickness);
        }
        writer.write_tagged_f32("RHGT", self.rhgt);
        if let Some(rubber_height) = self.rubber_height {
            writer.write_tagged_f32("RHGF", rubber_height);
        }
        writer.write_tagged_f32("RWDT", self.rwdt);
        if let Some(rubber_width) = self.rubber_width {
            writer.write_tagged_f32("RWDF", rubber_width);
        }
        writer.write_tagged_f32("STRG", self.strength);
        writer.write_tagged_f32("ELAS", self.elasticity);
        writer.write_tagged_f32("ELFO", self.elasticity_falloff);
        writer.write_tagged_f32("FRIC", self.friction);
        writer.write_tagged_f32("RPUP", self.ramp_up);
        if let Some(sctr) = self.scatter {
            writer.write_tagged_f32("SCTR", sctr);
        }
        if let Some(toda) = self.torque_damping {
            writer.write_tagged_f32("TODA", toda);
        }
        if let Some(tdaa) = self.torque_damping_angle {
            writer.write_tagged_f32("TDAA", tdaa);
        }
        writer.write_tagged_bool("VSBL", self.is_visible);
        writer.write_tagged_bool("ENBL", self.is_enabled);
        writer.write_tagged_f32("FRMN", self.flipper_radius_min);
        writer.write_tagged_f32("FHGT", self.height);
        writer.write_tagged_string("IMAG", &self.image);
        writer.write_tagged_bool("REEN", self.is_reflection_enabled);
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

    #[test]
    fn test_write_read() {
        let flipper = Flipper {
            center: Vertex2D::new(0.0, 0.0),
            base_radius: 21.5,
            end_radius: 13.0,
            flipper_radius_max: 130.0,
            return_: 0.058,
            start_angle: 121.0,
            end_angle: 70.0,
            override_physics: 0,
            mass: 1.0,
            is_timer_enabled: false,
            timer_interval: 0,
            surface: String::from("test surface"),
            material: String::from("test material"),
            name: String::from("test name"),
            rubber_material: String::from("test rubber material"),
            rwdt: 0.0,
            rubber_thickness: Some(7.0),
            rhgt: 0.0,
            rubber_height: Some(19.0),
            rthk: 0.0,
            rubber_width: Some(24.0),
            strength: 2200.0,
            elasticity: 0.8,
            elasticity_falloff: 0.43,
            friction: 0.6,
            ramp_up: 3.0,
            scatter: Some(0.0),
            torque_damping: Some(0.75),
            torque_damping_angle: Some(6.0),
            flipper_radius_min: 0.0,
            is_visible: true,
            is_enabled: true,
            height: 50.0,
            image: String::from("test image"),
            is_reflection_enabled: true,
            is_locked: false,
            editor_layer: 123,
            editor_layer_name: Some(String::from("test editor layer name")),
            editor_layer_visibility: Some(true),
        };
        let mut writer = BiffWriter::new();
        Flipper::biff_write(&flipper, &mut writer);
        let flipper_read = Flipper::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(flipper, flipper_read);
    }
}
