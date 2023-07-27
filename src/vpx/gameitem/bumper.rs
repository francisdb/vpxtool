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
    scatter: f32,
    height_scale: f32,
    ring_speed: f32,
    orientation: f32,
    ring_drop_offset: f32,
    cap_material: String,
    base_material: String,
    socket_material: String,
    ring_material: String,
    surface: String,
    name: String,
    is_cap_visible: bool,
    is_base_visible: bool,
    is_ring_visible: bool,
    is_socket_visible: bool,
    hit_event: bool,
    is_collidable: bool,
    is_reflection_enabled: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
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
        let mut scatter: f32 = 0.0;
        let mut height_scale: f32 = 90.0;
        let mut ring_speed: f32 = 0.5;
        let mut orientation: f32 = 0.0;
        let mut ring_drop_offset: f32 = 0.0;
        let mut cap_material: String = Default::default();
        let mut base_material: String = Default::default();
        let mut socket_material: String = Default::default();
        let mut ring_material: String = Default::default();
        let mut surface: String = Default::default();
        let mut name = Default::default();
        let mut is_cap_visible: bool = true;
        let mut is_base_visible: bool = true;
        let mut is_ring_visible: bool = true;
        let mut is_socket_visible: bool = true;
        let mut hit_event: bool = true;
        let mut is_collidable: bool = true;
        let mut is_reflection_enabled: bool = true;

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
                    scatter = reader.get_f32();
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
                    ring_drop_offset = reader.get_f32();
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
                    ring_material = reader.get_string();
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
                    is_ring_visible = reader.get_bool();
                }
                "SKVS" => {
                    is_socket_visible = reader.get_bool();
                }
                "HAHE" => {
                    hit_event = reader.get_bool();
                }
                "COLI" => {
                    is_collidable = reader.get_bool();
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
    fn biff_write(item: &Self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VCEN", &item.center);
        writer.write_tagged_f32("RADI", item.radius);
        writer.write_tagged_bool("TMON", item.is_timer_enabled);
        writer.write_tagged_i32("TMIN", item.timer_interval);
        writer.write_tagged_f32("THRS", item.threshold);
        writer.write_tagged_f32("FORC", item.force);
        writer.write_tagged_f32("BSCT", item.scatter);
        writer.write_tagged_f32("HISC", item.height_scale);
        writer.write_tagged_f32("RISP", item.ring_speed);
        writer.write_tagged_f32("ORIN", item.orientation);
        writer.write_tagged_f32("RDLI", item.ring_drop_offset);
        writer.write_tagged_string("MATR", &item.cap_material);
        writer.write_tagged_string("BAMA", &item.base_material);
        writer.write_tagged_string("SKMA", &item.socket_material);
        writer.write_tagged_string("RIMA", &item.ring_material);
        writer.write_tagged_string("SURF", &item.surface);
        writer.write_tagged_wide_string("NAME", &item.name);
        writer.write_tagged_bool("CAVI", item.is_cap_visible);
        writer.write_tagged_bool("BSVS", item.is_base_visible);
        writer.write_tagged_bool("RIVS", item.is_ring_visible);
        writer.write_tagged_bool("SKVS", item.is_socket_visible);
        writer.write_tagged_bool("HAHE", item.hit_event);
        writer.write_tagged_bool("COLI", item.is_collidable);
        writer.write_tagged_bool("REEN", item.is_reflection_enabled);
        // shared
        writer.write_tagged_bool("LOCK", item.is_locked);
        writer.write_tagged_u32("LAYR", item.editor_layer);
        writer.write_tagged_string("LANR", &item.editor_layer_name);
        writer.write_tagged_bool("LVIS", item.editor_layer_visibility);
        
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
            scatter: 0.0,
            height_scale: 90.0,
            ring_speed: 0.5,
            orientation: 0.0,
            ring_drop_offset: 0.0,
            cap_material: "ctest cap material".to_string(),
            base_material: "test base material".to_string(),
            socket_material: "test socket material".to_string(),
            ring_material: "test ring material".to_string(),
            surface: "test surface".to_string(),
            name: "test bumper".to_string(),
            is_cap_visible: true,
            is_base_visible: true,
            is_ring_visible: true,
            is_socket_visible: true,
            hit_event: true,
            is_collidable: true,
            is_reflection_enabled: true,
            is_locked: true,
            editor_layer: 5,
            editor_layer_name: "layer".to_string(),
            editor_layer_visibility: true,
        };
        let mut writer = BiffWriter::new();
        Bumper::biff_write(&bumper, &mut writer);
        let bumper_read = Bumper::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(bumper, bumper_read);
    }
}
