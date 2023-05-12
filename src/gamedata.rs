#![allow(dead_code)]

use std::str::from_utf8;

use nom::bytes::streaming::take;
use nom::combinator::map;
use nom::multi::many0;
use nom::number::complete::le_f32;
use nom::{number::complete::le_u32, IResult};

use utf16string::WStr;

#[derive(Debug)]
pub struct GameData {
    left: f32,
    topx: f32,
    rght: f32,
    botm: f32,
}

/**
 * All records have a tag, eg CODE or NAME
 */
const RECORD_TAG_LEN: u32 = 4;

#[derive(Debug)]
pub enum Record {
    PlayfieldLeft(u32),
    PlayfieldTopX(u32),
    PlayfieldRight(u32),
    PlayfieldBottom(u32),
    Code { script: String },
    Name(String),
    MaterialsSize(u32),
    ImagesSize(u32),
    Unknonw { name: String, data: Vec<u8> },
    End,
}

fn read_record(input: &[u8]) -> IResult<&[u8], Record> {
    let (input, len) = le_u32(input)?;
    let (input, name_bytes) = take(4u8)(input)?;
    let tag = from_utf8(name_bytes).unwrap();
    //dbg!(tag, len);
    match tag {
        "LEFT" => {
            // let (rest, n) = read_u32_record(input)?;
            // Ok((rest, Record::PlayfieldLeft(n)))
            map(read_u32_record, Record::PlayfieldLeft)(input)
        }
        "TOPX" => map(read_u32_record, Record::PlayfieldTopX)(input),
        "RGHT" => map(read_u32_record, Record::PlayfieldRight)(input),
        "BOTM" => map(read_u32_record, Record::PlayfieldBottom)(input),
        "NAME" => {
            let n_rest = len - RECORD_TAG_LEN;
            let (rest, string) = read_wide_string_record(input, n_rest)?;
            let rec = Record::Name(string.to_string());
            println!("NAME: {}", string);
            Ok((rest, rec))
        }
        "CODE" => {
            let (rest, string) = read_string_record(input)?;
            let rec = Record::Code {
                script: string.to_string(),
            };
            Ok((rest, rec))
        }
        "MASI" => {
            let (rest, n) = read_u32_record(input)?;
            let rec = Record::MaterialsSize(n);
            Ok((rest, rec))
        }
        "SIMG" => {
            let (rest, n) = read_u32_record(input)?;
            let rec = Record::ImagesSize(n);
            Ok((rest, rec))
        }
        "ENDB" => {
            // ENDB is just a tag, it should have a remaining length of 0
            read_tag_record(len);
            Ok((input, Record::End))
        }
        _ => {
            //let string = String::from_utf8(chars.to_vec()).unwrap();
            // the name tag is included in the length
            let n_rest = len - RECORD_TAG_LEN;
            let (rest, data) = take(n_rest)(input)?;
            let rec = Record::Unknonw {
                name: tag.to_owned(),
                data: data.to_owned(),
            };
            Ok((rest, rec))
        }
    }
}

fn read_tag_record(len: u32) {
    let n_rest = len - RECORD_TAG_LEN;
    assert!(n_rest == 0, "a tag should have not have any data");
}

fn read_wide_string_record(input: &[u8], len: u32) -> IResult<&[u8], String> {
    let (input, len) = le_u32(input)?;
    let (input, data) = take(len)(input)?;
    // hmm, this ? seems to be different for nom and utf16string
    // see https://docs.rs/utf16string/latest/utf16string/
    let string = WStr::from_utf16le(data).unwrap().to_utf8();
    Ok((input, string))
}

fn read_string_record(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, len) = le_u32(input)?;
    let (input, data) = take(len)(input)?;
    let string = from_utf8(data).unwrap();
    Ok((input, string))
}

pub fn read_all_records(input: &[u8]) -> IResult<&[u8], Vec<Record>> {
    many0(read_record)(input)
}

fn read_u32_record(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, data) = le_u32(input)?;
    Ok((input, data))
}

fn read_float_record(input: &[u8]) -> IResult<&[u8], (&str, f32)> {
    let (input, n) = le_u32(input)?;
    let n_rest = n - RECORD_TAG_LEN;
    assert!(n_rest == 4, "A float record should be 4 bytes long");
    let (input, name_bytes) = take(4u8)(input)?;
    let (input, data) = le_f32(input)?;
    //let string = String::from_utf8(chars.to_vec()).unwrap();
    let name = from_utf8(name_bytes).unwrap();
    Ok((input, (name, data)))
}

fn read_game_data(input: &[u8]) -> IResult<&[u8], GameData> {
    let (input, n1) = read_float_record(input)?;
    let (input, n2) = read_float_record(input)?;
    let (input, n3) = read_float_record(input)?;
    let (input, n4) = read_float_record(input)?;
    let game_data = GameData {
        left: n1.1,
        topx: n2.1,
        rght: n3.1,
        botm: n4.1,
    };
    Ok((input, game_data))
}
