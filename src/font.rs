use nom::{bytes::complete::take, IResult};

use crate::biff::{read_empty_tag, read_string_record, read_tag_start, read_u32};

#[derive(Debug)]
pub struct FontData {
    pub name: String,
    pub path: String, // patho of original file for easy re-importing
    pub data: Vec<u8>,
}

impl FontData {
    pub(crate) fn ext(&self) -> String {
        // TODO we might want to also check the jpeg fsPath
        match self.path.split('.').last() {
            Some(ext) => ext.to_string(),
            None => "bin".to_string(),
        }
    }
}

pub fn read(input: &[u8]) -> IResult<&[u8], FontData> {
    let mut input = input;
    let mut name: String = "".to_string();
    let mut path: String = "".to_string();
    let mut size_opt: Option<u32> = None;
    let mut data: &[u8] = &[];
    while !input.is_empty() {
        let (i, (tag, len)) = read_tag_start(input)?;
        input = match tag {
            "NAME" => {
                let (i, string) = read_string_record(i)?;
                name = string.to_owned();
                i
            }
            "PATH" => {
                let (i, string) = read_string_record(i)?;
                path = string.to_owned();
                i
            }
            "SIZE" => {
                let (i, num) = read_u32(i)?;
                size_opt = Some(num);
                i
            }
            "DATA" => match size_opt {
                Some(size) => {
                    let (i, d) = take(size)(i)?;
                    data = d;
                    i
                }
                None => {
                    panic!("DATA tag without SIZE tag");
                }
            },
            "ENDB" => {
                // ENDB is just a tag, it should have a remaining length of 0
                // dbg!(tag, len);
                let (i, _) = read_empty_tag(i, len)?;
                i
            }
            _ => {
                println!("Skipping font tag: {} len: {}", tag, len);
                let (i, _) = take(len)(i)?;
                i
            }
        }
    }
    let rest = &[];
    let data = data.to_vec();
    Ok((rest, FontData { name, path, data }))
}
