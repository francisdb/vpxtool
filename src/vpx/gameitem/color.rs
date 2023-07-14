use crate::vpx::biff::{BiffRead, BiffReader};

// maybe we also want this format
// public Color Color = new Color(0xffff00, ColorFormat.Argb);
#[derive(Debug, PartialEq)]
pub struct ColorARGB {
    a: u8,
    r: u8,
    g: u8,
    b: u8,
}

impl ColorARGB {
    pub fn new(arg: u32) -> ColorARGB {
        let a = ((arg >> 24) & 0xff) as u8;
        let r = ((arg >> 16) & 0xff) as u8;
        let g = ((arg >> 8) & 0xff) as u8;
        let b = (arg & 0xff) as u8;
        ColorARGB { a, r, g, b }
    }
}

impl std::fmt::Display for ColorARGB {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}",
            self.a, self.r, self.g, self.b
        )
    }
}

impl BiffRead for ColorARGB {
    fn biff_read(reader: &mut BiffReader<'_>) -> ColorARGB {
        let a = reader.get_u8();
        let r = reader.get_u8();
        let g = reader.get_u8();
        let b = reader.get_u8();
        ColorARGB { a, r, g, b }
    }
}
