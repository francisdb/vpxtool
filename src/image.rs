use std::fmt;

use nom::bytes::complete::take;

use nom::IResult;

use crate::biff::{
    drop_record, read_empty_tag, read_float, read_string_record, read_tag_start, read_u32,
};

// #[derive(Debug)]
pub struct ImageDataJpeg {
    path: String,
    name: String,
    /**
     * Lowercased name?
     */
    inme: String,
    alpha_test_value: f32,
    pub data: Vec<u8>,
}

impl fmt::Debug for ImageDataJpeg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // avoid writing the data to the debug output
        f.debug_struct("ImageDataJpeg")
            .field("path", &self.path)
            .field("name", &self.name)
            .field("data", &self.data.len())
            .finish()
    }
}

/**
 * An bitmap blob, typically used by textures.
 */
// #[derive(Debug)]
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

#[derive(Debug)]
pub struct ImageData {
    /**
     * Original path of the image in the vpx file
     * we could probably just keep the index?
     */
    fs_path: String,
    pub name: String,
    /**
     * Lowercased name?
     */
    inme: String,
    path: String,
    width: u32,
    height: u32,
    alpha_test_value: f32,
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

pub fn read(fs_path: String, input: &[u8]) -> IResult<&[u8], ImageData> {
    let mut input = input;
    let mut name: String = "".to_string();
    let mut inme: String = "".to_string();
    let mut height: u32 = 0;
    let mut width: u32 = 0;
    let mut path: String = "".to_string();
    let mut alpha_test_value: f32 = 0.0;
    let mut jpeg: Option<ImageDataJpeg> = None;
    let mut bits: Option<ImageDataBits> = None;
    while !input.is_empty() {
        let (i, (tag, len)) = read_tag_start(input)?;
        input = match tag {
            "NAME" => {
                let (i, string) = read_string_record(i)?;
                name = string.to_owned();
                i
            }
            "INME" => {
                let (i, string) = read_string_record(i)?;
                inme = string.to_owned();
                i
            }
            "WDTH" => {
                let (i, num) = read_u32(i)?;
                width = num;
                i
            }
            "HGHT" => {
                let (i, num) = read_u32(i)?;
                height = num;
                i
            }
            "PATH" => {
                let (i, string) = read_string_record(i)?;
                path = string.to_owned();
                i
            }
            "ALTV" => {
                let (i, f) = read_float(i, len)?;
                alpha_test_value = f;
                i
            }
            "ENDB" => {
                // ENDB is just a tag, it should have a remaining length of 0
                // dbg!(tag, len);
                let (i, _) = read_empty_tag(i, len)?;
                i
            }
            "BITS" => {
                // these have zero as length
                let (i, b) = read_bits(i)?;
                bits = Some(b);
                i
            }
            "JPEG" => {
                // these have zero as length
                let (i, j) = read_jpeg(i)?;
                jpeg = Some(j);
                i
            }
            "LINK" => {
                // TODO seems to be 1 for some kind of link type img, related to screenshots.
                // we only see this where a screenshot is set on the table info.
                // https://github.com/vpinball/vpinball/blob/1a70aa35eb57ec7b5fbbb9727f6735e8ef3183e0/Texture.cpp#L588
                let (i, _link) = read_u32(i)?;
                i
            }
            _ => {
                println!("Skipping image tag: {} len: {}", tag, len);
                let (i, _) = take(len)(i)?;
                i
            }
        }
    }
    let rest = &[];
    Ok((
        rest,
        ImageData {
            fs_path,
            name,
            inme,
            path,
            width,
            height,
            alpha_test_value,
            jpeg,
            bits,
        },
    ))
    // while input is not empty consume 4 bytes in a loop
}

fn read_bits(input: &[u8]) -> IResult<&[u8], ImageDataBits> {
    // let mut input = input;

    // lzw encoded data but probably using some custom format
    // using

    //let (i, len) = read_u32_record(input)?;
    //println!("len: {:?}", len);

    // decode using lzw::Decoder
    // let reader = lzw::MsbReader::new();
    // let mut decoder = lzw::DecoderEarlyChange::new(reader, 8);
    // let (n, decoded) = decoder.decode_bytes(input).unwrap();
    // let mut data: Vec<u8> = vec![];
    // while let Some(byte) = decoder.next() {
    //     data.push(byte);
    // }

    //dbg!(decoded.len(), n);

    println!("dropping remaining bytes for BITS: {:?}", input.len());

    Ok((&[], ImageDataBits { data: vec![] }))
}

fn read_jpeg(input: &[u8]) -> IResult<&[u8], ImageDataJpeg> {
    // I do wonder why all the tags are duplicated here
    let mut input = input;
    let mut size_opt: Option<u32> = None;
    let mut path: String = "".to_string();
    let mut name: String = "".to_string();
    let mut data: &[u8] = &[];
    let mut alpha_test_value: f32 = 0.0;
    let mut inme: String = "".to_string();
    let mut end_reached = false;
    while !end_reached {
        let (i, (tag, len)) = read_tag_start(input)?;
        input = match tag {
            "SIZE" => {
                let (i, num) = read_u32(i)?;
                size_opt = Some(num);
                i
            }
            "DATA" => match size_opt {
                Some(size) => {
                    let (i, d) = take(size)(i)?;
                    data = d;
                    i
                }
                None => {
                    panic!("DATA tag without SIZE tag");
                }
            },
            "NAME" => {
                let (i, string) = read_string_record(i)?;
                name = string.to_owned();
                i
            }
            "PATH" => {
                let (i, string) = read_string_record(i)?;
                path = string.to_owned();
                i
            }
            "ALTV" => {
                let (i, f) = read_float(i, len)?;
                alpha_test_value = f;
                i
            }
            "INME" => {
                let (i, string) = read_string_record(i)?;
                inme = string.to_owned();
                i
            }
            "ENDB" => {
                // ENDB is just a tag, it should have a remaining length of 0
                // dbg!(tag, len);
                let (i, _) = read_empty_tag(i, len)?;
                end_reached = true;
                i
            }
            _ => {
                // skip this record
                println!("skipping tag inside JPEG {} {}", tag, len);
                let (i, _) = drop_record(i, len)?;
                i
            }
        }
    }
    let data = data.to_vec();
    Ok((
        input,
        ImageDataJpeg {
            path,
            name,
            inme,
            alpha_test_value,
            data,
        },
    ))
}
