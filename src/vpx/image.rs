use std::fmt;

use super::biff::{self, BiffRead, BiffReader, BiffWrite, BiffWriter};

#[derive(PartialEq)]
pub struct ImageDataJpeg {
    path: String,
    name: String,
    // /**
    //  * Lowercased name?
    //  */
    inme: Option<String>,
    // alpha_test_value: f32,
    pub data: Vec<u8>,
}

impl fmt::Debug for ImageDataJpeg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // avoid writing the data to the debug output
        f.debug_struct("ImageDataJpeg")
            .field("path", &self.path)
            .field("name", &self.name)
            // .field("alpha_test_value", &self.alpha_test_value)
            .field("data", &self.data.len())
            .finish()
    }
}

/**
 * An bitmap blob, typically used by textures.
 */
#[derive(PartialEq)]
pub struct ImageDataBits {
    pub data: Vec<u8>,
}

impl fmt::Debug for ImageDataBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // avoid writing the data to the debug output
        f.debug_struct("ImageDataJpeg")
            .field("data", &self.data.len())
            .finish()
    }
}

#[derive(PartialEq, Debug)]
pub struct ImageData {
    pub name: String, // NAME
    // /**
    //  * Lowercased name?
    //  */
    inme: Option<String>,
    path: String, // PATH
    width: u32,   // WDTH
    height: u32,  // HGHT
    // TODO seems to be 1 for some kind of link type img, related to screenshots.
    // we only see this where a screenshot is set on the table info.
    // https://github.com/vpinball/vpinball/blob/1a70aa35eb57ec7b5fbbb9727f6735e8ef3183e0/Texture.cpp#L588
    link: Option<u32>,     // LINK
    alpha_test_value: f32, // ALTV
    // TODO we can probably only have one of these so we can make an enum
    pub jpeg: Option<ImageDataJpeg>,
    pub bits: Option<ImageDataBits>,
}

impl ImageData {
    pub(crate) fn ext(&self) -> String {
        // TODO we might want to also check the jpeg fsPath
        match self.path.split('.').last() {
            Some(ext) => ext.to_string(),
            None => "bin".to_string(),
        }
    }
}

impl BiffWrite for ImageData {
    fn biff_write(&self, writer: &mut BiffWriter) {
        write(self, writer);
    }
}

impl BiffRead for ImageData {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self {
        read(reader)
    }
}

fn read(reader: &mut BiffReader) -> ImageData {
    let mut name: String = "".to_string();
    let mut inme: Option<String> = None;
    let mut height: u32 = 0;
    let mut width: u32 = 0;
    let mut path: String = "".to_string();
    let mut alpha_test_value: f32 = 0.0;
    let mut jpeg: Option<ImageDataJpeg> = None;
    let mut bits: Option<ImageDataBits> = None;
    let mut link: Option<u32> = None;
    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();
        match tag_str {
            "NAME" => {
                name = reader.get_string();
            }
            "PATH" => {
                path = reader.get_string();
            }
            "INME" => {
                inme = Some(reader.get_string());
            }
            "WDTH" => {
                width = reader.get_u32();
            }
            "HGHT" => {
                height = reader.get_u32();
            }
            "ALTV" => {
                alpha_test_value = reader.get_f32();
            }
            "BITS" => {
                // these have zero as length
                // read all the data until the next expected tag
                let data = reader.data_until("ALTV".as_bytes());
                //let reader = std::io::Cursor::new(data);

                // uncompressed = zlib.decompress(image_data.data[image_data.pos:]) #, wbits=9)
                // reader.skip_end_tag(len.try_into().unwrap());
                bits = Some(ImageDataBits { data });
            }
            "JPEG" => {
                // these have zero as length
                // Strangely, raw data are pushed outside of the JPEG tag (breaking the BIFF structure of the file)
                let mut sub_reader = reader.child_reader();
                let jpeg_data = read_jpeg(&mut sub_reader);
                jpeg = Some(jpeg_data);
                let pos = sub_reader.pos();
                reader.skip_end_tag(pos);
            }
            "LINK" => {
                // TODO seems to be 1 for some kind of link type img, related to screenshots.
                // we only see this where a screenshot is set on the table info.
                // https://github.com/vpinball/vpinball/blob/1a70aa35eb57ec7b5fbbb9727f6735e8ef3183e0/Texture.cpp#L588
                link = Some(reader.get_u32());
            }
            _ => {
                println!("Skipping image tag: {}", tag);
                reader.skip_tag();
            }
        }
    }
    ImageData {
        name,
        inme,
        path,
        width,
        height,
        link,
        alpha_test_value,
        jpeg,
        bits,
    }
}

