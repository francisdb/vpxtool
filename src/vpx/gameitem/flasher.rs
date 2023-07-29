use base64::write;

use crate::vpx::{
    biff::{self, BiffRead, BiffReader, BiffWrite},
    color::Color,
};

use super::{dragpoint::DragPoint, FILTER_OVERLAY, IMAGE_ALIGN_CENTER, IMAGE_ALIGN_TOP_LEFT};

// [BiffString("NAME", IsWideString = true, Pos = 10)]
// public string Name = string.Empty;

// [BiffFloat("FHEI", Pos = 1)]
// public float Height = 50.0f;

// [BiffFloat("FLAX", Pos = 2)] public float PosX { set => Center.X = value; get => Center.X; }
// [BiffFloat("FLAY", Pos = 3)] public float PosY { set => Center.Y = value; get => Center.Y; }
// public Vertex2D Center = new Vertex2D();

// [BiffFloat("FROX", Pos = 4)]
// public float RotX = 0.0f;

// [BiffFloat("FROY", Pos = 5)]
// public float RotY = 0.0f;

// [BiffFloat("FROZ", Pos = 6)]
// public float RotZ = 0.0f;

// [BiffColor("COLR", Pos = 7)]
// public Color Color = new Color(0xfffffff, ColorFormat.Bgr);

// [BiffString("IMAG", Pos = 11)]
// public string ImageA;

// [BiffString("IMAB", Pos = 12)]
// public string ImageB;

// [BiffInt("FALP", Min = 0, Pos = 13)]
// public int Alpha = 100;

// [BiffFloat("MOVA", Pos = 14)]
// public float ModulateVsAdd = 0.9f;

// [BiffBool("FVIS", Pos = 15)]
// public bool IsVisible = true;

// [BiffBool("ADDB", Pos = 17)]
// public bool AddBlend = false;

// [BiffBool("IDMD", Pos = 18)]
// public bool IsDmd = false;

// [BiffBool("DSPT", Pos = 16)]
// public bool DisplayTexture = false;

// [BiffFloat("FLDB", Pos = 19)]
// public float DepthBias = 0.0f;

// [BiffInt("ALGN", Pos = 20)]
// public int ImageAlignment = VisualPinball.Engine.VPT.ImageAlignment.ImageAlignTopLeft;

// [BiffInt("FILT", Pos = 21)]
// public int Filter = Filters.Filter_Overlay;

