use crate::vpx::biff::{self, BiffRead, BiffReader};

use super::vertex3d::Vertex3D;

#[derive(Debug, PartialEq)]
pub struct Primitive {
    pub position: Vertex3D,
    pub size: Vertex3D,

    pub name: String,
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
                // Unknown tag: VSIZ
                // Unknown tag: RTV0
                // Unknown tag: RTV1
                // Unknown tag: RTV2
                // Unknown tag: RTV3
                // Unknown tag: RTV4
                // Unknown tag: RTV5
                // Unknown tag: RTV6
                // Unknown tag: RTV7
                // Unknown tag: RTV8
                // Unknown tag: IMAG
                // Unknown tag: NRMA
                // Unknown tag: SIDS
                "VPOS" => {
                    position = Vertex3D::biff_read(reader);
                }
                "VSIZ" => {
                    size = Vertex3D::biff_read(reader);
                }
                "NAME" => {
                    name = reader.get_wide_string();
                }
                // Unknown tag: MATR
                // Unknown tag: SCOL
                // Unknown tag: TVIS
                // Unknown tag: DTXI
                // Unknown tag: HTEV
                // Unknown tag: THRS
                // Unknown tag: ELAS
                // Unknown tag: ELFO
                // Unknown tag: RFCT
                // Unknown tag: RSCT
                // Unknown tag: EFUI
                // Unknown tag: CORF
                // Unknown tag: CLDR
                // Unknown tag: ISTO
                // Unknown tag: U3DM
                // Unknown tag: STRE
                // Unknown tag: DILI
                // Unknown tag: DILB
                // Unknown tag: REEN
                // Unknown tag: EBFC
                // Unknown tag: MAPH
                // Unknown tag: OVPH
                // Unknown tag: DIPT
                // Unknown tag: OSNM
                // Unknown tag: M3DN
                // Unknown tag: M3VN
                // Unknown tag: M3CY
                // Unknown tag: M3CX
                // Unknown tag: M3FN
                // Unknown tag: M3CJ
                // Unknown tag: M3CI
                // Unknown tag: PIDB
                // Unknown tag: ADDB
                // Unknown tag: FALP
                // Unknown tag: COLR

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
                    println!("Unknown tag: {}", tag_str);
                    reader.skip_tag();
                }
            }
        }
        Primitive {
            position,
            size,
            name,
            is_locked,
            editor_layer,
            editor_layer_name,
            editor_layer_visibility,
        }
    }
}
