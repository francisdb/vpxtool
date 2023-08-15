use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

use super::{vertex2d::Vertex2D, GameItem};

#[derive(Debug, PartialEq)]
pub struct Bumper {
    center: Vertex2D,
    radius: f32,
    is_timer_enabled: bool,
    timer_interval: i32,
    threshold: f32,
    force: f32,
    scatter: Option<f32>, // BSCT (added in ?)
    height_scale: f32,
    ring_speed: f32,
    orientation: f32,
    ring_drop_offset: Option<f32>, // RDLI (added in ?)
    cap_material: String,
    base_material: String,
    socket_material: String,
    ring_material: Option<String>, // RIMA (added in ?)
    surface: String,
    name: String,
    is_cap_visible: bool,
    is_base_visible: bool,
    is_ring_visible: Option<bool>,   // RIVS (added in ?)
    is_socket_visible: Option<bool>, // SKVS (added in ?)
    hit_event: Option<bool>,         // HAHE (added in ?)
    is_collidable: Option<bool>,     // COLI (added in ?)
    is_reflection_enabled: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,
}
impl GameItem for Bumper {
    fn name(&self) -> &str {
        &self.name
    }
}

impl BiffRead for Bumper {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut radius: f32 = 45.0;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: i32 = 0;
        let mut threshold: f32 = 1.0;
        let mut force: f32 = 15.0;
        let mut scatter: Option<f32> = None; //0.0;
        let mut height_scale: f32 = 90.0;
        let mut ring_speed: f32 = 0.5;
        let mut orientation: f32 = 0.0;
        let mut ring_drop_offset: Option<f32> = None; //0.0;
        let mut cap_material: String = Default::default();
        let mut base_material: String = Default::default();
        let mut socket_material: String = Default::default();
        let mut ring_material: Option<String> = None;
        let mut surface: String = Default::default();
        let mut name = Default::default();
        let mut is_cap_visible: bool = true;
        let mut is_base_visible: bool = true;
        let mut is_ring_visible: Option<bool> = None; //true;
        let mut is_socket_visible: Option<bool> = None; //true;
        let mut hit_event: Option<bool> = None; //true;
        let mut is_collidable: Option<bool> = None; //true;
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
                "RADI" => {
                    radius = reader.get_f32();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_i32();
                }
                "THRS" => {
                    threshold = reader.get_f32();
                }
                "FORC" => {
                    force = reader.get_f32();
                }
                "BSCT" => {
                    scatter = Some(reader.get_f32());
                }
                "HISC" => {
                    height_scale = reader.get_f32();
                }
                "RISP" => {
                    ring_speed = reader.get_f32();
                }
                "ORIN" => {
                    orientation = reader.get_f32();
                }
                "RDLI" => {
                    ring_drop_offset = Some(reader.get_f32());
                }
                "MATR" => {
                    cap_material = reader.get_string();
                }
                "BAMA" => {
                    base_material = reader.get_string();
                }
                "SKMA" => {
                    socket_material = reader.get_string();
                }
                "RIMA" => {
                    ring_material = Some(reader.get_string());
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "CAVI" => {
                    is_cap_visible = reader.get_bool();
                }
                "BSVS" => {
                    is_base_visible = reader.get_bool();
                }
                "RIVS" => {
                    is_ring_visible = Some(reader.get_bool());
                }
                "SKVS" => {
                    is_socket_visible = Some(reader.get_bool());
                }
                "HAHE" => {
                    hit_event = Some(reader.get_bool());
                }
                "COLI" => {
                    is_collidable = Some(reader.get_bool());
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
            radius,
            is_timer_enabled,
            timer_interval,
            threshold,
            force,
            scatter,
            height_scale,
            ring_speed,
            orientation,
            ring_drop_offset,
            cap_material,
            base_material,
            socket_material,
            ring_material,
            surface,
            name,
            is_cap_visible,
            is_base_visible,
            is_ring_visible,
            is_socket_visible,
            hit_event,
            is_collidable,
            is_reflection_enabled,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}

impl BiffWrite for Bumper {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VCEN", &self.center);
        writer.write_tagged_f32("RADI", self.radius);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_i32("TMIN", self.timer_interval);
        writer.write_tagged_f32("THRS", self.threshold);
        writer.write_tagged_f32("FORC", self.force);
        if let Some(scatter) = self.scatter {
            writer.write_tagged_f32("BSCT", scatter);
        }
        writer.write_tagged_f32("HISC", self.height_scale);
        writer.write_tagged_f32("RISP", self.ring_speed);
        writer.write_tagged_f32("ORIN", self.orientation);
        if let Some(ring_drop_offset) = self.ring_drop_offset {
            writer.write_tagged_f32("RDLI", ring_drop_offset);
        }
        writer.write_tagged_string("MATR", &self.cap_material);
        writer.write_tagged_string("BAMA", &self.base_material);
        writer.write_tagged_string("SKMA", &self.socket_material);
        if let Some(ring_material) = &self.ring_material {
            writer.write_tagged_string("RIMA", ring_material);
        }
        writer.write_tagged_string("SURF", &self.surface);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_bool("CAVI", self.is_cap_visible);
        writer.write_tagged_bool("BSVS", self.is_base_visible);
        if let Some(is_ring_visible) = self.is_ring_visible {
            writer.write_tagged_bool("RIVS", is_ring_visible);
        }
        if let Some(is_socket_visible) = self.is_socket_visible {
            writer.write_tagged_bool("SKVS", is_socket_visible);
        }
        if let Some(hit_event) = self.hit_event {
            writer.write_tagged_bool("HAHE", hit_event);
        }
        if let Some(is_collidable) = self.is_collidable {
            writer.write_tagged_bool("COLI", is_collidable);
        }
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
        // random data not same as default data above
        let bumper = Bumper {
            center: Vertex2D::new(1.0, 2.0),
            radius: 45.0,
            is_timer_enabled: true,
            timer_interval: 3,
            threshold: 1.0,
            force: 15.0,
            scatter: Some(0.0),
            height_scale: 90.0,
            ring_speed: 0.5,
            orientation: 0.0,
            ring_drop_offset: Some(0.0),
            cap_material: "ctest cap material".to_string(),
            base_material: "test base material".to_string(),
            socket_material: "test socket material".to_string(),
            ring_material: Some("test ring material".to_string()),
            surface: "test surface".to_string(),
            name: "test bumper".to_string(),
            is_cap_visible: true,
            is_base_visible: true,
            is_ring_visible: Some(true),
            is_socket_visible: Some(true),
            hit_event: Some(true),
            is_collidable: Some(true),
            is_reflection_enabled: true,
            is_locked: true,
            editor_layer: 5,
            editor_layer_name: Some("layer".to_string()),
            editor_layer_visibility: Some(true),
        };
        let mut writer = BiffWriter::new();
        Bumper::biff_write(&bumper, &mut writer);
        let bumper_read = Bumper::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(bumper, bumper_read);
    }
}
