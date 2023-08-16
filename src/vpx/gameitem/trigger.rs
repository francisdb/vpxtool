use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

use super::{dragpoint::DragPoint, vertex2d::Vertex2D, TRIGGER_SHAPE_WIRE_A};

#[derive(Debug, PartialEq)]
pub struct Trigger {
    pub center: Vertex2D,
    pub radius: f32,
    pub rotation: f32,
    pub wire_thickness: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub is_timer_enabled: bool,
    pub timer_interval: i32,
    pub material: String,
    pub surface: String,
    pub is_visible: bool,
    pub is_enabled: bool,
    pub hit_height: f32,
    pub name: String,
    // [BiffInt("SHAP", Pos = 15)]
    // public int Shape = TriggerShape.TriggerWireA;

    // [BiffFloat("ANSP", Pos = 16)]
    // public float AnimSpeed = 1f;

    // [BiffBool("REEN", Pos = 17)]
    // public bool IsReflectionEnabled = true;
    pub shape: u32,
    pub anim_speed: f32,
    pub is_reflection_enabled: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,

    drag_points: Vec<DragPoint>,
}

impl BiffRead for Trigger {
    fn biff_read(reader: &mut BiffReader<'_>) -> Trigger {
        let mut center: Vertex2D = Default::default();
        let mut radius: f32 = 25.0;
        let mut rotation: f32 = Default::default();
        let mut wire_thickness: f32 = Default::default();
        let mut scale_x: f32 = Default::default();
        let mut scale_y: f32 = Default::default();
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: i32 = Default::default();
        let mut material: String = Default::default();
        let mut surface: String = Default::default();
        let mut is_visible: bool = true;
        let mut is_enabled: bool = true;
        let mut hit_height: f32 = 50.0;
        let mut name = Default::default();
        let mut shape: u32 = TRIGGER_SHAPE_WIRE_A;
        let mut anim_speed: f32 = Default::default();
        let mut is_reflection_enabled: bool = true;

        // these are shared between all items
        let mut is_locked: bool = false;
        let mut editor_layer: u32 = Default::default();
        let mut editor_layer_name: Option<String> = None;
        let mut editor_layer_visibility: Option<bool> = None;

        let mut drag_points: Vec<DragPoint> = Default::default();

        loop {
            reader.next(biff::WARN);
            if reader.is_eof() {
                break;
            }
            let tag = reader.tag();
            let tag_str = tag.as_str();
            match tag_str {
                // tag_str: SHAP
                // tag_str: ANSP
                // tag_str: REEN
                "VCEN" => {
                    center = Vertex2D::biff_read(reader);
                }
                "RADI" => {
                    radius = reader.get_f32();
                }
                "ROTA" => {
                    rotation = reader.get_f32();
                }
                "WITI" => {
                    wire_thickness = reader.get_f32();
                }
                "SCAX" => {
                    scale_x = reader.get_f32();
                }
                "SCAY" => {
                    scale_y = reader.get_f32();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_i32();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "SURF" => {
                    surface = reader.get_string();
                }
                "VSBL" => {
                    is_visible = reader.get_bool();
                }
                "EBLD" => {
                    is_enabled = reader.get_bool();
                }
                "THOT" => {
                    hit_height = reader.get_f32();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "SHAP" => {
                    shape = reader.get_u32();
                }
                "ANSP" => {
                    anim_speed = reader.get_f32();
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
                "DPNT" => {
                    let point = DragPoint::biff_read(reader);
                    drag_points.push(point);
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
        Trigger {
            center,
            radius,
            rotation,
            wire_thickness,
            scale_x,
            scale_y,
            is_timer_enabled,
            timer_interval,
            material,
            surface,
            is_visible,
            is_enabled,
            hit_height,
            name,
            shape,
            anim_speed,
            is_reflection_enabled,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
            drag_points,
        }
    }
}

impl BiffWrite for Trigger {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VCEN", &self.center);
        writer.write_tagged_f32("RADI", self.radius);
        writer.write_tagged_f32("ROTA", self.rotation);
        writer.write_tagged_f32("WITI", self.wire_thickness);
        writer.write_tagged_f32("SCAX", self.scale_x);
        writer.write_tagged_f32("SCAY", self.scale_y);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_i32("TMIN", self.timer_interval);
        writer.write_tagged_string("SURF", &self.surface);
        writer.write_tagged_string("MATR", &self.material);
        writer.write_tagged_bool("EBLD", self.is_enabled);
        writer.write_tagged_bool("VSBL", self.is_visible);
        writer.write_tagged_f32("THOT", self.hit_height);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_u32("SHAP", self.shape);
        writer.write_tagged_f32("ANSP", self.anim_speed);
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
        let trigger = Trigger {
            center: Vertex2D::new(1.0, 2.0),
            radius: 25.0,
            rotation: 3.0,
            wire_thickness: 4.0,
            scale_x: 5.0,
            scale_y: 6.0,
            is_timer_enabled: true,
            timer_interval: 7,
            material: "test material".to_string(),
            surface: "test surface".to_string(),
            is_visible: false,
            is_enabled: false,
            hit_height: 8.0,
            name: "test name".to_string(),
            shape: 9,
            anim_speed: 10.0,
            is_reflection_enabled: false,
            is_locked: true,
            editor_layer: 11,
            editor_layer_name: Some("test layer name".to_string()),
            editor_layer_visibility: Some(false),
            drag_points: vec![DragPoint::default()],
        };
        let mut writer = BiffWriter::new();
        Trigger::biff_write(&trigger, &mut writer);
        let trigger_read = Trigger::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(trigger, trigger_read);
    }
}
