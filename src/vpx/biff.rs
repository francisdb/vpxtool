use std::str::from_utf8;

use nom::bytes::streaming::take;
use nom::number::complete::{
    le_f32, le_f64, le_i16, le_i32, le_i64, le_u16, le_u32, le_u64, le_u8,
};
use nom::{IResult, ToUsize};
use utf16string::WStr;

pub struct BiffReader<'a> {
    data: &'a [u8],
    pos: usize,
    bytes_in_record_remaining: usize,
    record_start: usize,
    tag: String,
}
// TODO make private
/**
 * All records have a tag, eg CODE or NAME
 */
pub const RECORD_TAG_LEN: u32 = 4;

pub const WARN: bool = true;

impl<'a> BiffReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let reader: BiffReader<'a> = BiffReader {
            data,
            pos: 0,
            bytes_in_record_remaining: 0,
            record_start: 0,
            tag: "".to_string(),
        };
        reader
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn is_eof(&mut self) -> bool {
        self.pos >= self.data.len() || self.tag == "ENDB"
    }

    pub fn get(&mut self, count: usize) -> &[u8] {
        self.bytes_in_record_remaining -= count;
        self.get_no_remaining_update(count)
    }

    pub fn get_no_remaining_update(&mut self, count: usize) -> &[u8] {
        let p = self.pos;
        self.pos += count;
        &self.data[p..p + count]
    }

    pub fn get_bool(&mut self) -> bool {
        let b = self.data[self.pos] != 0;
        self.pos += 4;
        self.bytes_in_record_remaining -= 4;
        b
    }

    pub fn get_u8(&mut self) -> u8 {
        let i = self.data[self.pos];
        self.pos += 1;
        self.bytes_in_record_remaining -= 1;
        i
    }

    pub fn get_u16(&mut self) -> u16 {
        let i: Result<(&[u8], u16), nom::Err<()>> = le_u16(&self.data[self.pos..]);
        self.pos += 2;
        self.bytes_in_record_remaining -= 2;
        i.unwrap().1
    }

    pub fn get_u32(&mut self) -> u32 {
        let res = self.get_u32_no_remaining_update();
        self.bytes_in_record_remaining -= 4;
        res
    }

    pub fn get_u32_no_remaining_update(&mut self) -> u32 {
        let i: Result<(&[u8], u32), nom::Err<()>> = le_u32(&self.data[self.pos..]);
        self.pos += 4;
        i.unwrap().1
    }

    pub fn get_32(&mut self) -> i32 {
        let i: Result<(&[u8], i32), nom::Err<()>> = le_i32(&self.data[self.pos..]);
        self.pos += 4;
        self.bytes_in_record_remaining -= 4;
        i.unwrap().1
    }

    pub fn get_float(&mut self) -> f32 {
        let i: Result<(&[u8], f32), nom::Err<()>> = le_f32(&self.data[self.pos..]);
        self.pos += 4;
        self.bytes_in_record_remaining -= 4;
        i.unwrap().1
    }

    pub fn get_str(&mut self, count: usize) -> String {
        let mut pos_0 = count;
        for p in 0..count {
            if self.data[self.pos + p] == 0 {
                pos_0 = p;
                break;
            }
        }
        let s = from_utf8(&self.data[self.pos..self.pos + pos_0]).unwrap();
        self.pos += count;
        self.bytes_in_record_remaining -= count;
        s.to_string()
    }

    pub fn get_string(&mut self) -> String {
        let size = self.get_u32().to_usize();
        self.get_str(size)
    }

    pub fn get_wide_string(&mut self) -> String {
        let count = self.get_u32().to_usize();
        let i = from_utf8(&self.data[self.pos..self.pos + count]).unwrap();
        self.pos += count;
        self.bytes_in_record_remaining -= count;
        i.to_string()
    }

    pub fn get_color(&mut self, has_alpha: bool) -> (f32, f32, f32, f32) {
        if has_alpha {
            (
                self.get_u8() as f32 / 255.0,
                self.get_u8() as f32 / 255.0,
                self.get_u8() as f32 / 255.0,
                self.get_u8() as f32 / 255.0,
            )
        } else {
            (
                self.get_u8() as f32 / 255.0,
                self.get_u8() as f32 / 255.0,
                self.get_u8() as f32 / 255.0,
                1.0,
            )
        }
    }

    pub fn get_double(&mut self) -> f64 {
        let i: Result<(&[u8], f64), nom::Err<()>> = le_f64(&self.data[self.pos..]);
        self.pos += 8;
        self.bytes_in_record_remaining -= 8;
        i.unwrap().1
    }

    pub fn get_i16(&mut self) -> i16 {
        let i: Result<(&[u8], i16), nom::Err<()>> = le_i16(&self.data[self.pos..]);
        self.pos += 2;
        self.bytes_in_record_remaining -= 2;
        i.unwrap().1
    }

    pub fn get_i32(&mut self) -> i32 {
        let i: Result<(&[u8], i32), nom::Err<()>> = le_i32(&self.data[self.pos..]);
        self.pos += 4;
        self.bytes_in_record_remaining -= 4;
        i.unwrap().1
    }

    pub fn get_i64(&mut self) -> i64 {
        let i: Result<(&[u8], i64), nom::Err<()>> = le_i64(&self.data[self.pos..]);
        self.pos += 8;
        self.bytes_in_record_remaining -= 8;
        i.unwrap().1
    }

    pub fn get_u64(&mut self) -> u64 {
        let i: Result<(&[u8], u64), nom::Err<()>> = le_u64(&self.data[self.pos..]);
        self.pos += 8;
        self.bytes_in_record_remaining -= 8;
        i.unwrap().1
    }

    pub fn get_u32_array(&mut self, count: usize) -> Vec<u32> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.get_u32());
        }
        v
    }

    pub fn get_u16_array(&mut self, count: usize) -> Vec<u16> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.get_u16());
        }
        v
    }

    pub fn get_i16_array(&mut self, count: usize) -> Vec<i16> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.get_i16());
        }
        v
    }

    pub fn get_i32_array(&mut self, count: usize) -> Vec<i32> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.get_i32());
        }
        v
    }

    pub fn get_i64_array(&mut self, count: usize) -> Vec<i64> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.get_i64());
        }
        v
    }

    pub fn get_u64_array(&mut self, count: usize) -> Vec<u64> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.get_u64());
        }
        v
    }

    pub fn get_f32_array(&mut self, count: usize) -> Vec<f32> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.get_float());
        }
        v
    }

    pub fn get_f64_array(&mut self, count: usize) -> Vec<f64> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.get_double());
        }
        v
    }

    pub fn get_string_array(&mut self, count: usize) -> Vec<String> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.get_string().to_string());
        }
        v
    }

    pub fn get_record_data(&mut self, with_tag: bool) -> &[u8] {
        let d = if with_tag {
            &self.data[self.pos - 4..self.pos + self.bytes_in_record_remaining]
        } else {
            &self.data[self.pos..self.pos + self.bytes_in_record_remaining]
        };
        self.pos += self.bytes_in_record_remaining;
        self.bytes_in_record_remaining = 0;
        d
    }

    pub fn skip(&mut self, count: usize) {
        self.pos += count;
        self.bytes_in_record_remaining -= count;
    }

    pub fn skip_tag(&mut self) {
        self.pos += self.bytes_in_record_remaining;
        self.bytes_in_record_remaining = 0;
    }

    pub fn next(&mut self, warn: bool) {
        if self.bytes_in_record_remaining > 0 {
            if warn {
                println!(
                    "{} : {} unread octets",
                    self.tag, self.bytes_in_record_remaining
                );
            }
            self.skip(self.bytes_in_record_remaining);
        }
        self.record_start = self.pos;
        self.bytes_in_record_remaining = self.get_u32_no_remaining_update().to_usize();
        let tag = self.get_str(RECORD_TAG_LEN as usize);
        self.tag = tag;
    }
}


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
