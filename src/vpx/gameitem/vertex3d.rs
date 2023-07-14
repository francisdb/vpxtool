use crate::vpx::biff::{BiffRead, BiffReader};

#[derive(Debug, PartialEq)]
pub struct Vertex3D {
    x: f32,
    y: f32,
    z: f32,
}

impl Vertex3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl std::fmt::Display for Vertex3D {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{},{},{}", self.x, self.y, self.z)
    }
}

impl Default for Vertex3D {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl BiffRead for Vertex3D {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        let x = reader.get_float();
        let y = reader.get_float();
        let z = reader.get_float();
        let _padding = reader.get_float();
        Vertex3D { x, y, z }
    }
}
