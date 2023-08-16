use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

use super::GameItem;

#[derive(Debug, PartialEq)]
pub struct DragPoint {
    x: f32,
    y: f32,
    z: f32,
    smooth: bool,
    is_slingshot: Option<bool>,
    has_auto_texture: bool,
    tex_coord: f32,

    // Somehow below items don't belong here?
    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: Option<String>, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>,
}

impl DragPoint {
    pub fn default() -> Self {
        let x = 0.0;
        let y = 0.0;
        let z = 0.0;
        let tex_coord = 0.0;
        let smooth = false;
        let is_slingshot: Option<bool> = None;
        let has_auto_texture = false;

        // these are shared between all items
        let is_locked: bool = false;
        let editor_layer: u32 = Default::default();
        let editor_layer_name: Option<String> = None;
        let editor_layer_visibility: Option<bool> = None;
        Self {
            x,
            y,
            z,
            smooth,
            is_slingshot,
            has_auto_texture,
            tex_coord,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}

impl GameItem for DragPoint {
    fn name(&self) -> &str {
        "Unnamed DragPoint"
    }
}

impl BiffRead for DragPoint {
    fn biff_read(reader: &mut BiffReader<'_>) -> DragPoint {
        let mut sub_data = reader.child_reader();

        let mut dragpoint = DragPoint::default();

        loop {
            sub_data.next(biff::WARN);
            if sub_data.is_eof() {
                break;
            }
            let tag = sub_data.tag();
            let tag_str = tag.as_str();
            match tag_str {
                "VCEN" => {
                    dragpoint.x = sub_data.get_f32();
                    dragpoint.y = sub_data.get_f32();
                }
                "POSZ" => {
                    dragpoint.z = sub_data.get_f32();
                }
                "SMTH" => {
                    dragpoint.smooth = sub_data.get_bool();
                }
                "SLNG" => {
                    dragpoint.is_slingshot = Some(sub_data.get_bool());
                }
                "ATEX" => {
                    dragpoint.has_auto_texture = sub_data.get_bool();
                }
                "TEXC" => {
                    dragpoint.tex_coord = sub_data.get_f32();
                }
                // shared
                "LOCK" => {
                    dragpoint.is_locked = sub_data.get_bool();
                }
                "LAYR" => {
                    dragpoint.editor_layer = sub_data.get_u32();
                }
                "LANR" => {
                    dragpoint.editor_layer_name = Some(sub_data.get_string());
                }
                "LVIS" => {
                    dragpoint.editor_layer_visibility = Some(sub_data.get_bool());
                }
                other => {
                    println!(
                        "Unknown tag {} for {}",
                        other,
                        std::any::type_name::<Self>()
                    );
                    sub_data.skip_tag();
                }
            }
        }
        let pos = sub_data.pos();
        reader.skip_end_tag(pos);
        dragpoint
    }
}

impl BiffWrite for DragPoint {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.new_tag("VCEN");
        writer.write_f32(self.x);
        writer.write_f32(self.y);
        writer.end_tag();
        writer.write_tagged_f32("POSZ", self.z);
        writer.write_tagged_bool("SMTH", self.smooth);
        if let Some(is_slingshot) = self.is_slingshot {
            writer.write_tagged_bool("SLNG", is_slingshot);
        }
        writer.write_tagged_bool("ATEX", self.has_auto_texture);
        writer.write_tagged_f32("TEXC", self.tex_coord);
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
        let dragpoint = DragPoint {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            smooth: true,
            is_slingshot: Some(true),
            has_auto_texture: true,
            tex_coord: 4.0,
            is_locked: true,
            editor_layer: 1,
            editor_layer_name: Some("test layer".to_string()),
            editor_layer_visibility: Some(true),
        };
        let mut writer = BiffWriter::new();
        DragPoint::biff_write(&dragpoint, &mut writer);
        let dragpoint_read = DragPoint::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(dragpoint, dragpoint_read);
    }
}
