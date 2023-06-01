use std::fmt::Debug;

use quick_xml::de::from_str;
use quick_xml::de::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ValueTag {
    #[serde(rename = "@Value")]
    pub value: String,
}

#[derive(Deserialize)]
pub struct ImageTag {
    #[serde(rename = "@Value")]
    pub value: String,
    #[serde(rename = "@FileName")]
    pub file_name: Option<String>,
}

// debug for ImageTag not showing length of value
impl Debug for ImageTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageTag")
            .field("value", &format!("base64 {:?} bytes", self.value.len()))
            .field("file_name", &self.file_name)
            .finish()
    }
}

#[derive(Debug, Deserialize)]
pub struct Images {
    #[serde(rename = "BackglassImage")]
    pub backglass_image: ImageTag,
    #[serde(rename = "DMDImage")]
    pub dmd_image: Option<ImageTag>,
    #[serde(rename = "ThumbnailImage")]
    pub thumbnail_image: ImageTag,
}

#[derive(Debug, Deserialize)]
pub struct AnimationStep {
    #[serde(rename = "@Step")]
    pub step: String,
    #[serde(rename = "@On")]
    pub on: String,
    #[serde(rename = "@WaitLoopsAfterOn")]
    pub wait_loops_after_on: String,
    #[serde(rename = "@Off")]
    pub off: String,
    #[serde(rename = "@WaitLoopsAfterOff")]
    pub wait_loops_after_off: String,
}

#[derive(Debug, Deserialize)]
pub struct Animation {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "@Parent")]
    parent: String,
    #[serde(rename = "@DualMode")]
    dual_mode: String,
    #[serde(rename = "@Interval")]
    interval: String,
    #[serde(rename = "@Loops")]
    loops: String,
    #[serde(rename = "@IDJoin")]
    id_join: String,
    #[serde(rename = "@StartAnimationAtBackglassStartup")]
    start_animation_at_backglass_startup: String,
    #[serde(rename = "@LightsStateAtAnimationStart")]
    lights_state_at_animation_start: String,
    #[serde(rename = "@LightsStateAtAnimationEnd")]
    lights_state_at_animation_end: String,
    #[serde(rename = "@AnimationStopBehaviour")]
    animation_stop_behaviour: String,
    #[serde(rename = "@LockInvolvedLamps")]
    lock_involved_lamps: String,
    #[serde(rename = "@HideScoreDisplays")]
    hide_score_displays: String,
    #[serde(rename = "@BringToFront")]
    bring_to_front: String,
    #[serde(rename = "AnimationStep")]
    animation_step: Vec<AnimationStep>,
}

#[derive(Debug, Deserialize)]
pub struct Animations {
    #[serde(rename = "Animation")]
    animation: Vec<Animation>,
}

#[derive(Deserialize)]
pub struct Bulb {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@RomID")]
    pub rom_id: Option<String>,
    #[serde(rename = "@RomIDType")]
    pub rom_id_type: Option<String>,
    #[serde(rename = "@RomInverted")]
    pub rom_inverted: Option<String>,
    #[serde(rename = "@InitialState")]
    pub initial_state: String,
    #[serde(rename = "@DualMode")]
    pub dual_mode: String,
    #[serde(rename = "@Intensity")]
    pub intensity: String,
    #[serde(rename = "@LightColor")]
    pub light_color: String,
    #[serde(rename = "@DodgeColor")]
    pub dodge_color: String,
    #[serde(rename = "@IlluMode")]
    pub illu_mode: Option<String>,
    #[serde(rename = "@Visible")]
    pub visible: String,
    #[serde(rename = "@LocX")]
    pub loc_x: String,
    #[serde(rename = "@LocY")]
    pub loc_y: String,
    #[serde(rename = "@Width")]
    pub width: String,
    #[serde(rename = "@Height")]
    pub height: String,
    #[serde(rename = "@IsImageSnippit")]
    pub is_image_snippit: String,
    #[serde(rename = "@SnippitType")]
    // TODO add more from https://github.com/vpinball/b2s-backglass/blob/f43ae8aacbb79d3413531991e4c0156264442c39/b2sbackglassdesigner/b2sbackglassdesigner/classes/Save.vb#LL309C50-L309C61
    pub snippit_type: Option<String>,
    #[serde(rename = "@Image")]
    pub image: String,
    #[serde(rename = "@Text")]
    pub text: String,
    #[serde(rename = "@TextAlignment")]
    pub text_alignment: String,
    #[serde(rename = "@FontName")]
    pub font_name: String,
    #[serde(rename = "@FontSize")]
    pub font_size: String,
    #[serde(rename = "@FontStyle")]
    pub font_style: String,
}

// debug for Bulb not showing length of image
impl Debug for Bulb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bulb")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("rom_id", &self.rom_id)
            .field("rom_id_type", &self.rom_id_type)
            .field("rom_inverted", &self.rom_inverted)
            .field("initial_state", &self.initial_state)
            .field("dual_mode", &self.dual_mode)
            .field("intensity", &self.intensity)
            .field("light_color", &self.light_color)
            .field("dodge_color", &self.dodge_color)
            .field("illu_mode", &self.illu_mode)
            .field("visible", &self.visible)
            .field("loc_x", &self.loc_x)
            .field("loc_y", &self.loc_y)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("is_image_snippit", &self.is_image_snippit)
            .field("image", &format!("base64 {:?} bytes", self.image.len()))
            .field("text", &self.text)
            .field("text_alignment", &self.text_alignment)
            .field("font_name", &self.font_name)
            .field("font_size", &self.font_size)
            .field("font_style", &self.font_style)
            .finish()
    }
}

#[derive(Debug, Deserialize)]
pub struct Illumination {
    #[serde(rename = "Bulb")]
    pub bulb: Vec<Bulb>,
}

#[derive(Debug, Deserialize)]
pub struct Scores {
    #[serde(rename = "@ReelCountOfIntermediates")]
    reel_count_of_intermediates: String,
    #[serde(rename = "@ReelRollingDirection")]
    reel_rolling_direction: String,
    #[serde(rename = "@ReelRollingInterval")]
    reel_rolling_interval: String,
}

#[derive(Debug, Deserialize)]
pub struct DirectB2SData {
    #[serde(rename = "@Version")]
    pub version: String,
    #[serde(rename = "Name")]
    pub name: ValueTag,
    #[serde(rename = "TableType")]
    pub table_type: ValueTag,
    #[serde(rename = "DMDType")]
    pub dmd_type: ValueTag,
    #[serde(rename = "VSName")]
    pub vsname: ValueTag,
    #[serde(rename = "NumberOfPlayers")]
    pub number_of_players: ValueTag,
    pub animations: Option<Animations>,
    pub scores: Option<Scores>,
    #[serde(rename = "Illumination")]
    pub illumination: Illumination,
    #[serde(rename = "Images")]
    pub images: Images,
}

pub fn load(text: &str) -> Result<DirectB2SData, DeError> {
    // this will probably use up a lot of memory
    from_str::<DirectB2SData>(text)
}
