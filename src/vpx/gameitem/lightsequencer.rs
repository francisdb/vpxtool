use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct LightSequencer {
    center: Vertex2D,
    collection: String,
    pos_x: f32,
    pos_y: f32,
    update_interval: u32,
    is_timer_enabled: bool,
    timer_interval: u32,
    pub name: String,
    backglass: bool,

    // these are shared between all items
    pub is_locked: Option<bool>,               // LOCK (added in 10.7?)
    pub editor_layer: Option<u32>,             // LAYR (added in 10.7?)
    pub editor_layer_name: Option<String>, // LANR (added in 10.7?) default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: Option<bool>, // LVIS (added in 10.7?)
}

impl BiffRead for LightSequencer {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut collection = Default::default();
        let mut pos_x: f32 = Default::default();
        let mut pos_y: f32 = Default::default();
        let mut update_interval: u32 = 25;
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: u32 = 0;
        let mut name = Default::default();
        let mut backglass: bool = false;

        // these are shared between all items
        let mut is_locked: Option<bool> = None; //false;
        let mut editor_layer: Option<u32> = None;
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
                "COLC" => {
                    collection = reader.get_wide_string();
                }
                "CTRX" => {
                    pos_x = reader.get_f32();
                }
                "CTRY" => {
                    pos_y = reader.get_f32();
                }
                "UPTM" => {
                    update_interval = reader.get_u32();
                }
                "TMON" => {
                    is_timer_enabled = reader.get_bool();
                }
                "TMIN" => {
                    timer_interval = reader.get_u32();
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                "BGLS" => {
                    backglass = reader.get_bool();
                }

                // shared
                "LOCK" => {
                    is_locked = Some(reader.get_bool());
                }
                "LAYR" => {
                    editor_layer = Some(reader.get_u32());
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
            collection,
            pos_x,
            pos_y,
            update_interval,
            is_timer_enabled,
            timer_interval,
            name,
            backglass,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}

impl BiffWrite for LightSequencer {
    fn biff_write(&self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VCEN", &self.center);
        writer.write_tagged_wide_string("COLC", &self.collection);
        writer.write_tagged_f32("CTRX", self.pos_x);
        writer.write_tagged_f32("CTRY", self.pos_y);
        writer.write_tagged_u32("UPTM", self.update_interval);
        writer.write_tagged_bool("TMON", self.is_timer_enabled);
        writer.write_tagged_u32("TMIN", self.timer_interval);
        writer.write_tagged_wide_string("NAME", &self.name);
        writer.write_tagged_bool("BGLS", self.backglass);

        // shared
        if let Some(is_locked) = self.is_locked {
            writer.write_tagged_bool("LOCK", is_locked);
        }
        if let Some(editor_layer) = self.editor_layer {
            writer.write_tagged_u32("LAYR", editor_layer);
        }
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
    use rand::Rng;

    #[test]
    fn test_write_read() {
        let mut rng = rand::thread_rng();
        // values not equal to the defaults
        let spinner = LightSequencer {
            center: Vertex2D::new(rng.gen(), rng.gen()),
            collection: "test collection".to_string(),
            pos_x: rng.gen(),
            pos_y: rng.gen(),
            update_interval: rng.gen(),
            is_timer_enabled: rng.gen(),
            timer_interval: rng.gen(),
            name: "test name".to_string(),
            backglass: rng.gen(),
            is_locked: rng.gen(),
            editor_layer: rng.gen(),
            editor_layer_name: Some("test layer name".to_string()),
            editor_layer_visibility: rng.gen(),
        };
        let mut writer = BiffWriter::new();
        LightSequencer::biff_write(&spinner, &mut writer);
        let spinner_read = LightSequencer::biff_read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(spinner, spinner_read);
    }
}
