use std::fmt::Debug;

use quick_xml::de::from_str;
use quick_xml::de::*;
use serde::Deserialize;

// The xml model is based on this
// https://github.com/vpinball/b2s-backglass/blob/f43ae8aacbb79d3413531991e4c0156264442c39/b2sbackglassdesigner/b2sbackglassdesigner/classes/CreateCode/Coding.vb#L30

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
    pub file_name: String,
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

#[derive(Deserialize)]
pub struct OnImageTag {
    #[serde(rename = "@Value")]
    pub value: String,
    #[serde(rename = "@FileName")]
    pub file_name: String,
    #[serde(rename = "@RomID")]
    pub rom_id: Option<String>,
    #[serde(rename = "@RomIDType")]
    pub rom_id_type: Option<String>,
}

// debug for ImageTag not showing length of value
impl Debug for OnImageTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnImageTag")
            .field("value", &format!("base64 {:?} bytes", self.value.len()))
            .field("file_name", &self.file_name)
            .finish()
    }
}

#[derive(Debug, Deserialize)]
pub struct Images {
    #[serde(rename = "BackglassOffImage")]
    pub backglass_off_image: Option<ValueTag>,
    #[serde(rename = "BackglassOnImage")]
    pub backglass_on_image: Option<OnImageTag>,
    #[serde(rename = "BackglassImage")]
    pub backglass_image: Option<ImageTag>,
    #[serde(rename = "DMDImage")]
    pub dmd_image: Option<ImageTag>,
    #[serde(rename = "IlluminationImage")]
    pub illumination_image: Option<ValueTag>,
    #[serde(rename = "ThumbnailImage")]
    pub thumbnail_image: ValueTag,
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
    pub name: String,
    #[serde(rename = "@Parent")]
    pub parent: String,
    #[serde(rename = "@DualMode")]
    pub dual_mode: String,
    #[serde(rename = "@Interval")]
    pub interval: String,
    #[serde(rename = "@Loops")]
    pub loops: String,
    #[serde(rename = "@IDJoin")]
    pub id_join: String,
    #[serde(rename = "@StartAnimationAtBackglassStartup")]
    pub start_animation_at_backglass_startup: String,
    #[serde(rename = "@LightsStateAtAnimationStart")]
    pub lights_state_at_animation_start: String,
    #[serde(rename = "@LightsStateAtAnimationEnd")]
    pub lights_state_at_animation_end: String,
    #[serde(rename = "@AnimationStopBehaviour")]
    pub animation_stop_behaviour: String,
    #[serde(rename = "@LockInvolvedLamps")]
    pub lock_involved_lamps: String,
    #[serde(rename = "@HideScoreDisplays")]
    pub hide_score_displays: String,
    #[serde(rename = "@BringToFront")]
    pub bring_to_front: String,
    #[serde(rename = "AnimationStep")]
    pub animation_step: Vec<AnimationStep>,
}

#[derive(Debug, Deserialize)]
pub struct Animations {
    #[serde(rename = "Animation")]
    pub animation: Option<Vec<Animation>>,
}

#[derive(Deserialize)]
pub struct Bulb {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "@Parent")]
    pub parent: Option<String>,
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@B2SID")]
    pub b2s_id: Option<String>,
    #[serde(rename = "@B2SIDType")]
    pub b2s_id_type: Option<String>,
    #[serde(rename = "@B2SValue")]
    pub b2s_value: Option<String>,
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
    #[serde(rename = "@ZOrder")]
    pub z_order: Option<String>,
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
    // SnippitMechID
    // SnippitRotatingSteps
    // SnippitRotatingDirection
    // SnippitRotatingStopBehaviour
    // SnippitRotatingInterval
    pub snippit_type: Option<String>,
    #[serde(rename = "@Image")]
    pub image: String,
    #[serde(rename = "@OffImage")]
    pub off_image: Option<String>,
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
    pub bulb: Option<Vec<Bulb>>,
}

#[derive(Debug, Deserialize)]
pub struct Score {
    #[serde(rename = "@Parent")]
    pub parent: String,
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "@B2SStartDigit")]
    pub b2s_start_digit: String,
    #[serde(rename = "@B2SScoreType")]
    pub b2s_score_type: String,
    #[serde(rename = "@B2SPlayerNo")]
    pub b2s_player_no: String,
    #[serde(rename = "@ReelType")]
    pub reel_type: String,
    #[serde(rename = "@ReelIlluImageSet")]
    pub reel_illu_image_set: Option<String>,
    #[serde(rename = "@ReelIlluLocation")]
    pub reel_illu_location: Option<String>,
    #[serde(rename = "@ReelIlluIntensity")]
    pub reel_illu_intensity: Option<String>,
    #[serde(rename = "@ReelIlluB2SID")]
    pub reel_illu_b2s_id: Option<String>,
    #[serde(rename = "@ReelIlluB2SIDType")]
    pub reel_illu_b2s_id_type: Option<String>,
    #[serde(rename = "@ReelIlluB2SValue")]
    pub reel_illu_b2s_value: Option<String>,
    #[serde(rename = "@ReelLitColor")]
    pub reel_lit_color: String,
    #[serde(rename = "@ReelDarkColor")]
    pub reel_dark_color: String,
    #[serde(rename = "@Glow")]
    pub glow: String,
    #[serde(rename = "@Thickness")]
    pub thickness: String,
    #[serde(rename = "@Shear")]
    pub shear: String,
    #[serde(rename = "@Digits")]
    pub digits: String,
    #[serde(rename = "@Spacing")]
    pub spacing: String,
    #[serde(rename = "@DisplayState")]
    pub display_state: String,
    #[serde(rename = "@LocX")]
    pub loc_x: String,
    #[serde(rename = "@LocY")]
    pub loc_y: String,
    #[serde(rename = "@Width")]
    pub width: String,
    #[serde(rename = "@Height")]
    pub height: String,
    // following fields are not really in use as far as I know
    #[serde(rename = "@Sound3")]
    pub sound3: Option<String>,
    #[serde(rename = "@Sound4")]
    pub sound4: Option<String>,
    #[serde(rename = "@Sound5")]
    pub sound5: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Scores {
    #[serde(rename = "@ReelCountOfIntermediates")]
    pub reel_count_of_intermediates: String,
    #[serde(rename = "@ReelRollingDirection")]
    pub reel_rolling_direction: String,
    #[serde(rename = "@ReelRollingInterval")]
    pub reel_rolling_interval: String,

    #[serde(rename = "Score")]
    pub score: Option<Vec<Score>>,
}