#[derive(Debug, PartialEq)]
pub struct Flasher {
    pub height: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub color: Color,
    pub is_timer_enabled: bool,
    pub timer_interval: i32,
    pub name: String,
    pub image_a: String,
    pub image_b: String,
    pub alpha: i32,
    pub modulate_vs_add: f32,
    pub is_visible: bool,
    pub add_blend: bool,
    pub is_dmd: bool,
    pub display_texture: bool,
    pub depth_bias: f32,
    pub image_alignment: u32,
    pub filter: u32,
    pub filter_amount: u32,
    pub drag_points: Vec<DragPoint>,
    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl BiffRead for Flasher {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut height = 50.0;
        let mut pos_x = Default::default();
        let mut pos_y = Default::default();
        let mut rot_x = Default::default();
        let mut rot_y = Default::default();
        let mut rot_z = Default::default();
        let mut color = Color::new_bgr(0xfffffff);
        let mut is_timer_enabled = Default::default();
        let mut timer_interval = Default::default();
        let mut name = Default::default();
        let mut image_a = Default::default();
        let mut image_b = Default::default();
        let mut alpha = 100;
        let mut modulate_vs_add = 0.9;
        let mut is_visible = true;
        let mut add_blend = Default::default();
        let mut is_dmd = Default::default();
        let mut display_texture = Default::default();
        let mut depth_bias = Default::default();
        let mut image_alignment = IMAGE_ALIGN_TOP_LEFT;
        let mut filter = FILTER_OVERLAY;
        let mut filter_amount: u32 = 100;

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
                "FHEI" => {
                    height = reader.get_f32();
                }
                "FLAX" => {
                    pos_x = reader.get_f32();
                }
                "FLAY" => {
                    pos_y = reader.get_f32();
                }
                "FROX" => {
                    rot_x = reader.get_f32();
                }
                "FROY" => {
                    rot_y = reader.get_f32();
                }
                "FROZ" => {
                    rot_z = reader.get_f32();
                }
                "COLR" => {
                    color = Color::biff_read_bgr(reader);
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
                    image_a = reader.get_string();
                }
                "IMAB" => {
                    image_b = reader.get_string();
                }
                "FALP" => {
                    alpha = reader.get_i32();
                }
                "MOVA" => {
                    modulate_vs_add = reader.get_f32();
                }
                "FVIS" => {
                    is_visible = reader.get_bool();
                }
                "DSPT" => {
                    display_texture = reader.get_bool();
                }
                "ADDB" => {
                    add_blend = reader.get_bool();
                }
                "IDMD" => {
                    is_dmd = reader.get_bool();
                }
                "FLDB" => {
                    depth_bias = reader.get_f32();
                }
                "ALGN" => {
                    image_alignment = reader.get_u32();
                }
                "FILT" => {
                    filter = reader.get_u32();
                }
                "FIAM" => {
                    filter_amount = reader.get_u32();
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
        Flasher {
            height,
            pos_x,
            pos_y,
            rot_x,
            rot_y,
            rot_z,
            color,
            is_timer_enabled,
            timer_interval,
            name,
            image_a,
            image_b,
            alpha,
            modulate_vs_add,
            is_visible,
            add_blend,
            is_dmd,
            display_texture,
            depth_bias,
            image_alignment,
            filter,
            filter_amount,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
            drag_points,
        }
    }
}

impl BiffWrite for Flasher {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged_f32("FHEI", self.height);
        writer.write_tagged_f32("FLAX", self.pos_x);
        writer.write_tagged_f32("FLAY", self.pos_y);
        writer.write_tagged_f32("FROX", self.rot_x);
        writer.write_tagged_f32("FROY", self.rot_y);
        writer.write_tagged_f32("FROZ", self.rot_z);
        writer.write_tagged_with("COLR", &self.color, Color::biff_write_bgr);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_i32("TMIN", self.timer_interval);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_string("IMAG", &self.image_a);
        writer.write_tagged_string("IMAB", &self.image_b);
        writer.write_tagged_i32("FALP", self.alpha);
        writer.write_tagged_f32("MOVA", self.modulate_vs_add);
        writer.write_tagged_bool("FVIS", self.is_visible);
        writer.write_tagged_bool("DSPT", self.display_texture);
        writer.write_tagged_bool("ADDB", self.add_blend);
        writer.write_tagged_bool("IDMD", self.is_dmd);
        writer.write_tagged_f32("FLDB", self.depth_bias);
        writer.write_tagged_u32("ALGN", self.image_alignment);
        writer.write_tagged_u32("FILT", self.filter);
        writer.write_tagged_u32("FIAM", self.filter_amount);
        // shared
        writer.write_tagged_bool("LOCK", self.is_locked);
        writer.write_tagged_u32("LAYR", self.editor_layer);
        writer.write_tagged_string("LANR", &self.editor_layer_name);
        writer.write_tagged_bool("LVIS", self.editor_layer_visibility);

        for drag_point in &self.drag_points {
            writer.write_tagged("DPNT", drag_point);
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
        let flasher = Flasher {
            height: rng.gen(),
            pos_x: rng.gen(),
            pos_y: rng.gen(),
            rot_x: rng.gen(),
            rot_y: rng.gen(),
            rot_z: rng.gen(),
            color: Color::new_bgr(rng.gen()),
            is_timer_enabled: rng.gen(),
            timer_interval: rng.gen(),
            name: "test name".to_string(),
            image_a: "test image a".to_string(),
            image_b: "test image b".to_string(),
            alpha: rng.gen(),
            modulate_vs_add: rng.gen(),
            is_visible: rng.gen(),
            add_blend: rng.gen(),
            is_dmd: rng.gen(),
            display_texture: rng.gen(),
            depth_bias: rng.gen(),
            image_alignment: rng.gen(),
            filter: rng.gen(),
            filter_amount: rng.gen(),
            is_locked: rng.gen(),
            editor_layer: rng.gen(),
            editor_layer_name: "test layer".to_string(),
            editor_layer_visibility: rng.gen(),
            drag_points: vec![DragPoint::default()],
        };
        let mut writer = BiffWriter::new();
        Flasher::biff_write(&flasher, &mut writer);
        let flasher_read = Flasher::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(flasher, flasher_read);
    }
}
