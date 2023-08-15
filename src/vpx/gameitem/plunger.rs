use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct Plunger {
    center: Vertex2D,
    width: f32,
    height: f32,
    z_adjust: f32,
    stroke: f32,
    speed_pull: f32,
    speed_fire: f32,
    plunger_type: u32,
    anim_frames: u32,
    material: String,
    image: String,
    mech_strength: f32,
    is_mech_plunger: bool,
    auto_plunger: bool,
    park_position: f32,
    scatter_velocity: f32,
    momentum_xfer: f32,
    is_timer_enabled: bool,
    timer_interval: u32,
    is_visible: bool,
    is_reflection_enabled: bool,
    surface: String,
    pub name: String,
    tip_shape: String,
    rod_diam: f32,
    ring_gap: f32,
    ring_diam: f32,
    ring_width: f32,
    spring_diam: f32,
    spring_gauge: f32,
    spring_loops: f32,
    spring_end_loops: f32,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,
}

impl Plunger {
    pub const PLUNGER_TYPE_MODERN: u32 = 1;
    pub const PLUNGER_TYPE_FLAT: u32 = 2;
    pub const PLUNGER_TYPE_CUSTOM: u32 = 3;
}

impl BiffRead for Plunger {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut width: f32 = 25.0;
        let mut height: f32 = 20.0;
        let mut z_adjust: f32 = 0.0;
        let mut stroke: f32 = 80.0;
        let mut speed_pull: f32 = 0.5;
        let mut speed_fire: f32 = 80.0;
        let mut plunger_type: u32 = Self::PLUNGER_TYPE_MODERN;
        let mut anim_frames: u32 = 1;
        let mut material = String::default();
        let mut image = String::default();
        let mut mech_strength: f32 = 85.0;
        let mut is_mech_plunger: bool = false;
        let mut auto_plunger: bool = false;
        let mut park_position: f32 = 0.5 / 3.0;
        let mut scatter_velocity: f32 = 0.0;
        let mut momentum_xfer: f32 = 1.0;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = 0;
        let mut is_visible: bool = true;
        let mut is_reflection_enabled: bool = true;
        let mut surface = String::default();
        let mut name = Default::default();
        let mut tip_shape =
            "0 .34; 2 .6; 3 .64; 5 .7; 7 .84; 8 .88; 9 .9; 11 .92; 14 .92; 39 .84".to_string();
        let mut rod_diam: f32 = 0.6;
        let mut ring_gap: f32 = 2.0;
        let mut ring_diam: f32 = 0.94;
        let mut ring_width: f32 = 3.0;
        let mut spring_diam: f32 = 0.77;
        let mut spring_gauge: f32 = 1.38;
        let mut spring_loops: f32 = 8.0;
        let mut spring_end_loops: f32 = 2.5;

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
                "WDTH" => {
                    width = reader.get_f32();
                }
                "HIGH" => {
                    height = reader.get_f32();
                }
                "ZADJ" => {
                    z_adjust = reader.get_f32();
                }
                "HPSL" => {
                    stroke = reader.get_f32();
                }
                "SPDP" => {
                    speed_pull = reader.get_f32();
                }
                "SPDF" => {
                    speed_fire = reader.get_f32();
                }
                "TYPE" => {
                    plunger_type = reader.get_u32();
                }
                "ANFR" => {
                    anim_frames = reader.get_u32();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "IMAG" => {
                    image = reader.get_string();
                }
                "MEST" => {
                    mech_strength = reader.get_f32();
                }
                "MECH" => {
                    is_mech_plunger = reader.get_bool();
                }
                "APLG" => {
                    auto_plunger = reader.get_bool();
                }
                "MPRK" => {
                    park_position = reader.get_f32();
                }
                "PSCV" => {
                    scatter_velocity = reader.get_f32();
                }
                "MOMX" => {
                    momentum_xfer = reader.get_f32();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "VSBL" => {
                    is_visible = reader.get_bool();
                }
                "REEN" => {
                    is_reflection_enabled = reader.get_bool();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "TIPS" => {
                    tip_shape = reader.get_string();
                }
                "RODD" => {
                    rod_diam = reader.get_f32();
                }
                "RNGG" => {
                    ring_gap = reader.get_f32();
                }
                "RNGD" => {
                    ring_diam = reader.get_f32();
                }
                "RNGW" => {
                    ring_width = reader.get_f32();
                }
                "SPRD" => {
                    spring_diam = reader.get_f32();
                }
                "SPRG" => {
                    spring_gauge = reader.get_f32();
                }
                "SPRL" => {
                    spring_loops = reader.get_f32();
                }
                "SPRE" => {
                    spring_end_loops = reader.get_f32();
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
            width,
            height,
            z_adjust,
            stroke,
            speed_pull,
            speed_fire,
            plunger_type,
            anim_frames,
            material,
            image,
            mech_strength,
            is_mech_plunger,
            auto_plunger,
            park_position,
            scatter_velocity,
            momentum_xfer,
            is_timer_enabled,
            timer_interval,
            is_visible,
            is_reflection_enabled,
            surface,
            name,
            tip_shape,
            rod_diam,
            ring_gap,
            ring_diam,
            ring_width,
            spring_diam,
            spring_gauge,
            spring_loops,
            spring_end_loops,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}

impl BiffWrite for Plunger {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VCEN", &self.center);
        writer.write_tagged_f32("WDTH", self.width);
        writer.write_tagged_f32("HIGH", self.height);
        writer.write_tagged_f32("ZADJ", self.z_adjust);
        writer.write_tagged_f32("HPSL", self.stroke);
        writer.write_tagged_f32("SPDP", self.speed_pull);
        writer.write_tagged_f32("SPDF", self.speed_fire);
        writer.write_tagged_u32("TYPE", self.plunger_type);
        writer.write_tagged_u32("ANFR", self.anim_frames);
        writer.write_tagged_string("MATR", &self.material);
        writer.write_tagged_string("IMAG", &self.image);
        writer.write_tagged_f32("MEST", self.mech_strength);
        writer.write_tagged_bool("MECH", self.is_mech_plunger);
        writer.write_tagged_bool("APLG", self.auto_plunger);
        writer.write_tagged_f32("MPRK", self.park_position);
        writer.write_tagged_f32("PSCV", self.scatter_velocity);
        writer.write_tagged_f32("MOMX", self.momentum_xfer);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_u32("TMIN", self.timer_interval);
        writer.write_tagged_bool("VSBL", self.is_visible);
        writer.write_tagged_bool("REEN", self.is_reflection_enabled);
        writer.write_tagged_string("SURF", &self.surface);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_string("TIPS", &self.tip_shape);
        writer.write_tagged_f32("RODD", self.rod_diam);
        writer.write_tagged_f32("RNGG", self.ring_gap);
        writer.write_tagged_f32("RNGD", self.ring_diam);
        writer.write_tagged_f32("RNGW", self.ring_width);
        writer.write_tagged_f32("SPRD", self.spring_diam);
        writer.write_tagged_f32("SPRG", self.spring_gauge);
        writer.write_tagged_f32("SPRL", self.spring_loops);
        writer.write_tagged_f32("SPRE", self.spring_end_loops);
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
        let plunger = Plunger {
            center: Vertex2D::new(1.0, 2.0),
            width: 1.0,
            height: 1.0,
            z_adjust: 0.1,
            stroke: 2.0,
            speed_pull: 0.5,
            speed_fire: 3.0,
            plunger_type: Plunger::PLUNGER_TYPE_MODERN,
            anim_frames: 1,
            material: "test material".to_string(),
            image: "test image".to_string(),
            mech_strength: 85.0,
            is_mech_plunger: false,
            auto_plunger: false,
            park_position: 0.5 / 3.0,
            scatter_velocity: 0.0,
            momentum_xfer: 1.0,
            is_timer_enabled: false,
            timer_interval: 0,
            is_visible: true,
            is_reflection_enabled: true,
            surface: "test surface".to_string(),
            name: "test plunger".to_string(),
            tip_shape: "0 .34; 2 .6; 3 .64; 5 .7; 7 .84; 8 .88; 9 .9; 11 .92; 14 .92; 39 .83"
                .to_string(),
            rod_diam: 0.6,
            ring_gap: 2.0,
            ring_diam: 0.94,
            ring_width: 3.0,
            spring_diam: 0.77,
            spring_gauge: 1.38,
            spring_loops: 8.0,
            spring_end_loops: 2.5,
            is_locked: true,
            editor_layer: 0,
            editor_layer_name: Some("test layer".to_string()),
            editor_layer_visibility: Some(false),
        };
        let mut writer = BiffWriter::new();
        Plunger::biff_write(&plunger, &mut writer);
        let plunger_read = Plunger::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(plunger, plunger_read);
    }
}
