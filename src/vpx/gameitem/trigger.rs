use crate::vpx::biff::{self, BiffRead, BiffReader};

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
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,

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
        let mut editor_layer_name: String = Default::default();
        let mut editor_layer_visibility: bool = true;

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
                    editor_layer_name = reader.get_string();
                }
                "LVIS" => {
                    editor_layer_visibility = reader.get_bool();
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
