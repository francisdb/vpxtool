use encoding_rs::mem::{decode_latin1, encode_latin1_lossy};
use nom::bytes::streaming::take;
use nom::number::complete::{
    le_f32, le_f64, le_i16, le_i32, le_i64, le_u16, le_u32, le_u64, le_u8,
};
use nom::{IResult, ToUsize};
use utf16string::WStr;

pub trait BiffRead {
    fn biff_read(reader: &mut BiffReader<'_>) -> Self;
}

pub trait BiffWrite {
    fn biff_write(item: &Self, writer: &mut BiffWriter);
}

pub struct BiffReader<'a> {
    data: &'a [u8],
    pos: usize,
    bytes_in_record_remaining: usize,
    record_start: usize,
    tag: String,
    warn_remaining: bool,
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
            warn_remaining: true,
        };
        reader
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn tag(&self) -> String {
        self.tag.to_string()
    }

    pub fn is_eof(&self) -> bool {
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
        let i = self.get_u8_no_remaining_update();
        self.bytes_in_record_remaining -= 1;
        i
    }

    pub fn get_u8_no_remaining_update(&mut self) -> u8 {
        let i = self.data[self.pos];
        self.pos += 1;
        i
    }

    pub fn get_u16(&mut self) -> u16 {
        let res = self.get_u16_no_remaining_update();
        self.bytes_in_record_remaining -= 2;
        res
    }

    pub fn get_u16_no_remaining_update(&mut self) -> u16 {
        let i: Result<(&[u8], u16), nom::Err<()>> = le_u16(&self.data[self.pos..]);
        self.pos += 2;
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
        let res = self.get_32_no_remaining_update();
        self.bytes_in_record_remaining -= 4;
        res
    }
    pub fn get_32_no_remaining_update(&mut self) -> i32 {
        let i: Result<(&[u8], i32), nom::Err<()>> = le_i32(&self.data[self.pos..]);
        self.pos += 4;
        i.unwrap().1
    }

    pub fn get_f32(&mut self) -> f32 {
        let i: Result<(&[u8], f32), nom::Err<()>> = le_f32(&self.data[self.pos..]);
        self.pos += 4;
        self.bytes_in_record_remaining -= 4;
        i.unwrap().1
    }

    pub fn get_str(&mut self, count: usize) -> String {
        let mut pos_0 = count;
        // find the end of the string
        for p in 0..count {
            if self.data[self.pos + p] == 0 {
                pos_0 = p;
                break;
            }
        }
        let data = &self.data[self.pos..self.pos + pos_0];
        let s = decode_latin1(data);
        self.pos += count;
        self.bytes_in_record_remaining -= count;
        s.to_string()
    }

    pub fn get_str_no_remaining_update(&mut self, count: usize) -> String {
        let mut pos_0 = count;
        // find the end of the string
        for p in 0..count {
            if self.data[self.pos + p] == 0 {
                pos_0 = p;
                break;
            }
        }
        let data = &self.data[self.pos..self.pos + pos_0];

        let s = decode_latin1(data);
        self.pos += count;
        s.to_string()
    }

    pub fn get_string(&mut self) -> String {
        let size = self.get_u32().to_usize();
        self.get_str(size)
    }

    pub fn get_wide_string(&mut self) -> String {
        let count = self.get_u32().to_usize();
        let data = &self.data[self.pos..self.pos + count];
        // hmm, this ? seems to be different for nom and utf16string
        // see https://docs.rs/utf16string/latest/utf16string/
        let i = WStr::from_utf16le(data).unwrap().to_utf8();
        self.pos += count;
        self.bytes_in_record_remaining -= count;
        i
    }

    #[deprecated]
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
            v.push(self.get_f32());
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

    pub fn get_data(&mut self, count: usize) -> &[u8] {
        let d = &self.data[self.pos..self.pos + count];
        self.pos += count;
        self.bytes_in_record_remaining = 0;
        d
    }

    pub fn skip(&mut self, count: usize) {
        self.pos += count;
        self.bytes_in_record_remaining -= count;
    }

    pub fn skip_end_tag(&mut self, count: usize) {
        self.pos += count;
        self.bytes_in_record_remaining = 0;
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
        if self.pos >= self.data.len() {
            panic!(
                "Unexpected end of biff stream at {}/{} while reading next tag. Missing ENDB?",
                self.pos(),
                self.data.len()
            );
        }
        self.bytes_in_record_remaining = self.get_u32_no_remaining_update().to_usize();
        let tag = self.get_str(RECORD_TAG_LEN.try_into().unwrap());
        self.tag = tag;
        if self.warn_remaining && self.tag == "ENDB" && self.pos < self.data.len() {
            panic!("{} Remaining bytes after ENDB", self.data.len() - self.pos);
        }
    }

    pub fn child_reader(&mut self) -> BiffReader {
        BiffReader {
            data: &self.data[self.pos..],
            pos: 0,
            bytes_in_record_remaining: 0,
            record_start: 0,
            tag: "".to_string(),
            warn_remaining: false,
        }
    }
}

pub struct BiffWriter {
    data: Vec<u8>,
    tag_start: usize,
    tag: String,
    record_size: usize,
}

impl Default for BiffWriter {
    fn default() -> Self {
        BiffWriter {
            data: Vec::new(),
            tag_start: 0,
            tag: "".to_string(),
            record_size: 0,
        }
    }
}

