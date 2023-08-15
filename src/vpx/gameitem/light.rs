use crate::vpx::{
    biff::{self, BiffRead, BiffReader, BiffWrite},
    color::Color,
};

use super::{dragpoint::DragPoint, vertex2d::Vertex2D};

#[derive(Debug, PartialEq)]
pub struct Light {
    pub center: Vertex2D,                   // VCEN
    pub height: Option<f32>,                // HGHT added in 10.8
    pub falloff_radius: f32,                // RADI
    pub falloff_power: f32,                 // FAPO
    pub status: u32,                        // STAT
    pub state: Option<f32>,                 // STTF added in 10.8
    pub color: Color,                       // COLR
    pub color2: Color,                      // COL2
    pub is_timer_enabled: bool,             // TMON
    pub timer_interval: u32,                // TMIN
    pub blink_pattern: String,              // BPAT
    pub off_image: String,                  // IMG1
    pub blink_interval: u32,                // BINT
    pub intensity: f32,                     // BWTH
    pub transmission_scale: f32,            // TRMS
    pub surface: String,                    // SURF
    pub name: String,                       // NAME
    pub is_backglass: bool,                 // BGLS
    pub depth_bias: f32,                    // LIDB
    pub fade_speed_up: f32,                 // FASP
    pub fade_speed_down: f32,               // FASD
    pub is_bulb_light: bool,                // BULT
    pub is_image_mode: bool,                // IMMO
    pub show_bulb_mesh: bool,               // SHBM
    pub has_static_bulb_mesh: Option<bool>, // STBM (added in 10.?)
    pub show_reflection_on_ball: bool,      // SHRB
    pub mesh_radius: f32,                   // BMSC
    pub bulb_modulate_vs_add: f32,          // BMVA
    pub bulb_halo_height: f32,              // BHHI
    pub shadows: Option<u32>,               // SHDW added in 10.8
    pub fader: Option<u32>,                 // FADE added in 10.8
    pub visible: Option<bool>,              // VSBL added in 10.8

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,
    // last
    pub drag_points: Vec<DragPoint>,
}

impl Light {
    // default
    pub fn default() -> Self {
        let name = Default::default();
        let height: Option<f32> = None;
        let center: Vertex2D = Default::default();
        let falloff_radius: f32 = Default::default();
        let falloff_power: f32 = Default::default();
        let status: u32 = Default::default();
        let state: Option<f32> = None;
        // should these not have alpha ff?
        let color: Color = Color::new_argb(0xffff00);
        let color2: Color = Color::new_argb(0xffffff);
        let is_timer_enabled: bool = false;
        let timer_interval: u32 = Default::default();
        let blink_pattern: String = "10".to_owned();
        let off_image: String = Default::default();
        let blink_interval: u32 = Default::default();
        let intensity: f32 = 1.0;
        let transmission_scale: f32 = 0.5;
        let surface: String = Default::default();
        let is_backglass: bool = false;
        let depth_bias: f32 = Default::default();
        let fade_speed_up: f32 = 0.2;
        let fade_speed_down: f32 = 0.2;
        let is_bulb_light: bool = false;
        let is_image_mode: bool = false;
        let show_bulb_mesh: bool = false;
        let has_static_bulb_mesh: Option<bool> = None; //true;
        let show_reflection_on_ball: bool = true;
        let mesh_radius: f32 = 20.0;
        let bulb_modulate_vs_add: f32 = 0.9;
        let bulb_halo_height: f32 = 28.0;
        let shadows: Option<u32> = None;
        let fader: Option<u32> = None;
        let visible: Option<bool> = None;

        // these are shared between all items
        let is_locked: bool = false;
        let editor_layer: u32 = Default::default();
        let editor_layer_name: Option<String> = None;
        let editor_layer_visibility: Option<bool> = None;
        Self {
            center,
            height,
            falloff_radius,
            falloff_power,
            status,
            state,
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
            name,
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
            shadows,
            fader,
            visible,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
            drag_points: Vec::new(),
        }
    }
}

