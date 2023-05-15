use std::fmt;
use std::str::from_utf8;

use nom::bytes::complete::take;
use nom::combinator::map;
use nom::multi::many0;
use nom::{number::complete::le_u32, IResult};
use utf16string::WStr;

use crate::biff::{
    drop_record, read_empty_tag, read_float, read_float_record, read_string_record,
    read_tag_record, read_tag_start, read_u32_record, RECORD_TAG_LEN,
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

#[derive(Debug)]
pub struct ImageData {
    /**
     * Original path of the image in the vpx file
     */
    fsPath: String,
    pub name: String,
    /**
     * Lowercased name?
     */
    inme: String,
    path: String,
    width: u32,
    height: u32,
    alpha_test_value: f32,
    pub jpeg: Option<ImageDataJpeg>,
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

pub fn read(fsPath: String, input: &[u8]) -> IResult<&[u8], ImageData> {
    let mut input = input;
    let mut name: String = "".to_string();
    let mut inme: String = "".to_string();
    let mut height: u32 = 0;
    let mut width: u32 = 0;
    let mut path: String = "".to_string();
    let mut alpha_test_value: f32 = 0.0;
    let mut jpeg: Option<ImageDataJpeg> = None;
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
                let (i, num) = read_u32_record(i)?;
                width = num;
                i
            }
            "HGHT" => {
                let (i, num) = read_u32_record(i)?;
                height = num;
                i
            }
            "PATH" => {
                let (i, string) = read_string_record(i)?;
                path = string.to_owned();
                i
            }
            "ALTV" => {
                // not sure what this is?
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
                // TODO how is this
                dbg!(tag, len);
                let data = i;
                println!("got BITS, skipping remaining data: {} bytes", data.len());
                &[]
            }
            "JPEG" => {
                // these are zero length
                let (i, j) = read_jpeg(i)?;
                jpeg = Some(j);
                i
            }
            _ => {
                // skip this record
                dbg!(tag, len);
                let (i, _) = take(len)(i)?;
                i
            }
        }
    }
    let rest = &[];
    Ok((
        rest,
        ImageData {
            fsPath,
            name,
            inme,
            path,
            width,
            height,
            alpha_test_value,
            jpeg,
        },
    ))
    // while input is not empty consume 4 bytes in a loop
}

fn read_jpeg(input: &[u8]) -> IResult<&[u8], ImageDataJpeg> {
    // I do wonder why all the tags are duplicated here
    let mut input = input;
    let mut sizeOpt: Option<u32> = None;
    let mut path: String = "".to_string();
    let mut name: String = "".to_string();
    let mut data: &[u8] = &[];
    let mut alpha_test_value: f32 = 0.0;
    let mut inme: String = "".to_string();
    let mut endReached = false;
    while !endReached {
        let (i, (tag, len)) = read_tag_start(input)?;
        input = match tag {
            "SIZE" => {
                let (i, num) = read_u32_record(i)?;
                sizeOpt = Some(num);
                i
            }
            "DATA" => match sizeOpt {
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
                // not sure what this is?
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
                endReached = true;
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
