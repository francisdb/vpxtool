use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite, BiffWriter};

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
    pub elasticity_falloff: Option<f32>,
    pub friction: f32,
    pub scatter: f32,
    pub is_top_bottom_visible: bool,
    pub slingshot_animation: bool,
    pub is_side_visible: bool,
    pub disable_lighting_top: f32,
    pub disable_lighting_below: Option<f32>, // DILB (added in 10.?)
    pub is_reflection_enabled: bool,
    pub physics_material: Option<String>, // MAPH (added in 10.?)
    pub overwrite_physics: Option<bool>,  // OVPH (added in 10.?)

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,

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
        let mut elasticity_falloff: Option<f32> = None; // added in ?
        let mut friction: f32 = 0.3;
        let mut scatter: f32 = Default::default();
        let mut is_top_bottom_visible: bool = true;
        let mut overwrite_physics: Option<bool> = None; //true;
        let mut disable_lighting_top: f32 = Default::default();
        let mut disable_lighting_below: Option<f32> = None;
        let mut is_side_visible: bool = true;
        let mut is_reflection_enabled: bool = true;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = Default::default();
        let mut physics_material: Option<String> = None;

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
                "HTBT" => {
                    height_bottom = reader.get_f32();
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
                    elasticity_falloff = Some(reader.get_f32());
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
                    overwrite_physics = Some(reader.get_bool());
                }
                "DLTO" => {
                    disable_lighting_top = reader.get_f32();
                }
                "DLBE" => {
                    disable_lighting_below = Some(reader.get_f32());
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
                    physics_material = Some(reader.get_string());
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
                    disable_lighting_below = Some(reader.get_f32());
                }
                "MAPH" => {
                    physics_material = Some(reader.get_string());
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
                    friction = reader.get_f32();
                }
                "WSCT" => {
                    scatter = reader.get_f32();
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

impl BiffWrite for Wall {
    fn biff_write(&self, writer: &mut BiffWriter) {
        writer.write_tagged_bool("HTEV", self.hit_event);
        writer.write_tagged_bool("DROP", self.is_droppable);
        writer.write_tagged_bool("FLIP", self.is_flipbook);
        writer.write_tagged_bool("ISBS", self.is_bottom_solid);
        writer.write_tagged_bool("CLDW", self.is_collidable);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_u32("TMIN", self.timer_interval);
        writer.write_tagged_f32("THRS", self.threshold);
        writer.write_tagged_string("IMAG", &self.image);
        writer.write_tagged_string("SIMG", &self.side_image);
        writer.write_tagged_string("SIMA", &self.side_material);
        writer.write_tagged_string("TOMA", &self.top_material);
        writer.write_tagged_string("SLMA", &self.slingshot_material);
        writer.write_tagged_f32("HTBT", self.height_bottom);
        writer.write_tagged_f32("HTTP", self.height_top);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_bool("DSPT", self.display_texture);
        writer.write_tagged_f32("SLGF", self.slingshot_force);
        writer.write_tagged_f32("SLTH", self.slingshot_threshold);
        writer.write_tagged_f32("ELAS", self.elasticity);
        if let Some(elasticity_falloff) = self.elasticity_falloff {
            writer.write_tagged_f32("ELFO", elasticity_falloff);
        }
        writer.write_tagged_f32("WFCT", self.friction);
        writer.write_tagged_f32("WSCT", self.scatter);
        writer.write_tagged_bool("VSBL", self.is_top_bottom_visible);
        writer.write_tagged_bool("SLGA", self.slingshot_animation);
        writer.write_tagged_bool("SVBL", self.is_side_visible);
        writer.write_tagged_f32("DILI", self.disable_lighting_top);
        if let Some(disable_lighting_below) = self.disable_lighting_below {
            writer.write_tagged_f32("DILB", disable_lighting_below);
        }
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
        for point in &self.drag_points {
            writer.write_tagged("DPNT", point);
        }

        writer.close(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_write_read() {
        let wall = Wall {
            hit_event: true,
            is_droppable: true,
            is_flipbook: true,
            is_bottom_solid: true,
            is_collidable: true,
            is_timer_enabled: true,
            timer_interval: 1,
            threshold: 2.0,
            image: "image".to_string(),
            side_image: "side_image".to_string(),
            side_material: "side_material".to_string(),
            top_material: "top_material".to_string(),
            slingshot_material: "slingshot_material".to_string(),
            height_bottom: 3.0,
            height_top: 4.0,
            name: "name".to_string(),
            display_texture: true,
            slingshot_force: 5.0,
            slingshot_threshold: 6.0,
            elasticity: 7.0,
            elasticity_falloff: Some(8.0),
            friction: 9.0,
            scatter: 10.0,
            is_top_bottom_visible: true,
            slingshot_animation: true,
            is_side_visible: true,
            disable_lighting_top: 11.0,
            disable_lighting_below: Some(12.0),
            is_reflection_enabled: true,
            physics_material: Some("physics_material".to_string()),
            overwrite_physics: Some(true),
            is_locked: true,
            editor_layer: 13,
            editor_layer_name: Some("editor_layer_name".to_string()),
            editor_layer_visibility: Some(true),
            drag_points: vec![DragPoint::default()],
        };
        let mut writer = BiffWriter::new();
        Wall::biff_write(&wall, &mut writer);
        let wall_read = Wall::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(wall, wall_read);
    }
}
