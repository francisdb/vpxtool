use std::str::from_utf8;

use nom::bytes::streaming::take;
use nom::number::complete::{le_f32, le_u16, le_u8};
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

pub fn read_string_record(input: &[u8]) -> IResult<&[u8], String> {
    let (input, len) = le_u32(input)?;
    let (input, data) = take(len)(input)?;
    // should probably use latin_1?
    // use encoding_rs::WINDOWS_1252;
    // TODO this fails for "Spider-Man Classic_VPWmod_V1.0.1.vpx"
    // let string = from_utf8(data).unwrap();
    let string = String::from_utf8_lossy(data);
    Ok((input, string.to_string()))
}

pub fn read_bytes_record(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, len) = le_u32(input)?;
    take(len)(input)
}

pub fn read_byte(input: &[u8]) -> IResult<&[u8], u8> {
    le_u8(input)
}

pub fn read_u32(input: &[u8]) -> IResult<&[u8], u32> {
    le_u32(input)
}

pub fn read_u16(input: &[u8]) -> IResult<&[u8], u16> {
    le_u16(input)
}

pub fn read_float_record(input: &[u8]) -> IResult<&[u8], (&str, f32)> {
    let (input, n) = le_u32(input)?;
    let n_rest = n - RECORD_TAG_LEN;
    let (input, name_bytes) = take(4u8)(input)?;
    let name = from_utf8(name_bytes).unwrap();
    let (input, _data) = le_f32(input)?;
    // TODO does data always have the same value and do we want to add an assertion?
    let (input, f) = read_float(input, n_rest)?;
    Ok((input, (name, f)))
}

pub fn read_float(input: &[u8], n_rest: u32) -> IResult<&[u8], f32> {
    assert!(n_rest == 4, "A float record should be 4 bytes long");
    let (input, data) = le_f32(input)?;
    //let string = String::from_utf8(chars.to_vec()).unwrap();
    Ok((input, data))
}

pub fn read_tag_record(len: u32) {
    let n_rest = len - RECORD_TAG_LEN;
    assert!(n_rest == 0, "a tag should have not have any data");
}

pub fn read_empty_tag(input: &[u8], len: u32) -> IResult<&[u8], ()> {
    assert!(len == 0, "a tag should have not have any data");
    Ok((input, ()))
}

pub fn read_wide_string_record(input: &[u8], _len: u32) -> IResult<&[u8], String> {
    let (input, len) = le_u32(input)?;
    let (input, data) = take(len)(input)?;
    // hmm, this ? seems to be different for nom and utf16string
    // see https://docs.rs/utf16string/latest/utf16string/
    let string = WStr::from_utf16le(data).unwrap().to_utf8();
    Ok((input, string))
}

pub fn drop_record(input: &[u8], len: u32) -> IResult<&[u8], ()> {
    let (input, _) = take(len)(input)?;
    Ok((input, ()))
}
