use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::dragpoint::DragPoint;

/**
 * Surface
 */
#[derive(Debug, PartialEq)]
pub struct Wall {
    pub hit_event: bool,
    pub is_droppable: bool,
    pub is_flipbook: bool,
    pub is_bottom_solid: bool,
    pub is_collidable: bool,
    pub is_timer_enabled: bool,
    pub timer_interval: u32,
    pub threshold: f32,
    pub image: String,
    pub side_image: String,
    pub side_material: String,
    pub top_material: String,
    pub slingshot_material: String,
    pub height_bottom: f32,
    pub height_top: f32,
    pub name: String,
    pub display_texture: bool,
    pub slingshot_force: f32,
    pub slingshot_threshold: f32,
    pub elasticity: f32,
    pub elasticity_falloff: f32,
    pub friction: f32,
    pub scatter: f32,
    pub is_top_bottom_visible: bool,
    pub slingshot_animation: bool,
    pub is_side_visible: bool,
    pub disable_lighting_top: f32,
    pub disable_lighting_below: f32,
    pub is_reflection_enabled: bool,
    pub physics_material: String,
    pub overwrite_physics: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,

    drag_points: Vec<DragPoint>,
}

impl BiffRead for Wall {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut hit_event: bool = false;
        let mut is_droppable: bool = false;
        let mut is_flipbook: bool = false;
        let mut is_bottom_solid: bool = false;
        let mut is_collidable: bool = true;
        let mut threshold: f32 = 2.0;
        let mut image: String = Default::default();
        let mut side_image: String = Default::default();
        let mut side_material: String = Default::default();
        let mut top_material: String = Default::default();
        let mut slingshot_material: String = Default::default();
        let mut height_bottom: f32 = Default::default();
        let mut height_top: f32 = 50.0;
        let mut name = Default::default();
        let mut display_texture: bool = false;
        let mut slingshot_force: f32 = 80.0;
        let mut slingshot_threshold: f32 = 0.0;
        let mut slingshot_animation: bool = true;
        let mut elasticity: f32 = 0.3;
        let mut elasticity_falloff: f32 = Default::default();
        let mut friction: f32 = 0.3;
        let mut scatter: f32 = Default::default();
        let mut is_top_bottom_visible: bool = true;
        let mut overwrite_physics: bool = true;
        let mut disable_lighting_top: f32 = Default::default();
        let mut disable_lighting_below: f32 = Default::default();
        let mut is_side_visible: bool = true;
        let mut is_reflection_enabled: bool = true;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = Default::default();
        let mut physics_material: String = Default::default();

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
                "HTEV" => {
                    hit_event = reader.get_bool();
                }
                "DROP" => {
                    is_droppable = reader.get_bool();
                }
                "FLIP" => {
                    is_flipbook = reader.get_bool();
                }
                "BOTS" => {
                    is_bottom_solid = reader.get_bool();
                }
                "COLL" => {
                    is_collidable = reader.get_bool();
                }
                "THRS" => {
                    threshold = reader.get_f32();
                }
                "IMGF" => {
                    image = reader.get_string();
                }
                "IMGS" => {
                    side_image = reader.get_string();
                }
                "MATR" => {
                    side_material = reader.get_string();
                }
                "MATP" => {
                    top_material = reader.get_string();
                }
                "MATL" => {
                    slingshot_material = reader.get_string();
                }
                "HIBO" => {
                    height_bottom = reader.get_f32();
                }
                "HITO" => {
                    height_top = reader.get_f32();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "DTEX" => {
                    display_texture = reader.get_bool();
                }
                "SLFO" => {
                    slingshot_force = reader.get_f32();
                }
                "SLTH" => {
                    slingshot_threshold = reader.get_f32();
                }
                "SLAN" => {
                    slingshot_animation = reader.get_bool();
                }
                "ELAS" => {
                    elasticity = reader.get_f32();
                }
                "ELFO" => {
                    elasticity_falloff = reader.get_f32();
                }
                "FRIC" => {
                    friction = reader.get_f32();
                }
                "SCAT" => {
                    scatter = reader.get_f32();
                }
                "TBVI" => {
                    is_top_bottom_visible = reader.get_bool();
                }
                "OVPH" => {
                    overwrite_physics = reader.get_bool();
                }
                "DLTO" => {
                    disable_lighting_top = reader.get_f32();
                }
                "DLBE" => {
                    disable_lighting_below = reader.get_f32();
                }
                "SIVI" => {
                    is_side_visible = reader.get_bool();
                }
                "REFL" => {
                    is_reflection_enabled = reader.get_bool();
                }
                "TMRN" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "PMAT" => {
                    physics_material = reader.get_string();
                }
                "ISBS" => {
                    is_bottom_solid = reader.get_bool();
                }
                "CLDW" => {
                    is_collidable = reader.get_bool();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "VSBL" => {
                    is_top_bottom_visible = reader.get_bool();
                }
                "SLGA" => {
                    slingshot_animation = reader.get_bool();
                }
                "SVBL" => {
                    is_side_visible = reader.get_bool();
                }
                "DILI" => {
                    disable_lighting_top = reader.get_f32();
                }
                "DILB" => {
                    disable_lighting_below = reader.get_f32();
                }
                "MAPH" => {
                    physics_material = reader.get_string();
                }
                "REEN" => {
                    is_reflection_enabled = reader.get_bool();
                }
                "IMAG" => {
                    image = reader.get_string();
                }
                "SIMG" => {
                    side_image = reader.get_string();
                }
                "SIMA" => {
                    side_material = reader.get_string();
                }
                "TOMA" => {
                    top_material = reader.get_string();
                }
                "SLMA" => {
                    slingshot_material = reader.get_string();
                }
                "HTBT" => {
                    height_bottom = reader.get_f32();
                }
                "HTTP" => {
                    height_top = reader.get_f32();
                }
                "DSPT" => {
                    display_texture = reader.get_bool();
                }
                "SLGF" => {
                    slingshot_force = reader.get_f32();
                }
                "WFCT" => {
                    slingshot_threshold = reader.get_f32();
                }
                "WSCT" => {
                    slingshot_animation = reader.get_bool();
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
                "PNTS" => {
                    // this is just a tag with no data
                }
                "DPNT" => {
                    // many of these
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
        Wall {
            hit_event,
            is_droppable,
            is_flipbook,
            is_bottom_solid,
            is_collidable,
            is_timer_enabled,
            timer_interval,
            threshold,
            image,
            side_image,
            side_material,
            top_material,
            slingshot_material,
            height_bottom,
            height_top,
            name,
            display_texture,
            slingshot_force,
            slingshot_threshold,
            elasticity,
            elasticity_falloff,
            friction,
            scatter,
            is_top_bottom_visible,
            slingshot_animation,
            is_side_visible,
            disable_lighting_top,
            disable_lighting_below,
            is_reflection_enabled,
            physics_material,
            overwrite_physics,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
            drag_points,
        }
    }
}
