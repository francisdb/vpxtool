#![allow(dead_code)]

use std::str::from_utf8;

use nom::bytes::streaming::take;
use nom::multi::many0;
use nom::number::complete::le_f32;
use nom::{number::complete::le_u32, IResult};

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

fn read_record(input: &[u8]) -> IResult<&[u8], (&str, &[u8])> {
    let (input, n) = le_u32(input)?;
    let (input, name_bytes) = take(4u8)(input)?;
    let name = from_utf8(name_bytes).unwrap();

    let (input, len) = match name {
        "CODE" => {
            // for code the length is put after the name
            // maybe because the length is not known upfront?
            let (input, len) = le_u32(input)?;
            (input, len)
        }
        "ENDB" => {
            // ENDB is just a tag, it should have a remaining length of 0
            let n_rest = n - RECORD_TAG_LEN;
            (input, n_rest)
        }
        _ => {
            //let string = String::from_utf8(chars.to_vec()).unwrap();
            // the name tag is included in the length
            let n_rest = n - RECORD_TAG_LEN;
            (input, n_rest)
        }
    };
    // println!("{} {}", name, len);
    let (input, data) = take(len)(input)?;
    Ok((input, (name, data)))
}

pub fn read_all_records(input: &[u8]) -> IResult<&[u8], Vec<(&str, &[u8])>> {
    many0(read_record)(input)
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
