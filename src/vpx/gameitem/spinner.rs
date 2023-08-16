use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct Spinner {
    center: Vertex2D,
    rotation: f32,
    is_timer_enabled: bool,
    timer_interval: u32,
    height: f32,
    length: f32,
    damping: f32,
    angle_max: f32,
    angle_min: f32,
    elasticity: f32,
    is_visible: bool,
    show_bracket: bool,
    material: String,
    image: String,
    surface: String,
    pub name: String,
    pub is_reflection_enabled: Option<bool>, // added in ?

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,
}

impl BiffRead for Spinner {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut rotation: f32 = 0.0;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = 0;
        let mut height: f32 = 60.0;
        let mut length: f32 = 80.0;
        let mut damping: f32 = 0.9879;
        let mut angle_max: f32 = 0.0;
        let mut angle_min: f32 = 0.0;
        let mut elasticity: f32 = 0.3;
        let mut is_visible: bool = true;
        let mut show_bracket: bool = true;
        let mut material = Default::default();
        let mut image = Default::default();
        let mut surface = Default::default();
        let mut name = Default::default();
        let mut is_reflection_enabled: Option<bool> = None;

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
                "ROTA" => {
                    rotation = reader.get_f32();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "HIGH" => {
                    height = reader.get_f32();
                }
                "LGTH" => {
                    length = reader.get_f32();
                }
                "AFRC" => {
                    damping = reader.get_f32();
                }
                "SMAX" => {
                    angle_max = reader.get_f32();
                }
                "SMIN" => {
                    angle_min = reader.get_f32();
                }
                "SELA" => {
                    elasticity = reader.get_f32();
                }
                "SVIS" => {
                    is_visible = reader.get_bool();
                }
                "SSUP" => {
                    show_bracket = reader.get_bool();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "IMGF" => {
                    image = reader.get_string();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "REEN" => {
                    is_reflection_enabled = Some(reader.get_bool());
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
            rotation,
            is_timer_enabled,
            timer_interval,
            height,
            length,
            damping,
            angle_max,
            angle_min,
            elasticity,
            is_visible,
            show_bracket,
            material,
            image,
            surface,
            name,
            is_reflection_enabled,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}

impl BiffWrite for Spinner {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VCEN", &self.center);
        writer.write_tagged_f32("ROTA", self.rotation);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_u32("TMIN", self.timer_interval);
        writer.write_tagged_f32("HIGH", self.height);
        writer.write_tagged_f32("LGTH", self.length);
        writer.write_tagged_f32("AFRC", self.damping);
        writer.write_tagged_f32("SMAX", self.angle_max);
        writer.write_tagged_f32("SMIN", self.angle_min);
        writer.write_tagged_f32("SELA", self.elasticity);
        writer.write_tagged_bool("SVIS", self.is_visible);
        writer.write_tagged_bool("SSUP", self.show_bracket);
        writer.write_tagged_string("MATR", &self.material);
        writer.write_tagged_string("IMGF", &self.image);
        writer.write_tagged_string("SURF", &self.surface);
        writer.write_tagged_wide_string("NAME", &self.name);
        if let Some(is_reflection_enabled) = self.is_reflection_enabled {
            writer.write_tagged_bool("REEN", is_reflection_enabled);
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
        let spinner = Spinner {
            center: Vertex2D::new(rng.gen(), rng.gen()),
            rotation: rng.gen(),
            is_timer_enabled: rng.gen(),
            timer_interval: rng.gen(),
            height: rng.gen(),
            length: rng.gen(),
            damping: rng.gen(),
            angle_max: rng.gen(),
            angle_min: rng.gen(),
            elasticity: rng.gen(),
            is_visible: rng.gen(),
            show_bracket: rng.gen(),
            material: "test material".to_string(),
            image: "test image".to_string(),
            surface: "test surface".to_string(),
            name: "test name".to_string(),
            is_reflection_enabled: rng.gen(),
            is_locked: rng.gen(),
            editor_layer: rng.gen(),
            editor_layer_name: Some("test layer name".to_string()),
            editor_layer_visibility: rng.gen(),
        };
        let mut writer = BiffWriter::new();
        Spinner::biff_write(&spinner, &mut writer);
        let spinner_read = Spinner::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(spinner, spinner_read);
    }
}
