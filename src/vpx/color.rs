use crate::vpx::biff::BiffReader;

use super::biff::BiffWriter;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color {
    a: u8,
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new_argb(arg: u32) -> Color {
        let a = ((arg >> 24) & 0xff) as u8;
        let r = ((arg >> 16) & 0xff) as u8;
        let g = ((arg >> 8) & 0xff) as u8;
        let b = (arg & 0xff) as u8;
        Color { a, r, g, b }
    }

    pub fn new_bgr(arg: u32) -> Color {
        let a = ((arg >> 24) & 0xff) as u8;
        let b = ((arg >> 16) & 0xff) as u8;
        let g = ((arg >> 8) & 0xff) as u8;
        let r = (arg & 0xff) as u8;
        Color { a, r, g, b }
    }

    pub fn bgr(&self) -> u32 {
        let a = (self.a as u32) << 24;
        let b = (self.b as u32) << 16;
        let g = (self.g as u32) << 8;
        let r = self.r as u32;
        a | b | g | r
    }

    pub fn argb(&self) -> u32 {
        let a = (self.a as u32) << 24;
        let r = (self.r as u32) << 16;
        let g = (self.g as u32) << 8;
        let b = self.b as u32;
        a | r | g | b
    }

    pub const BLACK: Color = Color {
        a: 255,
        r: 0,
        g: 0,
        b: 0,
    };
    pub const WHITE: Color = Color {
        a: 255,
        r: 255,
        g: 255,
        b: 255,
    };
    pub const RED: Color = Color {
        a: 255,
        r: 255,
        g: 0,
        b: 0,
    };

    // TODO do we want a BiffRead with a parameter?

    pub fn biff_read_argb(reader: &mut BiffReader<'_>) -> Color {
        let a = reader.get_u8();
        let r = reader.get_u8();
        let g = reader.get_u8();
        let b = reader.get_u8();
        Color { a, r, g, b }
    }

    pub fn biff_read_bgr(reader: &mut BiffReader<'_>) -> Color {
        let a = reader.get_u8();
        let b = reader.get_u8();
        let g = reader.get_u8();
        let r = reader.get_u8();
        Color { a, r, g, b }
    }

    pub fn biff_write_argb(&self, writer: &mut BiffWriter) {
        writer.write_u8(self.a);
        writer.write_u8(self.r);
        writer.write_u8(self.g);
        writer.write_u8(self.b);
    }

    pub fn biff_write_bgr(&self, writer: &mut BiffWriter) {
        writer.write_u8(self.a);
        writer.write_u8(self.b);
        writer.write_u8(self.g);
        writer.write_u8(self.r);
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}",
            self.a, self.r, self.g, self.b
        )
    }
}
