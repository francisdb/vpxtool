use crate::vpx::biff::{BiffRead, BiffReader, BiffWrite, BiffWriter};

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
        let x = reader.get_f32();
        let y = reader.get_f32();
        let z = reader.get_f32();
        let _padding = reader.get_f32();
        Vertex3D { x, y, z }
    }
}

impl BiffWrite for Vertex3D {
    fn biff_write(&self, writer: &mut BiffWriter) {
        writer.write_f32(self.x);
        writer.write_f32(self.y);
        writer.write_f32(self.z);
        writer.write_f32(0.0);
    }
}

// TODO enable and fix this test

// #[cfg(test)]
// mod tests {
//     use crate::vpx::biff::BiffWriter;

//     use super::*;
//     use pretty_assertions::assert_eq;

//     #[test]
//     fn test_write_read() {
//         // values not equal to the defaults
//         let vertex = Vertex3D {
//             x: 1.0,
//             y: 2.0,
//             z: 3.0,
//         };
//         let mut writer = BiffWriter::new();
//         Vertex3D::biff_write(&vertex, &mut writer);
//         let vertex_read = Vertex3D::biff_read(&mut BiffReader::new(writer.get_data()));
//         assert_eq!(vertex, vertex_read);
//     }
// }
