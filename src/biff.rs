use std::str::from_utf8;

use nom::bytes::streaming::take;
use nom::combinator::map;
use nom::multi::many0;
use nom::number::complete::le_f32;
use nom::{number::complete::le_u32, IResult};
use utf16string::WStr;

// TODO make private
/**
 * All records have a tag, eg CODE or NAME
 */
pub const RECORD_TAG_LEN: u32 = 4;

pub fn read_tag_start(input: &[u8]) -> IResult<&[u8], (&str, u32)> {
    let (input, len) = le_u32(input)?;
    let (input, name_bytes) = take(4u8)(input)?;
    let tag = from_utf8(name_bytes).unwrap();
    let n_rest = len - RECORD_TAG_LEN;
    Ok((input, (tag, n_rest)))
}

pub fn read_string_record(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, len) = le_u32(input)?;
    let (input, data) = take(len)(input)?;
    // should probably use latin_1
    let string = from_utf8(data).unwrap();
    Ok((input, string))
}

pub fn read_u32_record(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, data) = le_u32(input)?;
    Ok((input, data))
}

pub fn read_float_record(input: &[u8]) -> IResult<&[u8], (&str, f32)> {
    let (input, n) = le_u32(input)?;
    let n_rest = n - RECORD_TAG_LEN;
    assert!(n_rest == 4, "A float record should be 4 bytes long");
    let (input, name_bytes) = take(4u8)(input)?;
    let (input, data) = le_f32(input)?;
    //let string = String::from_utf8(chars.to_vec()).unwrap();
    let name = from_utf8(name_bytes).unwrap();
    Ok((input, (name, data)))
}

pub fn read_tag_record(len: u32) {
    let n_rest = len - RECORD_TAG_LEN;
    assert!(n_rest == 0, "a tag should have not have any data");
}

pub fn read_tag_record2(len: u32) {
    assert!(len == 0, "a tag should have not have any data");
}

pub fn read_wide_string_record(input: &[u8], _len: u32) -> IResult<&[u8], String> {
    let (input, len) = le_u32(input)?;
    let (input, data) = take(len)(input)?;
    // hmm, this ? seems to be different for nom and utf16string
    // see https://docs.rs/utf16string/latest/utf16string/
    let string = WStr::from_utf16le(data).unwrap().to_utf8();
    Ok((input, string))
}
