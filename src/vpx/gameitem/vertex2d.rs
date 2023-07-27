use crate::vpx::biff::{BiffRead, BiffReader, BiffWrite, BiffWriter};

#[derive(Debug, PartialEq)]
pub struct Vertex2D {
    x: f32,
    y: f32,
}
impl Vertex2D {
    pub(crate) fn new(x: f64, y: f64) -> Vertex2D {
        Vertex2D {
            x: x as f32,
            y: y as f32,
        }
    }
}

impl std::fmt::Display for Vertex2D {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl Default for Vertex2D {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl BiffRead for Vertex2D {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let x = reader.get_f32();
        let y = reader.get_f32();
        Vertex2D { x, y }
    }
}
impl BiffWrite for Vertex2D {
    fn biff_write(vertex: &Self, writer: &mut BiffWriter) {
        writer.write_f32(vertex.x);
        writer.write_f32(vertex.y);
    }
}
