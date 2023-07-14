use crate::vpx::biff::{BiffRead, BiffReader};

#[derive(Debug, PartialEq)]
pub struct Vertex2D {
    x: f32,
    y: f32,
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
        let x = reader.get_float();
        let y = reader.get_float();
        Vertex2D { x, y }
    }
}
