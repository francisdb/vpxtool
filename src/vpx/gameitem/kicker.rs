use crate::vpx::biff::{self, BiffRead, BiffReader};

#[derive(Debug, PartialEq)]
pub struct Kicker {
    pub name: String,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl BiffRead for Kicker {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let mut name = Default::default();

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
                // Unknown tag VCEN for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag RADI for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag TMON for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag TMIN for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag MATR for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag SURF for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag EBLD for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag TYPE for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag KSCT for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag KHAC for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag KHHI for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag KORI for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag FATH for vpxtool::vpx::gameitem::kicker::Kicker
                // Unknown tag LEMO for vpxtool::vpx::gameitem::kicker::Kicker
                "NAME" => {
                    name = reader.get_wide_string();
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
        Self {
            name,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}
