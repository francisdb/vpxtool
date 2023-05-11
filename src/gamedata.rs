use std::str::from_utf8;

use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    number::complete::le_u32,
    sequence::tuple,
    IResult,
};
use nom::bytes::streaming::take;
use nom::multi::many0;
use nom::number::complete::le_f32;

#[derive(Debug)]
pub struct GameData {
    left: f32,
    topx: f32,
    rght: f32,
    botm: f32,
}

fn parse_record(input: &[u8]) -> IResult<&[u8], (&str, &[u8])> {
    let (input, n) = le_u32(input)?;
    let (input, nameBytes) = take(4u8)(input)?;
    let name = from_utf8(nameBytes).unwrap();

    let (input, len) = if "CODE".eq(name) {
        // for code the length is put after the name
        // maybe because the length is not known upfront?
        let (input, len) = le_u32(input)?;
        println!("!!! {}", len);
        (input, len)
    } else {
        //let string = String::from_utf8(chars.to_vec()).unwrap();
        // the name tag is included in the length
        let n_rest = n - 4;
        (input, n_rest)
    };
    println!("{} {}", name, len);
    let (input, data) = take(len)(input)?;
    Ok((input, (name, data)))
}

pub fn parse_all(input: &[u8]) -> IResult<&[u8], Vec<(&str, &[u8])>> {
    many0(parse_record)(input)
}

fn parse_float(input: &[u8]) -> IResult<&[u8], (&str, f32)> {
    let (input, n) = le_u32(input)?;
    let n_rest = n - 4;
    let (input, nameBytes) = take(4u8)(input)?;
    let (input, data) = le_f32(input)?;
    //let string = String::from_utf8(chars.to_vec()).unwrap();
    let name = from_utf8(nameBytes).unwrap();
    Ok((input, (name, data)))
}

fn parse_game_data(input: &[u8]) -> IResult<&[u8], GameData> {
    let (input, n1) = parse_float(input)?;
    let (input, n2) = parse_float(input)?;
    let (input, n3) = parse_float(input)?;
    let (input, n4) = parse_float(input)?;
    let gameData = GameData {
        left: n1.1,
        topx: n2.1,
        rght: n3.1,
        botm: n4.1,
    };
    Ok((input, gameData))
}