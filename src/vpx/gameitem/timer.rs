use crate::vpx::biff::{self, BiffRead, BiffReader, BiffWrite};

use super::vertex2d::Vertex2D;

#[derive(Debug, PartialEq)]
pub struct Timer {
    pub center: Vertex2D,
    pub is_timer_enabled: bool,
    pub timer_interval: i32,
    pub name: String,
    pub backglass: bool,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl BiffRead for Timer {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut center = Vertex2D::default();
        let mut is_timer_enabled: bool = false;
        let mut timer_interval: i32 = Default::default();
        let mut name = String::new();
        let mut backglass: bool = false;

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
            match tag_str {
                "VCEN" => {
                    center = Vertex2D::biff_read(reader);
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
                "BGLS" => {
                    backglass = reader.get_bool();
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
        Timer {
            center,
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

impl BiffWrite for Timer {
    fn biff_write(item: &Self, writer: &mut biff::BiffWriter) {
        writer.write_tagged("VCEN", &item.center);
        writer.write_tagged_bool("TMON", item.is_timer_enabled);
        writer.write_tagged_i32("TMIN", item.timer_interval);
        writer.write_tagged_wide_string("NAME", &item.name);
        writer.write_tagged_bool("BGLS", item.backglass);
        // shared
        writer.write_tagged_bool("LOCK", item.is_locked);
        writer.write_tagged_u32("LAYR", item.editor_layer);
        writer.write_tagged_string("LANR", &item.editor_layer_name);
        writer.write_tagged_bool("LVIS", item.editor_layer_visibility);
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
        let timer = Timer {
            center: Vertex2D::new(1.0, 2.0),
            is_timer_enabled: true,
            timer_interval: 3,
            name: "test timer".to_string(),
            backglass: false,
            is_locked: true,
            editor_layer: 5,
            editor_layer_name: "test layer".to_string(),
            editor_layer_visibility: false,
        };
        let mut writer = BiffWriter::new();
        Timer::biff_write(&timer, &mut writer);
        let timer_read = Timer::biff_read(&mut BiffReader::new(&writer.get_data()));
        assert_eq!(timer, timer_read);
    }
}
