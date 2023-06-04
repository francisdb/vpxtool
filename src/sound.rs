use std::fmt;

use bytes::{BufMut, BytesMut};

use nom::IResult;

use crate::biff::{read_byte, read_bytes_record, read_string_record, read_u16, read_u32};

const NEW_SOUND_FORMAT_VERSION: u32 = 1031;

/**
 * An bitmap blob, typically used by textures.
 */
// #[derive(Debug)]
pub struct ImageDataBits {
    pub data: Vec<u8>,
}

impl fmt::Debug for SoundData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // avoid writing the data to the debug output
        f.debug_struct("SoundData")
            .field("path", &self.path)
            .field("name", &self.name)
            .field("internal_name", &self.internal_name)
            .field("data", &self.data.len())
            .finish()
    }
}

pub struct SoundData {
    /**
     * Original path of the sound in the vpx file
     * we could probably just keep the index?
     */
    pub fs_path: String,
    pub name: String,
    pub path: String,
    pub wave_form: WaveForm,
    pub data: Vec<u8>,
    // seems to like the images be the lowercase of name
    pub internal_name: String,
    pub fade: u32,
    pub volume: u32,
    pub balance: u32,
    pub output_target: u8,
}

fn write_wav_header(sound_data: &SoundData) -> Vec<u8> {
    let mut buf = BytesMut::with_capacity(44);
    buf.put(&b"RIFF"[..]); // 4
    buf.put_u32_le(sound_data.data.len() as u32 + 36); // 4
    buf.put(&b"WAVE"[..]); // 4
    buf.put(&b"fmt "[..]); // 4
    buf.put_u32_le(16); // 4
    buf.put_u16_le(sound_data.wave_form.format_tag); // 2
    buf.put_u16_le(sound_data.wave_form.channels); // 2
    buf.put_u32_le(sound_data.wave_form.samples_per_sec); // 4
    buf.put_u32_le(
        sound_data.wave_form.samples_per_sec
            * sound_data.wave_form.bits_per_sample as u32
            * sound_data.wave_form.channels as u32
            / 8,
    ); // 4
    buf.put_u16_le(sound_data.wave_form.block_align); // 2
    buf.put_u16_le(sound_data.wave_form.bits_per_sample); // 2
    buf.put(&b"data"[..]); // 4
    buf.put_u32_le(sound_data.data.len() as u32); // 4
    buf.to_vec() // total 44 bytes
}

pub fn write_sound(sound_data: &SoundData) -> Vec<u8> {
    let mut buf = if is_wav(&sound_data.path) {
        let mut buf = BytesMut::with_capacity(44 + sound_data.data.len());
        buf.put_slice(&write_wav_header(sound_data));
        buf
    } else {
        let buf = BytesMut::with_capacity(sound_data.data.len());
        buf
    };
    buf.put_slice(&sound_data.data);
    buf.to_vec()
}

#[derive(Debug)]
pub struct WaveForm {
    // Format type
    format_tag: u16,
    // Number of channels (i.e. mono, stereo...)
    channels: u16,
    // Sample rate
    samples_per_sec: u32,
    // For buffer estimation
    avg_bytes_per_sec: u32,
    // Block size of data
    block_align: u16,
    // Number of bits per sample of mono data
    bits_per_sample: u16,
    // The count in bytes of the size of extra information (after cbSize)
    cb_size: u16,
}

impl WaveForm {
    fn new() -> WaveForm {
        WaveForm {
            format_tag: 1,
            channels: 1,
            samples_per_sec: 44100,
            avg_bytes_per_sec: 88200,
            block_align: 2,
            bits_per_sample: 16,
            cb_size: 0,
        }
    }
}

impl Default for WaveForm {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundData {
    pub(crate) fn ext(&self) -> String {
        // TODO we might want to also check the jpeg fsPath
        match self.path.split('.').last() {
            Some(ext) => ext.to_string(),
            None => "bin".to_string(),
        }
    }
}

pub fn read(fs_path: String, file_version: u32, input: &[u8]) -> IResult<&[u8], SoundData> {
    let mut input = input;
    let mut name: String = "".to_string();
    let mut path: String = "".to_string();
    let mut internal_name: String = "".to_string();
    let mut fade: u32 = 0;
    let mut volume: u32 = 0;
    let mut balance: u32 = 0;
    let mut output_target: u8 = 0;
    let mut data: &[u8] = &[];
    let mut wave_form: WaveForm = WaveForm::new();

    // TODO add support for the old format file version < 1031
    // https://github.com/freezy/VisualPinball.Engine/blob/ec1e9765cd4832c134e889d6e6d03320bc404bd5/VisualPinball.Engine/VPT/Sound/SoundData.cs#L98

    let num_values = if file_version < NEW_SOUND_FORMAT_VERSION {
        5
    } else {
        10
    };

    for i in 0..num_values {
        input = match i {
            0 => {
                let (remaining, n) = read_string_record(input)?;
                name = n.to_string();
                remaining
            }
            1 => {
                let (remaining, p) = read_string_record(input)?;
                path = p.to_string();
                remaining
            }
            2 => {
                let (remaining, n) = read_string_record(input)?;
                internal_name = n.to_string();
                remaining
            }
            3 => {
                if is_wav(&path.to_owned()) {
                    // FormatTag = reader.ReadUInt16();
                    // Channels = reader.ReadUInt16();
                    // SamplesPerSec = reader.ReadUInt32();
                    // AvgBytesPerSec = reader.ReadUInt32();
                    // BlockAlign = reader.ReadUInt16();
                    // BitsPerSample = reader.ReadUInt16();
                    // CbSize = reader.ReadUInt16();
                    let (input, format_tag) = read_u16(input)?;
                    let (input, channels) = read_u16(input)?;
                    let (input, samples_per_sec) = read_u32(input)?;
                    let (input, avg_bytes_per_sec) = read_u32(input)?;
                    let (input, block_align) = read_u16(input)?;
                    let (input, bits_per_sample) = read_u16(input)?;
                    let (input, cb_size) = read_u16(input)?;
                    wave_form = WaveForm {
                        format_tag,
                        channels,
                        samples_per_sec,
                        avg_bytes_per_sec,
                        block_align,
                        bits_per_sample,
                        cb_size,
                    };
                    input
                } else {
                    input
                }
            }
            4 => {
                let (remaining, d) = read_bytes_record(input)?;
                data = d;
                remaining
            }
            5 => {
                let (remaining, t) = read_byte(input)?;
                output_target = t;
                remaining
            }
            6 => {
                let (remaining, v) = read_u32(input)?;
                volume = v;
                remaining
            }
            7 => {
                let (remaining, b) = read_u32(input)?;
                balance = b;
                remaining
            }
            8 => {
                let (remaining, f) = read_u32(input)?;
                fade = f;
                remaining
            }
            9 => {
                // TODO why do we have the volume twice?
                let (remaining, v) = read_u32(input)?;
                volume = v;
                remaining
            }
            unexpected => {
                panic!("unexpected value {}", unexpected);
            }
        }
    }

    Ok((
        input,
        SoundData {
            fs_path,
            name,
            path,
            data: data.to_vec(),
            wave_form,
            internal_name,
            fade,
            volume,
            balance,
            output_target,
        },
    ))
}

fn is_wav(path: &String) -> bool {
    path.to_lowercase().ends_with(".wav")
}
