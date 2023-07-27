use crate::vpx::{
    biff::{self, BiffRead, BiffReader, BiffWrite},
    color::Color,
};

use super::vertex3d::Vertex3D;

#[derive(Debug, PartialEq)]
pub struct Primitive {
    pub position: Vertex3D,              // 0 VPOS
    pub size: Vertex3D,                  // 1 VSIZ
    pub rot_and_tra: [f32; 9],           // 2-11 RTV0-RTV8
    pub image: String,                   // 12 IMAG
    pub normal_map: String,              // 13 NRMA
    pub sides: u32,                      // 14
    pub name: String,                    // 15
    pub material: String,                // 16
    pub side_color: Color,               // 17
    pub is_visible: bool,                // 18
    pub draw_textures_inside: bool,      // 19
    pub hit_event: bool,                 // 20
    pub threshold: f32,                  // 21
    pub elasticity: f32,                 // 22
    pub elasticity_falloff: f32,         // 23
    pub friction: f32,                   // 24
    pub scatter: f32,                    // 25
    pub edge_factor_ui: f32,             // 26
    pub collision_reduction_factor: f32, // 27
    pub is_collidable: bool,             // 28
    pub is_toy: bool,                    // 29
    pub use_3d_mesh: bool,               // 30
    pub static_rendering: bool,          // 31
    pub disable_lighting_top: bool,      // 32
    pub disable_lighting_below: bool,    // 33
    pub is_reflection_enabled: bool,     // 34
    pub backfaces_enabled: bool,         // 35
    pub physics_material: String,        // 36 MAPH
    pub overwrite_physics: bool,         // 37 OVPH
    pub display_texture: bool,           // 38 DIPT
    pub object_space_normal_map: bool,   // 38.5 OSNM
    pub mesh_file_name: String,          // 39 M3DN
    pub num_vertices: u32,               // 40 M3VN
    pub compressed_vertices: u32,        // 41 M3CY
    //pub m3cx: (),                        // 42 M3CX
    pub num_indices: u32,        // 42 M3FN
    pub compressed_indices: u32, // 43 M3CJ
    //pub m3ci: (),                        // 44 M3CI
    pub depth_bias: f32, // 45 PIDB
    pub add_blend: bool, // 46 ADDB
    pub alpha: f32,      // 47 FALP
    pub color: Color,    // 48 COLR

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl BiffRead for Primitive {
    fn biff_read(reader: &mut BiffReader<'_>) -> Primitive {
        let mut position = Default::default();
        let mut size = Vertex3D::new(100.0, 100.0, 100.0);
        let mut rot_and_tra: [f32; 9] = [0.0; 9];
        let mut image = Default::default();
        let mut normal_map = Default::default();
        let mut sides: u32 = 4;
        let mut name = Default::default();
        let mut material = Default::default();
        let mut side_color = Color::new_bgr(0x0);
        let mut is_visible: bool = true;
        let mut draw_textures_inside: bool = false;
        let mut hit_event: bool = true;
        let mut threshold: f32 = 2.0;
        let mut elasticity: f32 = 0.3;
        let mut elasticity_falloff: f32 = 0.5;
        let mut friction: f32 = 0.3;
        let mut scatter: f32 = 0.0;
        let mut edge_factor_ui: f32 = 0.25;
        let mut collision_reduction_factor: f32 = 0.0;
        let mut is_collidable: bool = true;
        let mut is_toy: bool = false;
        let mut use_3d_mesh: bool = false;
        let mut static_rendering: bool = false;
        let mut disable_lighting_top: bool = false;
        let mut disable_lighting_below: bool = false;
        let mut is_reflection_enabled: bool = true;
        let mut backfaces_enabled: bool = false;
        let mut physics_material: String = Default::default();
        let mut overwrite_physics: bool = true;
        let mut display_texture: bool = true;
        let mut object_space_normal_map: bool = false;
        let mut mesh_file_name: String = Default::default();
        let mut num_vertices: u32 = 0;
        let mut compressed_vertices: u32 = 0;
        let mut num_indices: u32 = 0;
        let mut compressed_indices: u32 = 0;
        let mut depth_bias: f32 = 0.0;
        let mut add_blend: bool = false;
        let mut alpha: f32 = 1.0;
        let mut color = Color::new_bgr(0x0);

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
            //println!("tag: {}", tag_str);
            match tag_str {
                // TOTAN4K had this
                // https://github.com/freezy/VisualPinball.Engine/blob/ec1e9765cd4832c134e889d6e6d03320bc404bd5/VisualPinball.Engine/VPT/Primitive/PrimitiveData.cs#L64
                // Unknown tag M3AY for vpxtool::vpx::gameitem::primitive::Primitive
                // Unknown tag M3AY for vpxtool::vpx::gameitem::primitive::Primitive
                // Unknown tag M3AY for vpxtool::vpx::gameitem::primitive::Primitive
                "VPOS" => {
                    position = Vertex3D::biff_read(reader);
                }
                "VSIZ" => {
                    size = Vertex3D::biff_read(reader);
                }
                "RTV0" => {
                    rot_and_tra[0] = reader.get_f32();
                }
                "RTV1" => {
                    rot_and_tra[1] = reader.get_f32();
                }
                "RTV2" => {
                    rot_and_tra[2] = reader.get_f32();
                }
                "RTV3" => {
                    rot_and_tra[3] = reader.get_f32();
                }
                "RTV4" => {
                    rot_and_tra[4] = reader.get_f32();
                }
                "RTV5" => {
                    rot_and_tra[5] = reader.get_f32();
                }
                "RTV6" => {
                    rot_and_tra[6] = reader.get_f32();
                }
                "RTV7" => {
                    rot_and_tra[7] = reader.get_f32();
                }
                "RTV8" => {
                    rot_and_tra[8] = reader.get_f32();
                }
                "IMAG" => {
                    image = reader.get_string();
                }
                "NRMA" => {
                    normal_map = reader.get_string();
                }
                "SIDS" => {
                    sides = reader.get_u32();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "MATR" => {
                    material = reader.get_string();
                }
                "SCOL" => {
                    side_color = Color::biff_read_bgr(reader);
                }
                "TVIS" => {
                    is_visible = reader.get_bool();
                }
                "DTXI" => {
                    draw_textures_inside = reader.get_bool();
                }
                "HTEV" => {
                    hit_event = reader.get_bool();
                }
                "THRS" => {
                    threshold = reader.get_f32();
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
                "EFUI" => {
                    edge_factor_ui = reader.get_f32();
                }
                "CORF" => {
                    collision_reduction_factor = reader.get_f32();
                }
                "CLDR" => {
                    is_collidable = reader.get_bool();
                }
                "ISTO" => {
                    is_toy = reader.get_bool();
                }
                "U3DM" => {
                    use_3d_mesh = reader.get_bool();
                }
                "STRE" => {
                    static_rendering = reader.get_bool();
                }
                //[BiffFloat("DILI", QuantizedUnsignedBits = 8, Pos = 32)]
                //public float DisableLightingTop; // m_d.m_fDisableLightingTop = (tmp == 1) ? 1.f : dequantizeUnsigned<8>(tmp); // backwards compatible hacky loading!
                "DILI" => {
                    disable_lighting_top = reader.get_bool();
                }
                "DILB" => {
                    disable_lighting_below = reader.get_bool();
                }
                "REEN" => {
                    is_reflection_enabled = reader.get_bool();
                }
                "EBFC" => {
                    backfaces_enabled = reader.get_bool();
                }
                "MAPH" => {
                    physics_material = reader.get_string();
                }
                "OVPH" => {
                    overwrite_physics = reader.get_bool();
                }
                "DIPT" => {
                    display_texture = reader.get_bool();
                }
                "OSNM" => {
                    object_space_normal_map = reader.get_bool();
                }
                "M3DN" => {
                    mesh_file_name = reader.get_string();
                }
                "M3VN" => {
                    num_vertices = reader.get_u32();
                }
                "M3CY" => {
                    compressed_vertices = reader.get_u32();
                }

                // [BiffVertices("M3DX", SkipWrite = true)]
                // [BiffVertices("M3CX", IsCompressed = true, Pos = 42)]
                // [BiffIndices("M3DI", SkipWrite = true)]
                // [BiffIndices("M3CI", IsCompressed = true, Pos = 45)]
                // [BiffAnimation("M3AX", IsCompressed = true, Pos = 47 )]
                // public Mesh Mesh = new Mesh();
                "M3CX" => {
                    reader.skip_tag();
                }
                "M3FN" => {
                    num_indices = reader.get_u32();
                }
                "M3CJ" => {
                    compressed_indices = reader.get_u32();
                }
                "M3CI" => {
                    reader.skip_tag();
                }
                "M3AX" => {
                    reader.skip_tag();
                }
                "PIDB" => {
                    depth_bias = reader.get_f32();
                }
                "ADDB" => {
                    add_blend = reader.get_bool();
                }
                "FALP" => {
                    alpha = reader.get_f32();
                }
                "COLR" => {
                    color = Color::biff_read_bgr(reader);
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
        Primitive {
            position,
            size,
            rot_and_tra,
            image,
            normal_map,
            sides,
            name,
            material,
            side_color,
            is_visible,
            draw_textures_inside,
            hit_event,
            threshold,
            elasticity,
            elasticity_falloff,
            friction,
            scatter,
            edge_factor_ui,
            collision_reduction_factor,
            is_collidable,
            is_toy,
            use_3d_mesh,
            static_rendering,
            disable_lighting_top,
            disable_lighting_below,
            is_reflection_enabled,
            backfaces_enabled,
            physics_material,
            overwrite_physics,
            display_texture,
            object_space_normal_map,
            mesh_file_name,
            num_vertices,
            compressed_vertices,
            num_indices,
            compressed_indices,
            depth_bias,
            add_blend,
            alpha,
            color,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}

impl BiffWrite for Primitive {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VPOS", &self.position);
        writer.write_tagged("VSIZ", &self.size);
        writer.write_tagged_f32("RTV0", self.rot_and_tra[0]);
        writer.write_tagged_f32("RTV1", self.rot_and_tra[1]);
        writer.write_tagged_f32("RTV2", self.rot_and_tra[2]);
        writer.write_tagged_f32("RTV3", self.rot_and_tra[3]);
        writer.write_tagged_f32("RTV4", self.rot_and_tra[4]);
        writer.write_tagged_f32("RTV5", self.rot_and_tra[5]);
        writer.write_tagged_f32("RTV6", self.rot_and_tra[6]);
        writer.write_tagged_f32("RTV7", self.rot_and_tra[7]);
        writer.write_tagged_f32("RTV8", self.rot_and_tra[8]);
        writer.write_tagged_string("IMAG", &self.image);
        writer.write_tagged_string("NRMA", &self.normal_map);
        writer.write_tagged_u32("SIDS", self.sides);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_string("MATR", &self.material);
        writer.write_tagged_with("SCOL", &self.side_color, Color::biff_write_bgr);
        writer.write_tagged_bool("TVIS", self.is_visible);
        writer.write_tagged_bool("DTXI", self.draw_textures_inside);
        writer.write_tagged_bool("HTEV", self.hit_event);
        writer.write_tagged_f32("THRS", self.threshold);
        writer.write_tagged_f32("ELAS", self.elasticity);
        writer.write_tagged_f32("ELFO", self.elasticity_falloff);
        writer.write_tagged_f32("RFCT", self.friction);
        writer.write_tagged_f32("RSCT", self.scatter);
        writer.write_tagged_f32("EFUI", self.edge_factor_ui);
        writer.write_tagged_f32("CORF", self.collision_reduction_factor);
        writer.write_tagged_bool("CLDR", self.is_collidable);
        writer.write_tagged_bool("ISTO", self.is_toy);
        writer.write_tagged_bool("U3DM", self.use_3d_mesh);
        writer.write_tagged_bool("STRE", self.static_rendering);
        writer.write_tagged_bool("DILI", self.disable_lighting_top);
        writer.write_tagged_bool("DILB", self.disable_lighting_below);
        writer.write_tagged_bool("REEN", self.is_reflection_enabled);
        writer.write_tagged_bool("EBFC", self.backfaces_enabled);
        writer.write_tagged_string("MAPH", &self.physics_material);
        writer.write_tagged_bool("OVPH", self.overwrite_physics);
        writer.write_tagged_bool("DIPT", self.display_texture);
        writer.write_tagged_bool("OSNM", self.object_space_normal_map);
        writer.write_tagged_string("M3DN", &self.mesh_file_name);
        writer.write_tagged_u32("M3VN", self.num_vertices);
        writer.write_tagged_u32("M3CY", self.compressed_vertices);
        writer.write_tagged_u32("M3FN", self.num_indices);
        writer.write_tagged_u32("M3CJ", self.compressed_indices);
        writer.write_tagged_f32("PIDB", self.depth_bias);
        writer.write_tagged_bool("ADDB", self.add_blend);
        writer.write_tagged_f32("FALP", self.alpha);
        writer.write_tagged_with("COLR", &self.color, Color::biff_write_bgr);
        // shared
        writer.write_tagged_bool("LOCK", self.is_locked);
        writer.write_tagged_u32("LAYR", self.editor_layer);
        writer.write_tagged_string("LANR", &self.editor_layer_name);
        writer.write_tagged_bool("LVIS", self.editor_layer_visibility);

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
        let primitive: Primitive = Primitive {
            position: Vertex3D::new(1.0, 2.0, 3.0),
            size: Vertex3D::new(4.0, 5.0, 6.0),
            rot_and_tra: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9],
            image: "image".to_string(),
            normal_map: "normal_map".to_string(),
            sides: 1,
            name: "name".to_string(),
            material: "material".to_string(),
            side_color: Color::new_bgr(0x12345678),
            is_visible: rng.gen(),
            // random bool
            draw_textures_inside: rng.gen(),
            hit_event: rng.gen(),
            threshold: 1.0,
            elasticity: 2.0,
            elasticity_falloff: 3.0,
            friction: 4.0,
            scatter: 5.0,
            edge_factor_ui: 6.0,
            collision_reduction_factor: 7.0,
            is_collidable: rng.gen(),
            is_toy: rng.gen(),
            use_3d_mesh: rng.gen(),
            static_rendering: rng.gen(),
            disable_lighting_top: rng.gen(),
            disable_lighting_below: rng.gen(),
            is_reflection_enabled: rng.gen(),
            backfaces_enabled: rng.gen(),
            physics_material: "physics_material".to_string(),
            overwrite_physics: rng.gen(),
            display_texture: rng.gen(),
            object_space_normal_map: rng.gen(),
            mesh_file_name: "mesh_file_name".to_string(),
            num_vertices: 8,
            compressed_vertices: 9,
            num_indices: 10,
            compressed_indices: 11,
            depth_bias: 12.0,
            add_blend: rng.gen(),
            alpha: 13.0,
            color: Color::new_bgr(0x23456789),
            is_locked: rng.gen(),
            editor_layer: 17,
            editor_layer_name: "editor_layer_name".to_string(),
            editor_layer_visibility: rng.gen(),
        };
        let mut writer = BiffWriter::new();
        Primitive::biff_write(&primitive, &mut writer);
        let primitive_read = Primitive::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(primitive, primitive_read);
    }
}
