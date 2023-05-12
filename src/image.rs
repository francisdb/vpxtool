use std::str::from_utf8;

use nom::bytes::complete::take;
use nom::combinator::map;
use nom::multi::many0;
use nom::{number::complete::le_u32, IResult};
use utf16string::WStr;

use crate::biff::{
    read_string_record, read_tag_record, read_tag_record2, read_tag_start, RECORD_TAG_LEN,
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
    data: Vec<u8>,
}

pub fn read(fsPath: String, input: &[u8]) -> IResult<&[u8], ImageData> {
    let mut input = input;
    let mut name: String = "".to_string();
    let mut inme: String = "".to_string();
    let mut path: String = "".to_string();
    while !input.is_empty() {
        let (i, (tag, len)) = read_tag_start(input)?;
        match tag {
            "NAME" => {
                let (i, string) = read_string_record(i)?;
                name = string.to_owned();
                input = i;
            }
            "INME" => {
                let (i, string) = read_string_record(i)?;
                inme = string.to_owned();
                input = i;
            }
            "PATH" => {
                let (i, string) = read_string_record(i)?;
                path = string.to_owned();
                input = i;
            }
            "ENDB" => {
                // ENDB is just a tag, it should have a remaining length of 0
                read_tag_record2(len);
                input = i
            }
            //     elif image_data.tag == 'WDTH':
            //     width = image_data.get_u32()
            // elif image_data.tag == 'HGHT':
            //     height = image_data.get_u32()
            _ => {
                // skip this record
                dbg!(tag, len);
                let (i, _) = take(len)(i)?;
                input = i;
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
            width: 0,
            height: 0,
            data: vec![],
        },
    ))
    // while input is not empty consume 4 bytes in a loop
}