impl BiffWriter {
    pub fn new() -> BiffWriter {
        BiffWriter::default()
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn end_tag(&mut self) {
        if !self.tag.is_empty() {
            //let length = self.data.len();
            let length: &u32 = &self.record_size.try_into().unwrap();
            let length_bytes = length.to_le_bytes();
            self.data[self.tag_start..self.tag_start + 4].copy_from_slice(&length_bytes);
            self.tag = "".to_string();
        }
    }

    pub fn end_tag_no_size(&mut self) {
        if !self.tag.is_empty() {
            let length: u32 = 4;
            let length_bytes = length.to_le_bytes();
            self.data[self.tag_start..self.tag_start + 4].copy_from_slice(&length_bytes);
            self.tag = "".to_string();
        }
    }

    pub fn new_tag(&mut self, tag: &str) {
        self.end_tag();
        self.tag_start = self.data.len();
        self.data.extend_from_slice(&[0, 0, 0, 0]); // placeholder for record size
        let tag_bytes = tag.as_bytes();
        // some tags are smaller than 4 characters, so we need to pad them
        let mut padded_tag_bytes = [0; 4];
        padded_tag_bytes[..tag_bytes.len()].copy_from_slice(tag_bytes);
        self.data.extend_from_slice(&padded_tag_bytes);
        self.tag = tag.to_string();
        self.record_size = 4;
    }

    pub fn write_u8(&mut self, value: u8) {
        self.record_size += 1;
        self.data.push(value);
    }

    pub fn write_8(&mut self, value: i8) {
        self.record_size += 1;
        self.data.push(value as u8);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.record_size += 2;
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_16(&mut self, value: i16) {
        self.record_size += 2;
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.record_size += 4;
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_32(&mut self, value: i32) {
        self.record_size += 4;
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_f32(&mut self, value: f32) {
        self.record_size += 4;
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_short_string(&mut self, value: &str) {
        let d = encode_latin1_lossy(value);
        self.write_u8(d.len().try_into().unwrap());
        self.write_data(&d);
    }

    pub fn write_string(&mut self, value: &str) {
        let d = encode_latin1_lossy(value);
        self.write_u32(d.len().try_into().unwrap());
        self.write_data(&d);
    }

    pub fn write_wide_string(&mut self, value: &str) {
        // utf-16-le encode as u8
        let d = value
            .encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect::<Vec<u8>>();
        self.write_u32(d.len().try_into().unwrap());
        self.write_data(&d);
    }

    pub fn write_bool(&mut self, value: bool) {
        if value {
            self.write_u32(0xFFFFFFFF);
        } else {
            self.write_u32(0x00000000);
        }
    }

    pub fn write_data(&mut self, value: &[u8]) {
        self.record_size += value.len();
        self.data.extend_from_slice(value);
    }

    pub fn write_tagged_empty(&mut self, tag: &str) {
        self.new_tag(tag);
        self.end_tag();
    }

    pub fn write_tagged_bool(&mut self, tag: &str, value: bool) {
        self.new_tag(tag);
        self.write_bool(value);
        self.end_tag();
    }

    pub fn write_tagged_f32(&mut self, tag: &str, value: f32) {
        self.new_tag(tag);
        self.write_f32(value);
        self.end_tag();
    }

    pub fn write_tagged_u32(&mut self, tag: &str, value: u32) {
        self.new_tag(tag);
        self.write_u32(value);
        self.end_tag();
    }

    pub fn write_tagged_i32(&mut self, tag: &str, value: i32) {
        self.new_tag(tag);
        self.write_32(value);
        self.end_tag();
    }

    pub fn write_tagged_string(&mut self, tag: &str, value: &str) {
        self.new_tag(tag);
        self.write_string(value);
        self.end_tag();
    }

    pub fn write_tagged_string_no_size(&mut self, tag: &str, value: &str) {
        self.new_tag(tag);
        self.write_string(value);
        self.end_tag_no_size();
    }

    pub fn write_tagged_wide_string(&mut self, tag: &str, value: &str) {
        self.new_tag(tag);
        self.write_wide_string(value);
        self.end_tag();
    }

    pub fn write_tagged_vec2(&mut self, tag: &str, x: f32, y: f32) {
        self.new_tag(tag);
        self.write_f32(x);
        self.write_f32(y);
        self.end_tag();
    }

    pub fn write_tagged_padded_vector(&mut self, tag: &str, x: f32, y: f32, z: f32) {
        self.new_tag(tag);
        self.write_f32(x);
        self.write_f32(y);
        self.write_f32(z);
        self.write_f32(0.0);
        self.end_tag();
    }

    pub fn write_tagged_data(&mut self, tag: &str, value: &[u8]) {
        self.new_tag(tag);
        self.write_data(value);
        self.end_tag();
    }

    pub fn write_tagged<T: BiffWrite>(&mut self, tag: &str, value: &T) {
        self.new_tag(tag);
        BiffWrite::biff_write(value, self);
        self.end_tag();
    }

    pub fn write_tagged_with<T>(&mut self, tag: &str, value: &T, f: fn(&T, &mut BiffWriter) -> ()) {
        self.new_tag(tag);
        f(value, self);
        self.end_tag();
    }

    pub fn close(&mut self, write_endb: bool) {
        if write_endb {
            self.new_tag("ENDB");
        }
        self.end_tag();
    }
}

#[deprecated]
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

#[deprecated]
pub fn read_bytes_record(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, len) = le_u32(input)?;
    take(len)(input)
}

#[deprecated]
pub fn read_byte(input: &[u8]) -> IResult<&[u8], u8> {
    le_u8(input)
}

#[deprecated]
pub fn read_u32(input: &[u8]) -> IResult<&[u8], u32> {
    le_u32(input)
}

#[deprecated]
pub fn read_u16(input: &[u8]) -> IResult<&[u8], u16> {
    le_u16(input)
}