impl BiffRead for Light {
    fn biff_read(reader: &mut BiffReader<'_>) -> Light {
        let mut light = Light::default();
        loop {
            reader.next(biff::WARN);
            if reader.is_eof() {
                break;
            }
            let tag = reader.tag();
            let tag_str = tag.as_str();
            match tag_str {
                "VCEN" => light.center = Vertex2D::biff_read(reader),
                "HGHT" => light.height = Some(reader.get_f32()),
                "RADI" => light.falloff_radius = reader.get_f32(),
                "FAPO" => light.falloff_power = reader.get_f32(),
                "STAT" => light.status = reader.get_u32(),
                "STTF" => light.state = Some(reader.get_f32()),
                "COLR" => light.color = Color::biff_read_bgr(reader),
                "COL2" => light.color2 = Color::biff_read_bgr(reader),
                "TMON" => light.is_timer_enabled = reader.get_bool(),
                "TMIN" => light.timer_interval = reader.get_u32(),
                "BPAT" => light.blink_pattern = reader.get_string(),
                "IMG1" => light.off_image = reader.get_string(),
                "BINT" => light.blink_interval = reader.get_u32(),
                "BWTH" => light.intensity = reader.get_f32(),
                "TRMS" => light.transmission_scale = reader.get_f32(),
                "SURF" => light.surface = reader.get_string(),
                "NAME" => light.name = reader.get_wide_string(),
                // shared
                "LOCK" => light.is_locked = reader.get_bool(),
                "LAYR" => light.editor_layer = reader.get_u32(),
                "LANR" => light.editor_layer_name = Some(reader.get_string()),
                "LVIS" => light.editor_layer_visibility = Some(reader.get_bool()),

                "BGLS" => light.is_backglass = reader.get_bool(),
                "LIDB" => light.depth_bias = reader.get_f32(),
                "FASP" => light.fade_speed_up = reader.get_f32(),
                "FASD" => light.fade_speed_down = reader.get_f32(),
                "BULT" => light.is_bulb_light = reader.get_bool(),
                "IMMO" => light.is_image_mode = reader.get_bool(),
                "SHBM" => light.show_bulb_mesh = reader.get_bool(),
                "STBM" => light.has_static_bulb_mesh = Some(reader.get_bool()),
                "SHRB" => light.show_reflection_on_ball = reader.get_bool(),
                "BMSC" => light.mesh_radius = reader.get_f32(),
                "BMVA" => light.bulb_modulate_vs_add = reader.get_f32(),
                "BHHI" => light.bulb_halo_height = reader.get_f32(),
                "SHDW" => light.shadows = Some(reader.get_u32()),
                "FADE" => light.fader = Some(reader.get_u32()),
                "VSBL" => light.visible = Some(reader.get_bool()),
                // many of these
                "DPNT" => {
                    let point = DragPoint::biff_read(reader);
                    light.drag_points.push(point);
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
        light
    }
}

impl BiffWrite for Light {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        // write all fields like n the read
        writer.write_tagged("VCEN", &self.center);
        if let Some(height) = self.height {
            writer.write_tagged_f32("HGHT", height);
        }
        writer.write_tagged_f32("RADI", self.falloff_radius);
        writer.write_tagged_f32("FAPO", self.falloff_power);
        writer.write_tagged_u32("STAT", self.status);
        if let Some(state) = self.state {
            writer.write_tagged_f32("STTF", state);
        }
        writer.write_tagged_with("COLR", &self.color, Color::biff_write_bgr);
        writer.write_tagged_with("COL2", &self.color2, Color::biff_write_bgr);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_u32("TMIN", self.timer_interval);
        writer.write_tagged_string("BPAT", &self.blink_pattern);
        writer.write_tagged_string("IMG1", &self.off_image);
        writer.write_tagged_u32("BINT", self.blink_interval);
        writer.write_tagged_f32("BWTH", self.intensity);
        writer.write_tagged_f32("TRMS", self.transmission_scale);

        writer.write_tagged_string("SURF", &self.surface);
        writer.write_tagged_wide_string("NAME", &self.name);

        writer.write_tagged_bool("BGLS", self.is_backglass);
        writer.write_tagged_f32("LIDB", self.depth_bias);
        writer.write_tagged_f32("FASP", self.fade_speed_up);
        writer.write_tagged_f32("FASD", self.fade_speed_down);
        writer.write_tagged_bool("BULT", self.is_bulb_light);
        writer.write_tagged_bool("IMMO", self.is_image_mode);
        writer.write_tagged_bool("SHBM", self.show_bulb_mesh);
        if let Some(stbm) = self.has_static_bulb_mesh {
            writer.write_tagged_bool("STBM", stbm);
        }
        writer.write_tagged_bool("SHRB", self.show_reflection_on_ball);
        writer.write_tagged_f32("BMSC", self.mesh_radius);
        writer.write_tagged_f32("BMVA", self.bulb_modulate_vs_add);
        writer.write_tagged_f32("BHHI", self.bulb_halo_height);
        if let Some(shadows) = self.shadows {
            writer.write_tagged_u32("SHDW", shadows);
        }
        if let Some(fader) = self.fader {
            writer.write_tagged_u32("FADE", fader);
        }
        if let Some(visible) = self.visible {
            writer.write_tagged_bool("VSBL", visible);
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
        // many of these
        for point in &self.drag_points {
            writer.write_tagged("DPNT", point);
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
        // values not equal to the defaults
        let light = Light {
            center: Vertex2D::new(1.0, 2.0),
            height: Some(3.0),
            falloff_radius: 25.0,
            falloff_power: 3.0,
            status: 4,
            state: Some(5.0),
            color: Color::new_argb(0x123456),
            color2: Color::new_argb(0x654321),
            is_timer_enabled: true,
            timer_interval: 7,
            blink_pattern: "test pattern".to_string(),
            off_image: "test image".to_string(),
            blink_interval: 8,
            intensity: 9.0,
            transmission_scale: 10.0,
            surface: "test surface".to_string(),
            name: "test name".to_string(),
            is_backglass: false,
            depth_bias: 11.0,
            fade_speed_up: 12.0,
            fade_speed_down: 13.0,
            is_bulb_light: true,
            is_image_mode: true,
            show_bulb_mesh: false,
            has_static_bulb_mesh: Some(false),
            show_reflection_on_ball: false,
            mesh_radius: 14.0,
            bulb_modulate_vs_add: 15.0,
            bulb_halo_height: 16.0,
            shadows: Some(18),
            fader: Some(19),
            visible: Some(true),
            is_locked: false,
            editor_layer: 17,
            editor_layer_name: Some("test layer".to_string()),
            editor_layer_visibility: Some(true),
            drag_points: vec![DragPoint::default()],
        };
        let mut writer = BiffWriter::new();
        Light::biff_write(&light, &mut writer);
        let light_read = Light::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(light, light_read);
    }
}
