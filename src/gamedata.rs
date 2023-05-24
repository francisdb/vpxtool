#![allow(dead_code)]

use std::str::from_utf8;

use nom::bytes::streaming::take;
use nom::combinator::map;
use nom::multi::many0;
use nom::{number::complete::le_u32, IResult};

use crate::biff::{
    read_float_record, read_string_record, read_tag_record, read_u32,
    read_wide_string_record, RECORD_TAG_LEN,
};

#[derive(Debug)]
pub struct GameData {
    left: f32,
    topx: f32,
    rght: f32,
    botm: f32,
}

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
    SoundsSize(u32),
    Unknonw { name: String, data: Vec<u8> },
    End,
}

fn read_record<T>(
    input: &[u8],
    f: fn(String, u32, &[u8]) -> IResult<&[u8], T>,
) -> IResult<&[u8], T> {
    let (input, len) = le_u32(input)?;
    let (input, name_bytes) = take(4u8)(input)?;
    let tag = from_utf8(name_bytes).unwrap();
    f(tag.to_string(), len, input)
}

fn read_gamedata_record(input: &[u8]) -> IResult<&[u8], Record> {
    read_record(input, read_gamedata_record_value)
}

fn read_gamedata_record_value(tag: String, len: u32, input: &[u8]) -> IResult<&[u8], Record> {
    match tag.as_str() {
        "LEFT" => {
            // let (rest, n) = read_u32_record(input)?;
            // Ok((rest, Record::PlayfieldLeft(n)))
            map(read_u32, Record::PlayfieldLeft)(input)
        }
        "TOPX" => map(read_u32, Record::PlayfieldTopX)(input),
        "RGHT" => map(read_u32, Record::PlayfieldRight)(input),
        "BOTM" => map(read_u32, Record::PlayfieldBottom)(input),
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
            let (rest, n) = read_u32(input)?;
            let rec = Record::MaterialsSize(n);
            Ok((rest, rec))
        }
        "SSND" => {
            let (rest, n) = read_u32(input)?;
            let rec = Record::SoundsSize(n);
            Ok((rest, rec))
        }
        "SIMG" => {
            let (rest, n) = read_u32(input)?;
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

pub fn read_all_gamedata_records(input: &[u8]) -> IResult<&[u8], Vec<Record>> {
    many0(read_gamedata_record)(input)
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
