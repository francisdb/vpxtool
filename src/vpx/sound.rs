use std::fmt;

use bytes::{BufMut, BytesMut};

use super::{
    biff::{BiffReader, BiffWriter},
    Version,
};

const NEW_SOUND_FORMAT_VERSION: u32 = 1031;

/**
 * An bitmap blob, typically used by textures.
 */
#[derive(PartialEq)]
pub struct ImageDataBits {
    pub data: Vec<u8>,
}

impl fmt::Debug for SoundData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // avoid writing the data to the debug output
        f.debug_struct("SoundData")
            .field("name", &self.name)
            .field("path", &self.path)
            .field("wave_form", &self.wave_form)
            .field("data", &self.data.len())
            .field("internal_name", &self.internal_name)
            .field("fade", &self.fade)
            .field("volume", &self.volume)
            .field("balance", &self.balance)
            .field("output_target", &self.output_target)
            .finish()
    }
}

#[derive(PartialEq)]
pub struct SoundData {
    pub name: String,
    pub path: String,
    pub wave_form: WaveForm, // we probably want this to be optional
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
        BytesMut::with_capacity(sound_data.data.len())
    };
    buf.put_slice(&sound_data.data);
    buf.to_vec()
}

#[derive(Debug, PartialEq)]
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

pub(crate) fn read(file_version: &Version, reader: &mut BiffReader) -> SoundData {
    let mut name: String = "".to_string();
    let mut path: String = "".to_string();
    let mut internal_name: String = "".to_string();
    let mut fade: u32 = 0;
    let mut volume: u32 = 0;
    let mut balance: u32 = 0;
    let mut output_target: u8 = 0;
    let mut data: Vec<u8> = Vec::new();
    let mut wave_form: WaveForm = WaveForm::new();

    // TODO add support for the old format file version < 1031
    // https://github.com/freezy/VisualPinball.Engine/blob/ec1e9765cd4832c134e889d6e6d03320bc404bd5/VisualPinball.Engine/VPT/Sound/SoundData.cs#L98

    let num_values = if file_version.u32() < NEW_SOUND_FORMAT_VERSION {
        6
    } else {
        10
    };

    for i in 0..num_values {
        match i {
            0 => {
                name = reader.get_string_no_remaining_update();
            }
            1 => {
                path = reader.get_string_no_remaining_update();
            }
            2 => {
                internal_name = reader.get_string_no_remaining_update();
            }
            3 => {
                if is_wav(&path.to_owned()) {
                    wave_form = read_wave_form(reader);
                } else {
                    // should we be doing something here?
                }
            }
            4 => {
                data = reader.get_data_no_remaining_update();
            }
            5 => {
                output_target = reader.get_u8_no_remaining_update();
            }
            6 => {
                volume = reader.get_u32_no_remaining_update();
            }
            7 => {
                balance = reader.get_u32_no_remaining_update();
            }
            8 => {
                fade = reader.get_u32_no_remaining_update();
            }
            9 => {
                // TODO why do we have the volume twice?
                volume = reader.get_u32_no_remaining_update();
            }
            unexpected => {
                panic!("unexpected value {}", unexpected);
            }
        }
    }

    SoundData {
        name,
        path,
        data: data.to_vec(),
        wave_form,
        internal_name,
        fade,
        volume,
        balance,
        output_target,
    }
}

fn is_wav(path: &str) -> bool {
    path.to_lowercase().ends_with(".wav")
}

pub(crate) fn write(file_version: &Version, sound: &SoundData, writer: &mut BiffWriter) {
    writer.write_string(&sound.name);
    writer.write_string(&sound.path);
    writer.write_string_empty_zero(&sound.internal_name);

    if is_wav(&sound.path.to_owned()) {
        write_wave_form(writer, &sound.wave_form);
    } else {
        // should we be doing something here?
    }

    writer.write_length_prefixed_data(&sound.data);
    writer.write_u8(sound.output_target);
    if file_version.u32() >= NEW_SOUND_FORMAT_VERSION {
        writer.write_u32(sound.volume);
        writer.write_u32(sound.balance);
        writer.write_u32(sound.fade);
        writer.write_u32(sound.volume);
    }
}

fn read_wave_form(reader: &mut BiffReader<'_>) -> WaveForm {
    let format_tag = reader.get_u16_no_remaining_update();
    let channels = reader.get_u16_no_remaining_update();
    let samples_per_sec = reader.get_u32_no_remaining_update();
    let avg_bytes_per_sec = reader.get_u32_no_remaining_update();
    let block_align = reader.get_u16_no_remaining_update();
    let bits_per_sample = reader.get_u16_no_remaining_update();
    let cb_size = reader.get_u16_no_remaining_update();
    WaveForm {
        format_tag,
        channels,
        samples_per_sec,
        avg_bytes_per_sec,
        block_align,
        bits_per_sample,
        cb_size,
    }
}

fn write_wave_form(writer: &mut BiffWriter, wave_form: &WaveForm) {
    writer.write_u16(wave_form.format_tag);
    writer.write_u16(wave_form.channels);
    writer.write_u32(wave_form.samples_per_sec);
    writer.write_u32(wave_form.avg_bytes_per_sec);
    writer.write_u16(wave_form.block_align);
    writer.write_u16(wave_form.bits_per_sample);
    writer.write_u16(wave_form.cb_size);
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    // TODO add test for non-wav sound

    #[test]
    fn test_write_read_wav() {
        let sound: SoundData = SoundData {
            name: "test name".to_string(),
            path: "test path.wav".to_string(),
            data: vec![1, 2, 3, 4],
            wave_form: WaveForm {
                format_tag: 1,
                channels: 2,
                samples_per_sec: 3,
                avg_bytes_per_sec: 4,
                block_align: 5,
                bits_per_sample: 6,
                cb_size: 7,
            },
            internal_name: "test internalname".to_string(),
            fade: 1,
            volume: 2,
            balance: 3,
            output_target: 4,
        };
        let mut writer = BiffWriter::new();
        write(&Version::new(1074), &sound, &mut writer);
        let sound_read = read(&Version::new(1074), &mut BiffReader::new(writer.get_data()));
        assert_eq!(sound, sound_read);
    }

    #[test]
    fn test_write_read_other() {
        let sound: SoundData = SoundData {
            name: "test name".to_string(),
            path: "test path.mp3".to_string(),
            data: vec![1, 2, 3, 4],
            wave_form: WaveForm::default(),
            internal_name: "test internalname".to_string(),
            fade: 1,
            volume: 2,
            balance: 3,
            output_target: 4,
        };
        let mut writer = BiffWriter::new();
        write(&Version::new(1083), &sound, &mut writer);
        let sound_read = read(&Version::new(1083), &mut BiffReader::new(writer.get_data()));
        assert_eq!(sound, sound_read);
    }
}
