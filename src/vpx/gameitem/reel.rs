use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::{color::Color, vertex2d::Vertex2D};

#[derive(Debug, PartialEq)]
pub struct Reel {
    ver1: Vertex2D,
    ver2: Vertex2D,
    back_color: Color,
    is_timer_enabled: bool,
    timer_interval: u32,
    is_transparent: bool,
    image: String,
    sound: String,
    pub name: String,
    width: f32,
    height: f32,
    reel_count: u32,
    reel_spacing: f32,
    motor_steps: u32,
    digit_range: u32,
    update_interval: u32,
    use_image_grid: bool,
    is_visible: bool,
    images_per_grid_row: u32,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    // TODO we found at least one table where these two were missing
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,
}

impl BiffRead for Reel {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut ver1 = Vertex2D::default();
        let mut ver2 = Vertex2D::default();
        let mut back_color = Color::new_bgr(0x404040f);
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = Default::default();
        let mut is_transparent: bool = false;
        let mut image = Default::default();
        let mut sound = Default::default();
        let mut name = Default::default();
        let mut width: f32 = 30.0;
        let mut height: f32 = 40.0;
        let mut reel_count: u32 = 5;
        let mut reel_spacing: f32 = 4.0;
        let mut motor_steps: u32 = 2;
        let mut digit_range: u32 = 9;
        let mut update_interval: u32 = 50;
        let mut use_image_grid: bool = false;
        let mut is_visible: bool = true;
        let mut images_per_grid_row: u32 = 1;

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
                "VER1" => {
                    ver1 = Vertex2D::biff_read(reader);
                }
                "VER2" => {
                    ver2 = Vertex2D::biff_read(reader);
                }
                "CLRB" => {
                    back_color = Color::biff_read_bgr(reader);
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "TRNS" => {
                    is_transparent = reader.get_bool();
                }
                "IMAG" => {
                    image = reader.get_string();
                }
                "SOUN" => {
                    sound = reader.get_string();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "WDTH" => {
                    width = reader.get_f32();
                }
                "HIGH" => {
                    height = reader.get_f32();
                }
                "RCNT" => {
                    reel_count = reader.get_u32();
                }
                "RSPC" => {
                    reel_spacing = reader.get_f32();
                }
                "MSTP" => {
                    motor_steps = reader.get_u32();
                }
                "RANG" => {
                    digit_range = reader.get_u32();
                }
                "UPTM" => {
                    update_interval = reader.get_u32();
                }
                "UGRD" => {
                    use_image_grid = reader.get_bool();
                }
                "VISI" => {
                    is_visible = reader.get_bool();
                }
                "GIPR" => {
                    images_per_grid_row = reader.get_u32();
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
            ver1,
            ver2,
            back_color,
            is_timer_enabled,
            timer_interval,
            is_transparent,
            image,
            sound,
            name,
            width,
            height,
            reel_count,
            reel_spacing,
            motor_steps,
            digit_range,
            update_interval,
            use_image_grid,
            is_visible,
            images_per_grid_row,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}
