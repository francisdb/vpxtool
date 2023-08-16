use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct Gate {
    pub center: Vertex2D,            // 1 VCEN
    pub length: f32,                 // 2 LGTH
    pub height: f32,                 // 3 HGTH
    pub rotation: f32,               // 4 ROTA
    pub material: String,            // 5 MATR
    pub is_timer_enabled: bool,      // 6 TMON
    pub show_bracket: bool,          // 7 GSUP
    pub is_collidable: bool,         // 8 GCOL
    pub timer_interval: f32,         // 9 TMIN
    pub surface: String,             // 10 SURF
    pub elasticity: f32,             // 11 ELAS
    pub angle_max: f32,              // 12 GAMA
    pub angle_min: f32,              // 13 GAMI
    pub friction: f32,               // 14 GFRC
    pub damping: Option<f32>,        // 15 AFRC (added in 10.?)
    pub gravity_factor: Option<f32>, // 16 GGFC (added in 10.?)
    pub is_visible: bool,            // 17 GVSB
    pub name: String,                // 18 NAME
    pub two_way: bool,               // 19 TWWA
    pub is_reflection_enabled: bool, // 20 REEN
    pub gate_type: u32,              // 21 GATY

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,
}

impl BiffRead for Gate {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut length: f32 = 100.0;
        let mut height: f32 = 50.0;
        let mut rotation: f32 = -90.0;
        let mut material: String = Default::default();
        let mut is_timer_enabled: bool = false;
        let mut show_bracket: bool = true;
        let mut is_collidable: bool = true;
        let mut timer_interval: f32 = Default::default();
        let mut surface: String = Default::default();
        let mut elasticity: f32 = 0.3;
        let mut angle_max: f32 = std::f32::consts::PI / 2.0;
        let mut angle_min: f32 = Default::default();
        let mut friction: f32 = 0.02;
        let mut damping: Option<f32> = None; //0.985;
        let mut is_visible: bool = true;
        let mut name: String = Default::default();
        let mut two_way: bool = false;
        let mut is_reflection_enabled: bool = true;
        let mut gate_type: u32 = Default::default();
        let mut gravity_factor: Option<f32> = None; //0.25;

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
                "LGTH" => {
                    length = reader.get_f32();
                }
                "HGTH" => {
                    height = reader.get_f32();
                }
                "ROTA" => {
                    rotation = reader.get_f32();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "GSUP" => {
                    show_bracket = reader.get_bool();
                }
                "GCOL" => {
                    is_collidable = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_f32();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "ELAS" => {
                    elasticity = reader.get_f32();
                }
                "GAMA" => {
                    angle_max = reader.get_f32();
                }
                "GAMI" => {
                    angle_min = reader.get_f32();
                }
                "GFRC" => {
                    friction = reader.get_f32();
                }
                "AFRC" => {
                    damping = Some(reader.get_f32());
                }
                "GGFC" => {
                    gravity_factor = Some(reader.get_f32());
                }
                "GVSB" => {
                    is_visible = reader.get_bool();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "TWWA" => {
                    two_way = reader.get_bool();
                }
                "REEN" => {
                    is_reflection_enabled = reader.get_bool();
                }
                "GATY" => {
                    gate_type = reader.get_u32();
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
            length,
            height,
            rotation,
            material,
            is_timer_enabled,
            show_bracket,
            is_collidable,
            timer_interval,
            surface,
            elasticity,
            angle_max,
            angle_min,
            friction,
            damping,
            gravity_factor,
            is_visible,
            name,
            two_way,
            is_reflection_enabled,
            gate_type,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}

impl BiffWrite for Gate {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VCEN", &self.center);
        writer.write_tagged_f32("LGTH", self.length);
        writer.write_tagged_f32("HGTH", self.height);
        writer.write_tagged_f32("ROTA", self.rotation);
        writer.write_tagged_string("MATR", &self.material);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_bool("GSUP", self.show_bracket);
        writer.write_tagged_bool("GCOL", self.is_collidable);
        writer.write_tagged_f32("TMIN", self.timer_interval);
        writer.write_tagged_string("SURF", &self.surface);
        writer.write_tagged_f32("ELAS", self.elasticity);
        writer.write_tagged_f32("GAMA", self.angle_max);
        writer.write_tagged_f32("GAMI", self.angle_min);
        writer.write_tagged_f32("GFRC", self.friction);
        if let Some(damping) = self.damping {
            writer.write_tagged_f32("AFRC", damping);
        }
        if let Some(gravity_factor) = self.gravity_factor {
            writer.write_tagged_f32("GGFC", gravity_factor);
        }
        writer.write_tagged_bool("GVSB", self.is_visible);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_bool("TWWA", self.two_way);
        writer.write_tagged_bool("REEN", self.is_reflection_enabled);
        writer.write_tagged_u32("GATY", self.gate_type);

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
        let gate = Gate {
            center: Vertex2D::new(1.0, 2.0),
            length: 3.0,
            height: 4.0,
            rotation: 5.0,
            material: "material".to_string(),
            is_timer_enabled: true,
            show_bracket: false,
            is_collidable: false,
            timer_interval: 6.0,
            surface: "surface".to_string(),
            elasticity: 7.0,
            angle_max: 8.0,
            angle_min: 9.0,
            friction: 10.0,
            damping: Some(11.0),
            gravity_factor: Some(12.0),
            is_visible: false,
            name: "name".to_string(),
            two_way: true,
            is_reflection_enabled: false,
            gate_type: 13,
            is_locked: true,
            editor_layer: 14,
            editor_layer_name: Some("editor_layer_name".to_string()),
            editor_layer_visibility: Some(false),
        };
        let mut writer = BiffWriter::new();
        Gate::biff_write(&gate, &mut writer);
        let gate_read = Gate::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(gate, gate_read);
    }
}
