use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct Kicker {
    center: Vertex2D,
    radius: f32,
    is_timer_enabled: bool,
    timer_interval: u32,
    material: String,
    surface: String,
    is_enabled: bool,
    pub name: String,
    kicker_type: u32,
    scatter: f32,
    hit_accuracy: f32,
    hit_height: f32,
    orientation: f32,
    fall_through: bool,
    legacy_mode: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,
}

impl Kicker {
    pub const KICKER_TYPE_INVISIBLE: u32 = 0;
    pub const KICKER_TYPE_HOLE: u32 = 1;
    pub const KICKER_TYPE_CUP: u32 = 2;
    pub const KICKER_TYPE_HOLE_SIMPLE: u32 = 3;
    pub const KICKER_TYPE_WILLIAMS: u32 = 4;
    pub const KICKER_TYPE_GOTTLIEB: u32 = 5;
    pub const KICKER_TYPE_CUP2: u32 = 6;
}

impl BiffRead for Kicker {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut radius: f32 = 25.0;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = 0;
        let mut material = Default::default();
        let mut surface = Default::default();
        let mut is_enabled: bool = true;
        let mut name = Default::default();
        let mut kicker_type: u32 = Kicker::KICKER_TYPE_HOLE;
        let mut scatter: f32 = 0.0;
        let mut hit_accuracy: f32 = 0.7;
        let mut hit_height: f32 = 40.0;
        let mut orientation: f32 = 0.0;
        let mut fall_through: bool = false;
        let mut legacy_mode: bool = true;

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
                "RADI" => {
                    radius = reader.get_f32();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "EBLD" => {
                    is_enabled = reader.get_bool();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "TYPE" => {
                    kicker_type = reader.get_u32();
                }
                "KSCT" => {
                    scatter = reader.get_f32();
                }
                "KHAC" => {
                    hit_accuracy = reader.get_f32();
                }
                "KHHI" => {
                    hit_height = reader.get_f32();
                }
                "KORI" => {
                    orientation = reader.get_f32();
                }
                "FATH" => {
                    fall_through = reader.get_bool();
                }
                "LEMO" => {
                    legacy_mode = reader.get_bool();
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
            radius,
            is_timer_enabled,
            timer_interval,
            material,
            surface,
            is_enabled,
            name,
            kicker_type,
            scatter,
            hit_accuracy,
            hit_height,
            orientation,
            fall_through,
            legacy_mode,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}

impl BiffWrite for Kicker {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VCEN", &self.center);
        writer.write_tagged_f32("RADI", self.radius);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_u32("TMIN", self.timer_interval);
        writer.write_tagged_string("MATR", &self.material);
        writer.write_tagged_string("SURF", &self.surface);
        writer.write_tagged_bool("EBLD", self.is_enabled);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_u32("TYPE", self.kicker_type);
        writer.write_tagged_f32("KSCT", self.scatter);
        writer.write_tagged_f32("KHAC", self.hit_accuracy);
        writer.write_tagged_f32("KHHI", self.hit_height);
        writer.write_tagged_f32("KORI", self.orientation);
        writer.write_tagged_bool("FATH", self.fall_through);
        writer.write_tagged_bool("LEMO", self.legacy_mode);
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
        // values not equal to the defaults
        let kicker = Kicker {
            center: Vertex2D::new(1.0, 2.0),
            radius: 3.0,
            is_timer_enabled: true,
            timer_interval: 4,
            material: "material".to_string(),
            surface: "surface".to_string(),
            is_enabled: false,
            name: "name".to_string(),
            kicker_type: 5,
            scatter: 6.0,
            hit_accuracy: 7.0,
            hit_height: 8.0,
            orientation: 9.0,
            fall_through: true,
            legacy_mode: false,
            is_locked: true,
            editor_layer: 10,
            editor_layer_name: Some("editor_layer_name".to_string()),
            editor_layer_visibility: Some(false),
        };
        let mut writer = BiffWriter::new();
        Kicker::biff_write(&kicker, &mut writer);
        let kicker_read = Kicker::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(kicker, kicker_read);
    }
}