#[derive(Debug, Deserialize)]
pub struct ReelsImage {
    // TODO there might be dynamic fields here for IntermediateImage0, IntermediateImage1, etc.
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@CountOfIntermediates")]
    pub count_of_intermediates: String,
    #[serde(rename = "@Image")]
    pub image: String, // base64 encoded image
}

#[derive(Debug, Deserialize)]
pub struct ReelsImages {
    #[serde(rename = "Image")]
    pub image: Option<Vec<ReelsImage>>,
}

#[derive(Debug, Deserialize)]
pub struct ReelsIlluminatedImage {
    // TODO there might be dynamic fields here for IntermediateImage0, IntermediateImage1, etc.
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@CountOfIntermediates")]
    pub count_of_intermediates: String,
    #[serde(rename = "@Image")]
    pub image: String, // base64 encoded image
}

#[derive(Debug, Deserialize)]
pub struct ReelsIlluminatedImagesSet {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "IlluminatedImage")]
    pub illuminated_image: Vec<ReelsIlluminatedImage>,
}

#[derive(Debug, Deserialize)]
pub struct ReelsIlluminatedImages {
    #[serde(rename = "Set")]
    pub set: Option<Vec<ReelsIlluminatedImagesSet>>,
}

#[derive(Debug, Deserialize)]
pub struct Reels {
    #[serde(rename = "Images")]
    pub images: ReelsImages,
    #[serde(rename = "IlluminatedImages")]
    pub illuminated_images: ReelsIlluminatedImages,
}

#[derive(Debug, Deserialize)]
pub struct Sounds {
    // as far as I can see this is not in use
}

#[derive(Debug, Deserialize)]
pub struct DMDDefaultLocation {
    #[serde(rename = "@LocX")]
    pub loc_x: String,
    #[serde(rename = "@LocY")]
    pub loc_y: String,
}

#[derive(Debug, Deserialize)]
pub struct GrillHeight {
    #[serde(rename = "@Value")]
    pub value: String,
    #[serde(rename = "@Small")]
    pub small: Option<String>,
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
    #[serde(rename = "DMDDefaultLocation")]
    pub dmd_default_location: DMDDefaultLocation,
    #[serde(rename = "GrillHeight")]
    pub grill_height: GrillHeight,
    #[serde(rename = "ProjectGUID")]
    pub project_guid: ValueTag,
    #[serde(rename = "ProjectGUID2")]
    pub project_guid2: ValueTag,
    #[serde(rename = "AssemblyGUID")]
    pub assembly_guid: ValueTag,
    #[serde(rename = "VSName")]
    pub vsname: ValueTag,
    #[serde(rename = "DualBackglass")]
    pub dual_backglass: ValueTag,
    #[serde(rename = "Author")]
    pub author: ValueTag,
    #[serde(rename = "Artwork")]
    pub artwork: ValueTag,
    #[serde(rename = "GameName")]
    pub game_name: ValueTag,
    #[serde(rename = "AddEMDefaults")]
    pub add_em_defaults: ValueTag,
    #[serde(rename = "CommType")]
    pub comm_type: ValueTag,
    #[serde(rename = "DestType")]
    pub dest_type: ValueTag,
    #[serde(rename = "NumberOfPlayers")]
    pub number_of_players: ValueTag,
    #[serde(rename = "B2SDataCount")]
    pub b2s_data_count: ValueTag,
    #[serde(rename = "ReelType")]
    pub reel_type: ValueTag,
    #[serde(rename = "UseDream7LEDs")]
    pub use_dream7_leds: ValueTag,
    #[serde(rename = "D7Glow")]
    pub d7_glow: ValueTag,
    #[serde(rename = "D7Thickness")]
    pub d7_thickness: ValueTag,
    #[serde(rename = "D7Shear")]
    pub d7_shear: ValueTag,
    #[serde(rename = "ReelColor")]
    pub reel_color: Option<ValueTag>,
    #[serde(rename = "ReelRollingDirection")]
    pub reel_rolling_direction: ValueTag,
    #[serde(rename = "ReelRollingInterval")]
    pub reel_rolling_interval: ValueTag,
    #[serde(rename = "ReelIntermediateImageCount")]
    pub reel_intermediate_image_count: ValueTag,
    #[serde(rename = "Animations")]
    pub animations: Animations,
    #[serde(rename = "Scores")]
    pub scores: Option<Scores>,
    #[serde(rename = "Reels")]
    pub reels: Option<Reels>,
    #[serde(rename = "Illumination")]
    pub illumination: Illumination,
    #[serde(rename = "Sounds")]
    pub sounds: Option<Sounds>,
    #[serde(rename = "Images")]
    pub images: Images,
}

pub fn load(text: &str) -> Result<DirectB2SData, DeError> {
    // this will probably use up a lot of memory
    from_str::<DirectB2SData>(text)
}
