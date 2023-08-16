use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

use super::dragpoint::DragPoint;

#[derive(Debug, PartialEq)]
pub struct Rubber {
    pub height: f32,
    pub hit_height: Option<f32>, // HTHI (added in 10.?)
    pub thickness: i32,
    pub hit_event: bool,
    pub material: String,
    pub is_timer_enabled: bool,
    pub timer_interval: i32,
    pub name: String,
    pub image: String,
    pub elasticity: f32,
    pub elasticity_falloff: f32,
    pub friction: f32,
    pub scatter: f32,
    pub is_collidable: bool,
    pub is_visible: bool,
    pub static_rendering: bool,
    pub show_in_editor: bool,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub is_reflection_enabled: bool,
    pub physics_material: Option<String>, // MAPH (added in 10.?)
    pub overwrite_physics: Option<bool>,  // OVPH (added in 10.?)

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,

    points: Vec<DragPoint>,
}

impl BiffRead for Rubber {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut height: f32 = 25.0;
        let mut hit_height: Option<f32> = None; //25.0;
        let mut thickness: i32 = 8;
        let mut hit_event: bool = false;
        let mut material: String = Default::default();
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: i32 = Default::default();
        let mut name: String = Default::default();
        let mut image: String = Default::default();
        let mut elasticity: f32 = Default::default();
        let mut elasticity_falloff: f32 = Default::default();
        let mut friction: f32 = Default::default();
        let mut scatter: f32 = Default::default();
        let mut is_collidable: bool = true;
        let mut is_visible: bool = true;
        let mut static_rendering: bool = true;
        let mut show_in_editor: bool = true;
        let mut rot_x: f32 = 0.0;
        let mut rot_y: f32 = 0.0;
        let mut rot_z: f32 = 0.0;
        let mut is_reflection_enabled: bool = true;
        let mut physics_material: Option<String> = None;
        let mut overwrite_physics: Option<bool> = None; //false;

        // these are shared between all items
        let mut is_locked: bool = false;
        let mut editor_layer: u32 = Default::default();
        let mut editor_layer_name: Option<String> = None;
        let mut editor_layer_visibility: Option<bool> = None;

        let mut points: Vec<DragPoint> = Default::default();

        loop {
            reader.next(biff::WARN);
            if reader.is_eof() {
                break;
            }
            let tag = reader.tag();
            let tag_str = tag.as_str();
            match tag_str {
                "HTTP" => {
                    height = reader.get_f32();
                }
                "HTHI" => {
                    hit_height = Some(reader.get_f32());
                }
                "WDTP" => {
                    thickness = reader.get_i32();
                }
                "HTEV" => {
                    hit_event = reader.get_bool();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_i32();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "IMAG" => {
                    image = reader.get_string();
                }
                "ELAS" => {
                    elasticity = reader.get_f32();
                }
                "ELFO" => {
                    elasticity_falloff = reader.get_f32();
                }
                "RFCT" => {
                    friction = reader.get_f32();
                }
                "RSCT" => {
                    scatter = reader.get_f32();
                }
                "CLDR" => {
                    is_collidable = reader.get_bool();
                }
                "RVIS" => {
                    is_visible = reader.get_bool();
                }
                "ESTR" => {
                    static_rendering = reader.get_bool();
                }
                "ESIE" => {
                    show_in_editor = reader.get_bool();
                }
                "ROTX" => {
                    rot_x = reader.get_f32();
                }
                "ROTY" => {
                    rot_y = reader.get_f32();
                }
                "ROTZ" => {
                    rot_z = reader.get_f32();
                }
                "REEN" => {
                    is_reflection_enabled = reader.get_bool();
                }
                "MAPH" => {
                    physics_material = Some(reader.get_string());
                }
                "OVPH" => {
                    overwrite_physics = Some(reader.get_bool());
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

                "PNTS" => {
                    // this is just a tag with no data
                }
                "DPNT" => {
                    let point = DragPoint::biff_read(reader);
                    points.push(point);
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
        Rubber {
            height,
            hit_height,
            thickness,
            hit_event,
            material,
            is_timer_enabled,
            timer_interval,
            name,
            image,
            elasticity,
            elasticity_falloff,
            friction,
            scatter,
            is_collidable,
            is_visible,
            static_rendering,
            show_in_editor,
            rot_x,
            rot_y,
            rot_z,
            is_reflection_enabled,
            physics_material,
            overwrite_physics,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
            points,
        }
    }
}

impl BiffWrite for Rubber {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged_f32("HTTP", self.height);
        if let Some(hthi) = self.hit_height {
            writer.write_tagged_f32("HTHI", hthi);
        }
        writer.write_tagged_i32("WDTP", self.thickness);
        writer.write_tagged_bool("HTEV", self.hit_event);
        writer.write_tagged_string("MATR", &self.material);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_i32("TMIN", self.timer_interval);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_string("IMAG", &self.image);
        writer.write_tagged_f32("ELAS", self.elasticity);
        writer.write_tagged_f32("ELFO", self.elasticity_falloff);
        writer.write_tagged_f32("RFCT", self.friction);
        writer.write_tagged_f32("RSCT", self.scatter);
        writer.write_tagged_bool("CLDR", self.is_collidable);
        writer.write_tagged_bool("RVIS", self.is_visible);
        writer.write_tagged_bool("ESTR", self.static_rendering);
        writer.write_tagged_bool("ESIE", self.show_in_editor);
        writer.write_tagged_f32("ROTX", self.rot_x);
        writer.write_tagged_f32("ROTY", self.rot_y);
        writer.write_tagged_f32("ROTZ", self.rot_z);
        writer.write_tagged_bool("REEN", self.is_reflection_enabled);
        if let Some(physics_material) = &self.physics_material {
            writer.write_tagged_string("MAPH", physics_material);
        }
        if let Some(overwrite_physics) = self.overwrite_physics {
            writer.write_tagged_bool("OVPH", overwrite_physics);
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

        writer.write_marker_tag("PNTS");

        for point in &self.points {
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
    use rand::Rng;

    #[test]
    fn test_write_read() {
        let mut rng = rand::thread_rng();
        // values not equal to the defaults
        let rubber: Rubber = Rubber {
            height: 1.0,
            hit_height: Some(2.0),
            thickness: 3,
            hit_event: rng.gen(),
            material: "material".to_string(),
            is_timer_enabled: rng.gen(),
            timer_interval: 4,
            name: "name".to_string(),
            image: "image".to_string(),
            elasticity: 5.0,
            elasticity_falloff: 6.0,
            friction: 7.0,
            scatter: 8.0,
            is_collidable: rng.gen(),
            is_visible: rng.gen(),
            static_rendering: rng.gen(),
            show_in_editor: rng.gen(),
            rot_x: 9.0,
            rot_y: 10.0,
            rot_z: 11.0,
            is_reflection_enabled: rng.gen(),
            physics_material: Some("physics_material".to_string()),
            overwrite_physics: rng.gen(),
            is_locked: rng.gen(),
            editor_layer: 12,
            editor_layer_name: Some("editor_layer_name".to_string()),
            editor_layer_visibility: rng.gen(),
            points: vec![DragPoint::default()],
        };
        let mut writer = BiffWriter::new();
        Rubber::biff_write(&rubber, &mut writer);
        let rubber_read = Rubber::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(rubber, rubber_read);
    }
}
