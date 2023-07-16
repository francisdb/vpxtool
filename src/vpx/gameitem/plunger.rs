use crate::vpx::biff::{self, BiffRead, BiffReader};

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
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
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
