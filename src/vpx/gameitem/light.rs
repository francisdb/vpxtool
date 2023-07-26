use crate::vpx::{
    biff::{self, BiffRead, BiffReader},
    color::Color,
};

use super::{dragpoint::DragPoint, vertex2d::Vertex2D};

#[derive(Debug, PartialEq)]
pub struct Light {
    pub center: Vertex2D,
    pub falloff_radius: f32,
    pub falloff_power: f32,
    pub status: u32,
    pub color: Color,
    pub color2: Color,
    pub is_timer_enabled: bool,
    pub timer_interval: u32,
    pub blink_pattern: String,
    pub off_image: String,
    pub blink_interval: u32,
    pub intensity: f32,
    pub transmission_scale: f32,
    pub surface: String,
    pub name: String,
    pub is_backglass: bool,
    pub depth_bias: f32,
    pub fade_speed_up: f32,
    pub fade_speed_down: f32,
    pub is_bulb_light: bool,
    pub is_image_mode: bool,
    pub show_bulb_mesh: bool,
    pub has_static_bulb_mesh: bool,
    pub show_reflection_on_ball: bool,
    pub mesh_radius: f32,
    pub bulb_modulate_vs_add: f32,
    pub bulb_halo_height: f32,
    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
    // last
    pub drag_points: Vec<DragPoint>,
}

impl BiffRead for Light {
    fn biff_read(reader: &mut BiffReader<'_>) -> Light {
        let mut name = Default::default();
        let mut center: Vertex2D = Default::default();
        let mut falloff_radius: f32 = Default::default();
        let mut falloff_power: f32 = Default::default();
        let mut status: u32 = Default::default();
        // should these not have alpha ff?
        let mut color: Color = Color::new_argb(0xffff00);
        let mut color2: Color = Color::new_argb(0xffffff);
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = Default::default();
        let mut blink_pattern: String = "10".to_owned();
        let mut off_image: String = Default::default();
        let mut blink_interval: u32 = Default::default();
        let mut intensity: f32 = 1.0;
        let mut transmission_scale: f32 = 0.5;
        let mut surface: String = Default::default();
        let mut is_backglass: bool = false;
        let mut depth_bias: f32 = Default::default();
        let mut fade_speed_up: f32 = 0.2;
        let mut fade_speed_down: f32 = 0.2;
        let mut is_bulb_light: bool = false;
        let mut is_image_mode: bool = false;
        let mut show_bulb_mesh: bool = false;
        let mut has_static_bulb_mesh: bool = true;
        let mut show_reflection_on_ball: bool = true;
        let mut mesh_radius: f32 = 20.0;
        let mut bulb_modulate_vs_add: f32 = 0.9;
        let mut bulb_halo_height: f32 = 28.0;

        // these are shared between all items
        let mut is_locked: bool = false;
        let mut editor_layer: u32 = Default::default();
        let mut editor_layer_name: String = Default::default();
        let mut editor_layer_visibility: bool = true;

        // last
        let mut points: Vec<DragPoint> = Default::default();

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
                    falloff_radius = reader.get_f32();
                }
                "FAPO" => {
                    falloff_power = reader.get_f32();
                }
                "STAT" => {
                    status = reader.get_u32();
                }
                "COLR" => {
                    color = Color::biff_read_bgr(reader);
                }
                "COL2" => {
                    color2 = Color::biff_read_bgr(reader);
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "BPAT" => {
                    blink_pattern = reader.get_string();
                }
                "IMG1" => {
                    off_image = reader.get_string();
                }
                "BINT" => {
                    blink_interval = reader.get_u32();
                }
                "BWTH" => {
                    intensity = reader.get_f32();
                }
                "TRMS" => {
                    transmission_scale = reader.get_f32();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "NAME" => {
                    name = reader.get_wide_string();
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

                "BGLS" => {
                    is_backglass = reader.get_bool();
                }
                "LIDB" => {
                    depth_bias = reader.get_f32();
                }
                "FASP" => {
                    fade_speed_up = reader.get_f32();
                }
                "FASD" => {
                    fade_speed_down = reader.get_f32();
                }
                "BULT" => {
                    is_bulb_light = reader.get_bool();
                }
                "IMMO" => {
                    is_image_mode = reader.get_bool();
                }
                "SHBM" => {
                    show_bulb_mesh = reader.get_bool();
                }
                "STBM" => {
                    has_static_bulb_mesh = reader.get_bool();
                }
                "SHRB" => {
                    show_reflection_on_ball = reader.get_bool();
                }
                "BMSC" => {
                    mesh_radius = reader.get_f32();
                }
                "BMVA" => {
                    bulb_modulate_vs_add = reader.get_f32();
                }
                "BHHI" => {
                    bulb_halo_height = reader.get_f32();
                }
                // many of these
                "DPNT" => {
                    let point = DragPoint::biff_read(reader);
                    points.push(point);
                }
                other => {
                    println!(
                        "Unknown tag {} for {}",
                        other,
                        std::any::type_name::<Self>()
                    );
                    reader.skip_tag();
                }
            }
        }
        Light {
            name,
            center,
            falloff_radius,
            falloff_power,
            status,
            color,
            color2,
            is_timer_enabled,
            timer_interval,
            blink_pattern,
            off_image,
            blink_interval,
            intensity,
            transmission_scale,
            surface,
            is_backglass,
            depth_bias,
            fade_speed_up,
            fade_speed_down,
            is_bulb_light,
            is_image_mode,
            show_bulb_mesh,
            has_static_bulb_mesh,
            show_reflection_on_ball,
            mesh_radius,
            bulb_modulate_vs_add,
            bulb_halo_height,

            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,

            drag_points: points,
        }
    }
}
