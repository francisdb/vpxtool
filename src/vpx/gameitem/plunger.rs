use crate::vpx::biff::{self, BiffRead, BiffReader};

#[derive(Debug, PartialEq)]
pub struct Plunger {
    pub name: String,

    // these are shared between all items
    pub is_locked: bool,
    pub editor_layer: u32,
    pub editor_layer_name: String, // default "Layer_{editor_layer + 1}"
    pub editor_layer_visibility: bool,
}

impl BiffRead for Plunger {
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
                // Unknown tag VCEN for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag WDTH for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag HIGH for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag ZADJ for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag HPSL for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag SPDP for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag SPDF for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag TYPE for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag ANFR for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag MATR for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag IMAG for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag MEST for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag MECH for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag APLG for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag MPRK for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag PSCV for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag MOMX for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag TMON for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag TMIN for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag VSBL for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag REEN for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag SURF for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag TIPS for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag RODD for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag RNGG for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag RNGD for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag RNGW for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag SPRD for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag SPRG for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag SPRL for vpxtool::vpx::gameitem::plunger::Plunger
                // Unknown tag SPRE for vpxtool::vpx::gameitem::plunger::Plunger
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