fn write(data: &ImageData, writer: &mut BiffWriter) {
    writer.write_tagged_string("NAME", &data.name);
    if let Some(inme) = &data.inme {
        writer.write_tagged_string("INME", inme);
    }
    writer.write_tagged_string("PATH", &data.path);
    writer.write_tagged_u32("WDTH", data.width);
    writer.write_tagged_u32("HGHT", data.height);
    if let Some(link) = data.link {
        writer.write_tagged_u32("LINK", link);
    }
    if let Some(bits) = &data.bits {
        writer.write_tagged_data("BITS", &bits.data);
    }
    if let Some(jpeg) = &data.jpeg {
        let bits = write_jpg(jpeg);
        writer.write_tagged_data("JPEG", &bits);
    }
    writer.write_tagged_f32("ALTV", data.alpha_test_value);
    writer.close(true);
}

fn read_jpeg(reader: &mut BiffReader) -> ImageDataJpeg {
    // I do wonder why all the tags are duplicated here
    let mut size_opt: Option<u32> = None;
    let mut path: String = "".to_string();
    let mut name: String = "".to_string();
    let mut data: Vec<u8> = vec![];
    // let mut alpha_test_value: f32 = 0.0;
    let mut inme: Option<String> = None;
    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();
        match tag_str {
            "SIZE" => {
                size_opt = Some(reader.get_u32());
            }
            "DATA" => match size_opt {
                Some(size) => data = reader.get_data(size.try_into().unwrap()).to_vec(),
                None => {
                    panic!("DATA tag without SIZE tag");
                }
            },
            "NAME" => name = reader.get_string(),
            "PATH" => path = reader.get_string(),
            // "ALTV" => alpha_test_value = reader.get_f32(), // TODO why are these duplicated?
            "INME" => inme = Some(reader.get_string()),
            _ => {
                // skip this record
                println!("skipping tag inside JPEG {}", tag);
                reader.skip_tag();
            }
        }
    }
    let data = data.to_vec();
    ImageDataJpeg {
        path,
        name,
        inme,
        // alpha_test_value,
        data,
    }
}

fn write_jpg(img: &ImageDataJpeg) -> Vec<u8> {
    let mut writer = BiffWriter::new();
    writer.write_tagged_string("NAME", &img.name);
    if let Some(inme) = &img.inme {
        writer.write_tagged_string("INME", inme);
    }
    writer.write_tagged_string("PATH", &img.path);
    writer.write_tagged_u32("SIZE", img.data.len().try_into().unwrap());
    writer.write_tagged_data("DATA", &img.data);
    // writer.write_tagged_f32("ALTV", img.alpha_test_value);
    writer.close(true);
    writer.get_data().to_vec()
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_write_read_jpeg() {
        let img = ImageDataJpeg {
            path: "path_value".to_string(),
            name: "name_value".to_string(),
            inme: Some("inme_value".to_string()),
            // alpha_test_value: 1.0,
            data: vec![1, 2, 3],
        };

        let bytes = write_jpg(&img);

        let read = read_jpeg(&mut BiffReader::new(&bytes));

        assert_eq!(read, img);
    }

    #[test]
    fn test_write_read() {
        let image: ImageData = ImageData {
            name: "name_value".to_string(),
            inme: Some("inme_value".to_string()),
            path: "path_value".to_string(),
            width: 1,
            height: 2,
            link: None,
            alpha_test_value: 1.0,
            jpeg: Some(ImageDataJpeg {
                path: "path_value".to_string(),
                name: "name_value".to_string(),
                inme: Some("inme_value".to_string()),
                // alpha_test_value: 1.0,
                data: vec![1, 2, 3],
            }),
            bits: None,
        };
        let mut writer = BiffWriter::new();
        ImageData::biff_write(&image, &mut writer);
        let image_read = read(&mut BiffReader::new(writer.get_data()));
        assert_eq!(image, image_read);
    }
}
