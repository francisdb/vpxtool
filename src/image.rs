use std::str::from_utf8;

use nom::bytes::complete::take;
use nom::combinator::map;
use nom::multi::many0;
use nom::{number::complete::le_u32, IResult};
use utf16string::WStr;

use crate::biff::{
    read_empty_tag, read_float, read_float_record, read_string_record, read_tag_record,
    read_tag_start, read_u32_record, RECORD_TAG_LEN,
};

#[derive(Debug)]
pub struct ImageData {
    /**
     * Original path of the image in the vpx file
     */
    fsPath: String,
    name: String,
    /**
     * Lowercased name?
     */
    inme: String,
    path: String,
    width: u32,
    height: u32,
    alpha_test_value: f32,
    data: Vec<u8>,
}

pub fn read(fsPath: String, input: &[u8]) -> IResult<&[u8], ImageData> {
    let mut input = input;
    let mut name: String = "".to_string();
    let mut inme: String = "".to_string();
    let mut height: u32 = 0;
    let mut width: u32 = 0;
    let mut path: String = "".to_string();
    let mut alpha_test_value: f32 = 0.0;
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
                read_empty_tag(len);
                i
            }
            "BITS" => {
                // TODO how is this encoded?
                dbg!(tag, len);
                let data = i;
                println!("got BITS, skipping remaining data: {} bytes", data.len());
                &[]
            }
            "JPEG" => {
                // TODO read the stream, can also be png/webp/...
                dbg!(tag, len);

                // sub_data.next()
                // if sub_data.tag == 'SIZE':
                //     size = sub_data.get_u32()
                // elif sub_data.tag == 'DATA':
                //     data = sub_data.get(size)
                // elif sub_data.tag == 'NAME':
                //     sub_data.skip_tag()
                // elif sub_data.tag == 'PATH':
                //     path = sub_data.get_string()
                // else:
                //     sub_data.skip_tag()

                let (i, _) = take(len)(i)?;
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
            data: vec![],
        },
    ))
    // while input is not empty consume 4 bytes in a loop
}
