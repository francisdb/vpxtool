#![allow(dead_code)]

use super::{
    biff::{self, BiffReader, BiffWriter},
    math::{dequantize_unsigned, quantize_unsigned},
    model::StringWithEncoding,
    version::Version,
};

pub const VIEW_LAYOUT_MODE_LEGACY: u32 = 0; // All tables before 10.8 used a viewer position relative to a fitting of a set of bounding vertices (not all parts) with a standard perspective projection skewed by a layback angle
pub const VIEW_LAYOUT_MODE_CAMERA: u32 = 1; // Position viewer relative to the bottom center of the table, use a standard camera perspective projection, replace layback by a frustrum offset
pub const VIEW_LAYOUT_MODE_WINDOW: u32 = 2; // Position viewer relative to the bottom center of the table, use an oblique surface (re)projection (needs some postprocess to avoid distortion)

// TODO switch to a array of 3 view modes like in the original code
#[derive(Debug, PartialEq)]
pub struct ViewSetup {
    // ViewLayoutMode mMode = VLM_LEGACY;

    // // Overall scene scale
    // float mSceneScaleZ = 1.0f;

    // // View position (relative to table bounds for legacy mode, relative to the bottom center of the table for others)
    // float mViewX = 0.f;
    // float mViewY = CMTOVPU(20.f);
    // float mViewZ = CMTOVPU(70.f);
    // float mLookAt = 0.25f; // Look at expressed as a camera inclination for legacy, or a percent of the table height, starting from bottom (0.25 is around top of slingshots)

    // // Viewport adjustments
    // float mViewportRotation = 0.f;
    // float mSceneScaleX = 1.0f;
    // float mSceneScaleY = 1.0f;

    // // View properties
    // float mFOV = 45.0f; // Camera & Legacy: Field of view, in degrees
    // float mLayback = 0.0f; // Legacy: A skewing angle that deform the table to make it look 'good'
    // float mViewHOfs = 0.0f; // Camera & Window: horizontal frustrum offset
    // float mViewVOfs = 0.0f; // Camera & Window: vertical frustrum offset

    // // Magic Window mode properties
    // float mWindowTopXOfs = 0.0f; // Upper window border offset from left and right table bounds
    // float mWindowTopYOfs = 0.0f; // Upper window border Y coordinate, relative to table top
    // float mWindowTopZOfs = CMTOVPU(20.0f); // Upper window border Z coordinate, relative to table playfield Z
    // float mWindowBottomXOfs = 0.0f; // Lower window border offset from left and right table bounds
    // float mWindowBottomYOfs = 0.0f; // Lower window border Y coordinate, relative to table bottom
    // float mWindowBottomZOfs = CMTOVPU(7.5f); // Lower window border Z coordinate, relative to table playfield Z
    pub mode: u32,
}

impl ViewSetup {
    pub fn new() -> Self {
        ViewSetup {
            mode: VIEW_LAYOUT_MODE_LEGACY,
        }
    }

    fn default() -> Self {
        ViewSetup::new()
    }
}

#[derive(Debug, PartialEq)]
pub struct GameData {
    pub left: f32,                                                 // LEFT 1
    pub top: f32,                                                  // TOPX 2
    pub right: f32,                                                // RGHT 3
    pub bottom: f32,                                               // BOTM 4
    pub clmo: Option<u32>,                                         // CLMO added in 10.8.0?
    pub bg_view_mode_desktop: Option<u32>,                         // VSM0 added in 10.8.x
    pub bg_rotation_desktop: f32,                                  // ROTA 5
    pub bg_inclination_desktop: f32,                               // INCL 6
    pub bg_layback_desktop: f32,                                   // LAYB 7
    pub bg_fov_desktop: f32,                                       // FOVX 8
    pub bg_offset_x_desktop: f32,                                  // XLTX 9
    pub bg_offset_y_desktop: f32,                                  // XLTY 10
    pub bg_offset_z_desktop: f32,                                  // XLTZ 11
    pub bg_scale_x_desktop: f32,                                   // SCLX 12
    pub bg_scale_y_desktop: f32,                                   // SCLY 13
    pub bg_scale_z_desktop: f32,                                   // SCLZ 14
    pub bg_enable_fss: Option<bool>,                               // EFSS 15 (added in 10.?)
    pub bg_view_horizontal_offset_desktop: Option<f32>,            // HOF0 added in 10.8.x
    pub bg_view_vertical_offset_desktop: Option<f32>,              // VOF0 added in 10.8.x
    pub bg_window_top_x_offset_desktop: Option<f32>,               // WTX0 added in 10.8.x
    pub bg_window_top_y_offset_desktop: Option<f32>,               // WTY0 added in 10.8.x
    pub bg_window_top_z_offset_desktop: Option<f32>,               // WTZ0 added in 10.8.x
    pub bg_window_bottom_x_offset_desktop: Option<f32>,            // WBX0 added in 10.8.x
    pub bg_window_bottom_y_offset_desktop: Option<f32>,            // WBY0 added in 10.8.x
    pub bg_window_bottom_z_offset_desktop: Option<f32>,            // WBZ0 added in 10.8.x
    pub bg_view_mode_fullscreen: Option<u32>,                      // VSM1 added in 10.8.x
    pub bg_rotation_fullscreen: f32,                               // ROTF 16
    pub bg_inclination_fullscreen: f32,                            // INCF 17
    pub bg_layback_fullscreen: f32,                                // LAYF 18
    pub bg_fov_fullscreen: f32,                                    // FOVF 19
    pub bg_offset_x_fullscreen: f32,                               // XLFX 20
    pub bg_offset_y_fullscreen: f32,                               // XLFY 21
    pub bg_offset_z_fullscreen: f32,                               // XLFZ 22
    pub bg_scale_x_fullscreen: f32,                                // SCFX 23
    pub bg_scale_y_fullscreen: f32,                                // SCFY 24
    pub bg_scale_z_fullscreen: f32,                                // SCFZ 25
    pub bg_view_horizontal_offset_fullscreen: Option<f32>,         // HOF1 added in 10.8.x
    pub bg_view_vertical_offset_fullscreen: Option<f32>,           // VOF1 added in 10.8.x
    pub bg_window_top_x_offset_fullscreen: Option<f32>,            // WTX1 added in 10.8.x
    pub bg_window_top_y_offset_fullscreen: Option<f32>,            // WTY1 added in 10.8.x
    pub bg_window_top_z_offset_fullscreen: Option<f32>,            // WTZ1 added in 10.8.x
    pub bg_window_bottom_x_offset_fullscreen: Option<f32>,         // WBX1 added in 10.8.x
    pub bg_window_bottom_y_offset_fullscreen: Option<f32>,         // WBY1 added in 10.8.x
    pub bg_window_bottom_z_offset_fullscreen: Option<f32>,         // WBZ1 added in 10.8.x
    pub bg_view_mode_full_single_screen: Option<u32>,              // VSM2 added in 10.8.x
    pub bg_rotation_full_single_screen: Option<f32>,               // ROFS 26 (added in 10.?)
    pub bg_inclination_full_single_screen: Option<f32>,            // INFS 27 (added in 10.?)
    pub bg_layback_full_single_screen: Option<f32>,                // LAFS 28 (added in 10.?)
    pub bg_fov_full_single_screen: Option<f32>,                    // FOFS 29 (added in 10.?)
    pub bg_offset_x_full_single_screen: Option<f32>,               // XLXS 30 (added in 10.?)
    pub bg_offset_y_full_single_screen: Option<f32>,               // XLYS 31 (added in 10.?)
    pub bg_offset_z_full_single_screen: Option<f32>,               // XLZS 32 (added in 10.?)
    pub bg_scale_x_full_single_screen: Option<f32>,                // SCXS 33 (added in 10.?)
    pub bg_scale_y_full_single_screen: Option<f32>,                // SCYS 34 (added in 10.?)
    pub bg_scale_z_full_single_screen: Option<f32>,                // SCZS 35 (added in 10.?)
    pub bg_view_horizontal_offset_full_single_screen: Option<f32>, // HOF2 added in 10.8.x
    pub bg_view_vertical_offset_full_single_screen: Option<f32>,   // VOF2 added in 10.8.x
    pub bg_window_top_x_offset_full_single_screen: Option<f32>,    // WTX2 added in 10.8.x
    pub bg_window_top_y_offset_full_single_screen: Option<f32>,    // WTY2 added in 10.8.x
    pub bg_window_top_z_offset_full_single_screen: Option<f32>,    // WTZ2 added in 10.8.x
    pub bg_window_bottom_x_offset_full_single_screen: Option<f32>, // WBX2 added in 10.8.x
    pub bg_window_bottom_y_offset_full_single_screen: Option<f32>, // WBY2 added in 10.8.x
    pub bg_window_bottom_z_offset_full_single_screen: Option<f32>, // WBZ2 added in 10.8.x
    pub override_physics: u32,                                     // ORRP 36
    pub override_physics_flipper: Option<bool>,                    // ORPF 37 added in ?
    pub gravity: f32,                                              // GAVT 38
    pub friction: f32,                                             // FRCT 39
    pub elasticity: f32,                                           // ELAS 40
    pub elastic_falloff: f32,                                      // ELFA 41
    pub scatter: f32,                                              // PFSC 42
    pub default_scatter: f32,                                      // SCAT 43
    pub nudge_time: f32,                                           // NDGT 44
    pub plunger_normalize: u32,                                    // MPGC 45
    pub plunger_filter: bool,                                      // MPDF 46
    pub physics_max_loops: u32,                                    // PHML 47
    pub render_em_reels: bool,                                     // REEL 48
    pub render_decals: bool,                                       // DECL 49
    pub offset_x: f32,                                             // OFFX 50
    pub offset_y: f32,                                             // OFFY 51
    pub zoom: f32,                                                 // ZOOM 52
    pub angle_tilt_max: f32,                                       // SLPX 53
    pub angle_tilt_min: f32,                                       // SLOP 54
    pub stereo_max_separation: f32,                                // MAXS 55
    pub stereo_zero_parallax_displacement: f32,                    // ZPD 56
    pub stereo_offset: f32,                                        // STO 57
    pub overwrite_global_stereo3d: bool,                           // OGST 58
    pub image: String,                                             // IMAG 59
    pub backglass_image_full_desktop: String,                      // BIMG 60
    pub backglass_image_full_fullscreen: String,                   // BIMF 61
    pub backglass_image_full_single_screen: Option<String>,        // BIMS 62 (added in 10.?)
    pub image_backdrop_night_day: bool,                            // BIMN 63
    pub image_color_grade: String,                                 // IMCG 64
    pub ball_image: String,                                        // BLIM 65
    pub ball_spherical_mapping: Option<bool>,                      // BLSM (added in 10.8)
    pub ball_image_front: String,                                  // BLIF 66
    pub env_image: String,                                         // EIMG 67
    pub notes: Option<String>,                                     // NOTX 67.5 (added in 10.7)
    pub screen_shot: String,                                       // SSHT 68
    pub display_backdrop: bool,                                    // FBCK 69
    pub glass_height: f32,                                         // GLAS 70
    pub table_height: f32,                                         // TBLH 71
    pub playfield_material: String,                                // PLMA 72
    pub backdrop_color: u32,                                       // BCLR 73 (color bgr)
    pub global_difficulty: f32,                                    // TDFT 74
    pub light_ambient: u32,                                        // LZAM 75 (color)
    pub light0_emission: u32,                                      // LZDI 76 (color)
    pub light_height: f32,                                         // LZHI 77
    pub light_range: f32,                                          // LZRA 78
    pub light_emission_scale: f32,                                 // LIES 79
    pub env_emission_scale: f32,                                   // ENES 80
    pub global_emission_scale: f32,                                // GLES 81
    pub ao_scale: f32,                                             // AOSC 82
    pub ssr_scale: Option<f32>,                                    // SSSC 83 (added in 10.?)
    pub table_sound_volume: f32,                                   // SVOL 84
    pub table_music_volume: f32,                                   // MVOL 85
    pub table_adaptive_vsync: i32,                                 // AVSY 86
    pub use_reflection_for_balls: i32,                             // BREF 87
    pub playfield_reflection_strength: f32,                        // PLST 88
    pub use_trail_for_balls: i32,                                  // BTRA 89
    pub ball_decal_mode: bool,                                     // BDMO 90
    pub ball_playfield_reflection_strength: f32,                   // BPRS 91
    pub default_bulb_intensity_scale_on_ball: Option<f32>,         // DBIS 92 (added in 10.?)
    /// this has a special quantization,
    /// See [`Self::get_ball_trail_strength`] and [`Self::set_ball_trail_strength`]
    pub ball_trail_strength: u32, // BTST 93
    pub user_detail_level: u32,                                    // ARAC 94
    pub overwrite_global_detail_level: bool,                       // OVDL 95
    pub overwrite_global_day_night: bool,                          // OVDN 96
    pub show_grid: bool,                                           // GDAC 97
    pub reflect_elements_on_playfield: bool,                       // REOP 98
    pub use_aal: i32,                                              // UAAL 99
    pub use_fxaa: i32,                                             // UFXA 100
    pub use_ao: i32,                                               // UAOC 101
    pub use_ssr: Option<i32>,                                      // USSR 102 (added in 10.?)
    pub bloom_strength: f32,                                       // BLST 103
    pub materials_size: u32,                                       // MASI 104
    pub materials: Vec<u8>,                                        // MATE 105
    pub materials_physics: Option<Vec<u8>>,                        // PHMA 106 (added in 10.?)
    pub materials_new: Option<Vec<Vec<u8>>>,                       // MATR (added in 10.8)
    pub render_probes: Option<Vec<Vec<u8>>>,                       // RPRB (added in 10.8)
    pub gameitems_size: u32,                                       // SEDT 107
    pub sounds_size: u32,                                          // SSND 108
    pub images_size: u32,                                          // SIMG 109
    pub fonts_size: u32,                                           // SFNT 110
    pub collections_size: u32,                                     // SCOL 111
    pub name: String,                                              // NAME 112
    pub custom_colors: Vec<u8>,                                    //[Color; 16], // CCUS 113
    pub protection_data: Option<Vec<u8>>,                          // SECB (removed in ?)
    pub code: StringWithEncoding,                                  // CODE 114
    // This is a bit of a hack because we want reproducible builds.
    // 10.8.0 beta 1-4 had EFSS at the old location, but it was moved to the new location in beta 5
    // Some tables were released with these old betas, so we need to support both locations to be 100% reproducing the orignal table
    // and it's MAC hash.
    pub is_10_8_0_beta1_to_beta4: bool,
}

impl GameData {
    pub fn set_code(&mut self, script: String) {
        self.code = StringWithEncoding::new(script);
    }

    pub fn get_ball_trail_strength(&self) -> f32 {
        dequantize_unsigned(8, self.ball_trail_strength)
    }

    pub fn set_ball_trail_strength(&mut self, value: f32) {
        self.ball_trail_strength = quantize_unsigned(8, value);
    }
}

impl Default for GameData {
    fn default() -> Self {
        GameData {
            left: 0.0,
            top: 0.0,
            right: 952.0,
            bottom: 2162.0,
            clmo: None,
            bg_view_mode_desktop: None,
            bg_rotation_desktop: 0.0,
            bg_inclination_desktop: 0.0,
            bg_layback_desktop: 0.0,
            bg_fov_desktop: 45.0,
            bg_offset_x_desktop: 0.0,
            bg_offset_y_desktop: 30.0,
            bg_offset_z_desktop: -200.0,
            bg_scale_x_desktop: 1.0,
            bg_scale_y_desktop: 1.0,
            bg_scale_z_desktop: 1.0,
            bg_enable_fss: None, //false,
            is_10_8_0_beta1_to_beta4: false,
            bg_rotation_fullscreen: 0.0,
            bg_inclination_fullscreen: 0.0,
            bg_layback_fullscreen: 0.0,
            bg_fov_fullscreen: 45.0,
            bg_offset_x_fullscreen: 110.0,
            bg_offset_y_fullscreen: -86.0,
            bg_offset_z_fullscreen: 400.0,
            bg_scale_x_fullscreen: 1.3,
            bg_scale_y_fullscreen: 1.41,
            bg_scale_z_fullscreen: 1.0,
            bg_rotation_full_single_screen: None,    //0.0,
            bg_inclination_full_single_screen: None, //52.0,
            bg_layback_full_single_screen: None,     //0.0,
            bg_fov_full_single_screen: None,         //45.0,
            bg_offset_x_full_single_screen: None,    //0.0,
            bg_offset_y_full_single_screen: None,    //30.0,
            bg_offset_z_full_single_screen: None,    //-50.0,
            bg_scale_x_full_single_screen: None,     //1.2,
            bg_scale_y_full_single_screen: None,     //1.1,
            bg_scale_z_full_single_screen: None,     //1.0,
            override_physics: 0,
            override_physics_flipper: None, //false,
            gravity: 1.762985,
            friction: 0.075,
            elasticity: 0.25,
            elastic_falloff: 0.0,
            scatter: 0.0,
            default_scatter: 0.0,
            nudge_time: 5.0,
            plunger_normalize: 100,
            plunger_filter: false,
            physics_max_loops: 0,
            render_em_reels: false,
            render_decals: false,
            offset_x: 476.0,
            offset_y: 1081.0,
            zoom: 0.5,
            angle_tilt_max: 6.0,
            angle_tilt_min: 6.0,
            stereo_max_separation: 0.015,
            stereo_zero_parallax_displacement: 0.1,
            stereo_offset: 0.0,
            overwrite_global_stereo3d: false,
            image: String::new(),
            backglass_image_full_desktop: String::new(),
            backglass_image_full_fullscreen: String::new(),
            backglass_image_full_single_screen: None,
            image_backdrop_night_day: false,
            image_color_grade: String::new(),
            ball_image: String::new(),
            ball_spherical_mapping: None,
            ball_image_front: String::new(),
            env_image: String::new(),
            notes: None,
            screen_shot: String::new(),
            display_backdrop: false,
            glass_height: 400.0,
            table_height: 0.0,
            playfield_material: "".to_string(),
            backdrop_color: 0x232323ff, // bgra
            global_difficulty: 0.2,
            light_ambient: 0x000000ff, // TODO what is the format for all these?
            light0_emission: 0xfffff0ff, // TODO is this correct?
            light_height: 5000.0,
            light_range: 4000000.0,
            light_emission_scale: 4000000.0,
            env_emission_scale: 2.0,
            global_emission_scale: 0.52,
            ao_scale: 1.75,
            ssr_scale: None, //1.0,
            table_sound_volume: 1.0,
            table_music_volume: 1.0,
            table_adaptive_vsync: -1,
            use_reflection_for_balls: -1,
            playfield_reflection_strength: 0.2941177,
            use_trail_for_balls: -1,
            ball_decal_mode: false,
            ball_playfield_reflection_strength: 1.0,
            default_bulb_intensity_scale_on_ball: None, //1.0,
            ball_trail_strength: quantize_unsigned(8, 0.4901961),
            user_detail_level: 5,
            overwrite_global_detail_level: false,
            overwrite_global_day_night: false,
            show_grid: true,
            reflect_elements_on_playfield: true,
            use_aal: -1,
            use_fxaa: -1,
            use_ao: -1,
            use_ssr: None, //-1,
            bloom_strength: 1.8,
            materials_size: 0,
            materials: Vec::new(),
            materials_physics: None,
            materials_new: None,
            render_probes: None,
            gameitems_size: 0,
            sounds_size: 0,
            images_size: 0,
            fonts_size: 0,
            collections_size: 0,
            name: "Table1".to_string(), // seems to be the default name
            custom_colors: vec![],      //[Color::BLACK; 16],
            protection_data: None,
            code: StringWithEncoding::empty(),
            bg_view_horizontal_offset_desktop: None,
            bg_view_vertical_offset_desktop: None,
            bg_window_top_x_offset_desktop: None,
            bg_window_top_y_offset_desktop: None,
            bg_window_top_z_offset_desktop: None,
            bg_window_bottom_x_offset_desktop: None,
            bg_window_bottom_y_offset_desktop: None,
            bg_window_bottom_z_offset_desktop: None,
            bg_view_mode_fullscreen: None,
            bg_view_horizontal_offset_fullscreen: None,
            bg_view_vertical_offset_fullscreen: None,
            bg_window_top_x_offset_fullscreen: None,
            bg_window_top_y_offset_fullscreen: None,
            bg_window_top_z_offset_fullscreen: None,
            bg_window_bottom_x_offset_fullscreen: None,
            bg_window_bottom_y_offset_fullscreen: None,
            bg_window_bottom_z_offset_fullscreen: None,
            bg_view_mode_full_single_screen: None,
            bg_view_horizontal_offset_full_single_screen: None,
            bg_view_vertical_offset_full_single_screen: None,
            bg_window_top_x_offset_full_single_screen: None,
            bg_window_top_y_offset_full_single_screen: None,
            bg_window_top_z_offset_full_single_screen: None,
            bg_window_bottom_x_offset_full_single_screen: None,
            bg_window_bottom_y_offset_full_single_screen: None,
            bg_window_bottom_z_offset_full_single_screen: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Record {
    name: String,
    data: Vec<u8>,
}

pub fn write_all_gamedata_records(gamedata: &GameData, version: &Version) -> Vec<u8> {
    let mut writer = BiffWriter::new();
    // order is important
    writer.write_tagged_f32("LEFT", gamedata.left);
    writer.write_tagged_f32("TOPX", gamedata.top);
    writer.write_tagged_f32("RGHT", gamedata.right);
    writer.write_tagged_f32("BOTM", gamedata.bottom);
    if let Some(clmo) = gamedata.clmo {
        writer.write_tagged_u32("CLMO", clmo);
    }

    if version.u32() >= 1080 && !gamedata.is_10_8_0_beta1_to_beta4 {
        if let Some(efss) = gamedata.bg_enable_fss {
            writer.write_tagged_bool("EFSS", efss);
        }
    }
    if let Some(vsm0) = gamedata.bg_view_mode_desktop {
        writer.write_tagged_u32("VSM0", vsm0);
    }
    writer.write_tagged_f32("ROTA", gamedata.bg_rotation_desktop);
    writer.write_tagged_f32("INCL", gamedata.bg_inclination_desktop);
    writer.write_tagged_f32("LAYB", gamedata.bg_layback_desktop);
    writer.write_tagged_f32("FOVX", gamedata.bg_fov_desktop);
    writer.write_tagged_f32("XLTX", gamedata.bg_offset_x_desktop);
    writer.write_tagged_f32("XLTY", gamedata.bg_offset_y_desktop);
    writer.write_tagged_f32("XLTZ", gamedata.bg_offset_z_desktop);
    writer.write_tagged_f32("SCLX", gamedata.bg_scale_x_desktop);
    writer.write_tagged_f32("SCLY", gamedata.bg_scale_y_desktop);
    writer.write_tagged_f32("SCLZ", gamedata.bg_scale_z_desktop);

    if let Some(hof0) = gamedata.bg_view_horizontal_offset_desktop {
        writer.write_tagged_f32("HOF0", hof0);
    }
    if let Some(vof0) = gamedata.bg_view_vertical_offset_desktop {
        writer.write_tagged_f32("VOF0", vof0);
    }
    if let Some(wtx0) = gamedata.bg_window_top_x_offset_desktop {
        writer.write_tagged_f32("WTX0", wtx0);
    }
    if let Some(wty0) = gamedata.bg_window_top_y_offset_desktop {
        writer.write_tagged_f32("WTY0", wty0);
    }
    if let Some(wtz0) = gamedata.bg_window_top_z_offset_desktop {
        writer.write_tagged_f32("WTZ0", wtz0);
    }
    if let Some(wbx0) = gamedata.bg_window_bottom_x_offset_desktop {
        writer.write_tagged_f32("WBX0", wbx0);
    }
    if let Some(wby0) = gamedata.bg_window_bottom_y_offset_desktop {
        writer.write_tagged_f32("WBY0", wby0);
    }
    if let Some(wbz0) = gamedata.bg_window_bottom_z_offset_desktop {
        writer.write_tagged_f32("WBZ0", wbz0);
    }

    if let Some(vsm1) = gamedata.bg_view_mode_fullscreen {
        writer.write_tagged_u32("VSM1", vsm1);
    }

    if version.u32() < 1080 || gamedata.is_10_8_0_beta1_to_beta4 {
        if let Some(efss) = gamedata.bg_enable_fss {
            writer.write_tagged_bool("EFSS", efss);
        }
    }
    writer.write_tagged_f32("ROTF", gamedata.bg_rotation_fullscreen);
    writer.write_tagged_f32("INCF", gamedata.bg_inclination_fullscreen);
    writer.write_tagged_f32("LAYF", gamedata.bg_layback_fullscreen);
    writer.write_tagged_f32("FOVF", gamedata.bg_fov_fullscreen);
    writer.write_tagged_f32("XLFX", gamedata.bg_offset_x_fullscreen);
    writer.write_tagged_f32("XLFY", gamedata.bg_offset_y_fullscreen);
    writer.write_tagged_f32("XLFZ", gamedata.bg_offset_z_fullscreen);
    writer.write_tagged_f32("SCFX", gamedata.bg_scale_x_fullscreen);
    writer.write_tagged_f32("SCFY", gamedata.bg_scale_y_fullscreen);
    writer.write_tagged_f32("SCFZ", gamedata.bg_scale_z_fullscreen);
    if let Some(hof1) = gamedata.bg_view_horizontal_offset_fullscreen {
        writer.write_tagged_f32("HOF1", hof1);
    }
    if let Some(vof1) = gamedata.bg_view_vertical_offset_fullscreen {
        writer.write_tagged_f32("VOF1", vof1);
    }
    if let Some(wtx1) = gamedata.bg_window_top_x_offset_fullscreen {
        writer.write_tagged_f32("WTX1", wtx1);
    }
    if let Some(wty1) = gamedata.bg_window_top_y_offset_fullscreen {
        writer.write_tagged_f32("WTY1", wty1);
    }
    if let Some(wtz1) = gamedata.bg_window_top_z_offset_fullscreen {
        writer.write_tagged_f32("WTZ1", wtz1);
    }
    if let Some(wbx1) = gamedata.bg_window_bottom_x_offset_fullscreen {
        writer.write_tagged_f32("WBX1", wbx1);
    }
    if let Some(wby1) = gamedata.bg_window_bottom_y_offset_fullscreen {
        writer.write_tagged_f32("WBY1", wby1);
    }
    if let Some(wbz1) = gamedata.bg_window_bottom_z_offset_fullscreen {
        writer.write_tagged_f32("WBZ1", wbz1);
    }

    if let Some(vsm2) = gamedata.bg_view_mode_full_single_screen {
        writer.write_tagged_u32("VSM2", vsm2);
    }
    if let Some(rofs) = gamedata.bg_rotation_full_single_screen {
        writer.write_tagged_f32("ROFS", rofs);
    }
    if let Some(infs) = gamedata.bg_inclination_full_single_screen {
        writer.write_tagged_f32("INFS", infs);
    }
    if let Some(lafs) = gamedata.bg_layback_full_single_screen {
        writer.write_tagged_f32("LAFS", lafs);
    }
    if let Some(fofs) = gamedata.bg_fov_full_single_screen {
        writer.write_tagged_f32("FOFS", fofs);
    }
    if let Some(xlxs) = gamedata.bg_offset_x_full_single_screen {
        writer.write_tagged_f32("XLXS", xlxs);
    }
    if let Some(xlys) = gamedata.bg_offset_y_full_single_screen {
        writer.write_tagged_f32("XLYS", xlys);
    }
    if let Some(xlzs) = gamedata.bg_offset_z_full_single_screen {
        writer.write_tagged_f32("XLZS", xlzs);
    }
    if let Some(scxs) = gamedata.bg_scale_x_full_single_screen {
        writer.write_tagged_f32("SCXS", scxs);
    }
    if let Some(scys) = gamedata.bg_scale_y_full_single_screen {
        writer.write_tagged_f32("SCYS", scys);
    }
    if let Some(sczs) = gamedata.bg_scale_z_full_single_screen {
        writer.write_tagged_f32("SCZS", sczs);
    }
    if let Some(hof2) = gamedata.bg_view_horizontal_offset_full_single_screen {
        writer.write_tagged_f32("HOF2", hof2);
    }
    if let Some(vof2) = gamedata.bg_view_vertical_offset_full_single_screen {
        writer.write_tagged_f32("VOF2", vof2);
    }
    if let Some(wtx2) = gamedata.bg_window_top_x_offset_full_single_screen {
        writer.write_tagged_f32("WTX2", wtx2);
    }
    if let Some(wty2) = gamedata.bg_window_top_y_offset_full_single_screen {
        writer.write_tagged_f32("WTY2", wty2);
    }
    if let Some(wtz2) = gamedata.bg_window_top_z_offset_full_single_screen {
        writer.write_tagged_f32("WTZ2", wtz2);
    }
    if let Some(wbx2) = gamedata.bg_window_bottom_x_offset_full_single_screen {
        writer.write_tagged_f32("WBX2", wbx2);
    }
    if let Some(wby2) = gamedata.bg_window_bottom_y_offset_full_single_screen {
        writer.write_tagged_f32("WBY2", wby2);
    }
    if let Some(wbz2) = gamedata.bg_window_bottom_z_offset_full_single_screen {
        writer.write_tagged_f32("WBZ2", wbz2);
    }

    writer.write_tagged_u32("ORRP", gamedata.override_physics);
    if let Some(orpf) = gamedata.override_physics_flipper {
        writer.write_tagged_bool("ORPF", orpf);
    }
    writer.write_tagged_f32("GAVT", gamedata.gravity);
    writer.write_tagged_f32("FRCT", gamedata.friction);
    writer.write_tagged_f32("ELAS", gamedata.elasticity);
    writer.write_tagged_f32("ELFA", gamedata.elastic_falloff);
    writer.write_tagged_f32("PFSC", gamedata.scatter);
    writer.write_tagged_f32("SCAT", gamedata.default_scatter);
    writer.write_tagged_f32("NDGT", gamedata.nudge_time);
    writer.write_tagged_u32("MPGC", gamedata.plunger_normalize);
    writer.write_tagged_bool("MPDF", gamedata.plunger_filter);
    writer.write_tagged_u32("PHML", gamedata.physics_max_loops);
    writer.write_tagged_bool("REEL", gamedata.render_em_reels);
    writer.write_tagged_bool("DECL", gamedata.render_decals);
    writer.write_tagged_f32("OFFX", gamedata.offset_x);
    writer.write_tagged_f32("OFFY", gamedata.offset_y);
    writer.write_tagged_f32("ZOOM", gamedata.zoom);
    writer.write_tagged_f32("SLPX", gamedata.angle_tilt_max);
    writer.write_tagged_f32("SLOP", gamedata.angle_tilt_min);
    writer.write_tagged_f32("MAXS", gamedata.stereo_max_separation);
    writer.write_tagged_f32("ZPD", gamedata.stereo_zero_parallax_displacement);
    writer.write_tagged_f32("STO", gamedata.stereo_offset);
    writer.write_tagged_bool("OGST", gamedata.overwrite_global_stereo3d);
    writer.write_tagged_string("IMAG", &gamedata.image);
    writer.write_tagged_string("BIMG", &gamedata.backglass_image_full_desktop);
    writer.write_tagged_string("BIMF", &gamedata.backglass_image_full_fullscreen);

    if let Some(bims) = &gamedata.backglass_image_full_single_screen {
        writer.write_tagged_string("BIMS", bims);
    }
    writer.write_tagged_bool("BIMN", gamedata.image_backdrop_night_day);
    writer.write_tagged_string("IMCG", &gamedata.image_color_grade);
    writer.write_tagged_string("BLIM", &gamedata.ball_image);
    if let Some(ball_spherical_mapping) = gamedata.ball_spherical_mapping {
        writer.write_tagged_bool("BLSM", ball_spherical_mapping);
    }
    writer.write_tagged_string("BLIF", &gamedata.ball_image_front);
    writer.write_tagged_string("EIMG", &gamedata.env_image);
    if let Some(notes) = &gamedata.notes {
        writer.write_tagged_string("NOTX", notes);
    }
    writer.write_tagged_string("SSHT", &gamedata.screen_shot);
    writer.write_tagged_bool("FBCK", gamedata.display_backdrop);
    writer.write_tagged_f32("GLAS", gamedata.glass_height);
    writer.write_tagged_f32("TBLH", gamedata.table_height);
    writer.write_tagged_string("PLMA", &gamedata.playfield_material);
    writer.write_tagged_u32("BCLR", gamedata.backdrop_color);
    writer.write_tagged_f32("TDFT", gamedata.global_difficulty);
    writer.write_tagged_u32("LZAM", gamedata.light_ambient);
    writer.write_tagged_u32("LZDI", gamedata.light0_emission);
    writer.write_tagged_f32("LZHI", gamedata.light_height);
    writer.write_tagged_f32("LZRA", gamedata.light_range);
    writer.write_tagged_f32("LIES", gamedata.light_emission_scale);
    writer.write_tagged_f32("ENES", gamedata.env_emission_scale);
    writer.write_tagged_f32("GLES", gamedata.global_emission_scale);
    writer.write_tagged_f32("AOSC", gamedata.ao_scale);
    if let Some(sssc) = gamedata.ssr_scale {
        writer.write_tagged_f32("SSSC", sssc);
    }
    writer.write_tagged_f32("SVOL", gamedata.table_sound_volume);
    writer.write_tagged_f32("MVOL", gamedata.table_music_volume);
    writer.write_tagged_i32("AVSY", gamedata.table_adaptive_vsync);
    writer.write_tagged_i32("BREF", gamedata.use_reflection_for_balls);
    writer.write_tagged_f32("PLST", gamedata.playfield_reflection_strength);
    writer.write_tagged_i32("BTRA", gamedata.use_trail_for_balls);
    writer.write_tagged_bool("BDMO", gamedata.ball_decal_mode);
    writer.write_tagged_f32("BPRS", gamedata.ball_playfield_reflection_strength);
    if let Some(dbis) = gamedata.default_bulb_intensity_scale_on_ball {
        writer.write_tagged_f32("DBIS", dbis);
    }
    writer.write_tagged_u32("BTST", gamedata.ball_trail_strength);
    writer.write_tagged_u32("ARAC", gamedata.user_detail_level);
    writer.write_tagged_bool("OGAC", gamedata.overwrite_global_detail_level);
    writer.write_tagged_bool("OGDN", gamedata.overwrite_global_day_night);
    writer.write_tagged_bool("GDAC", gamedata.show_grid);
    writer.write_tagged_bool("REOP", gamedata.reflect_elements_on_playfield);
    writer.write_tagged_i32("UAAL", gamedata.use_aal);
    writer.write_tagged_i32("UFXA", gamedata.use_fxaa);
    writer.write_tagged_i32("UAOC", gamedata.use_ao);
    if let Some(ussr) = gamedata.use_ssr {
        writer.write_tagged_i32("USSR", ussr);
    }
    writer.write_tagged_f32("BLST", gamedata.bloom_strength);
    writer.write_tagged_u32("MASI", gamedata.materials_size);
    writer.write_tagged_data("MATE", &gamedata.materials);
    if let Some(phma) = &gamedata.materials_physics {
        writer.write_tagged_data("PHMA", phma);
    }
    if let Some(materials_new) = &gamedata.materials_new {
        for mat in materials_new {
            // TODO proper new material reading?
            writer.write_tagged_data("MATR", mat);
        }
    }
    // multiple RPRB // added in 10.8.x
    if let Some(render_probes) = &gamedata.render_probes {
        for render_probe in render_probes {
            writer.write_tagged_data("RPRB", render_probe);
        }
    }
    writer.write_tagged_u32("SEDT", gamedata.gameitems_size);
    writer.write_tagged_u32("SSND", gamedata.sounds_size);
    writer.write_tagged_u32("SIMG", gamedata.images_size);
    writer.write_tagged_u32("SFNT", gamedata.fonts_size);
    writer.write_tagged_u32("SCOL", gamedata.collections_size);
    writer.write_tagged_wide_string("NAME", &gamedata.name);
    // TODO proper color writing
    writer.write_tagged_data("CCUS", &gamedata.custom_colors);
    if let Some(protection_data) = &gamedata.protection_data {
        writer.write_tagged_data("SECB", protection_data);
    }
    writer.write_tagged_string_with_encoding_no_size("CODE", &gamedata.code);

    writer.close(true);
    // TODO how do we get rid of this extra copy?
    writer.get_data().to_vec()
}

pub fn read_all_gamedata_records(input: &[u8], version: &Version) -> GameData {
    let mut reader = BiffReader::new(input);
    let mut gamedata = GameData::default();
    let mut previous_tag = String::new();
    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();

        let reader: &mut BiffReader<'_> = &mut reader;

        match tag.as_str() {
            "LEFT" => gamedata.left = reader.get_f32(),
            "TOPX" => gamedata.top = reader.get_f32(),
            "RGHT" => gamedata.right = reader.get_f32(),
            "BOTM" => gamedata.bottom = reader.get_f32(),
            "CLMO" => gamedata.clmo = Some(reader.get_u32()),
            "VSM0" => gamedata.bg_view_mode_desktop = Some(reader.get_u32()),
            "ROTA" => gamedata.bg_rotation_desktop = reader.get_f32(),
            "INCL" => gamedata.bg_inclination_desktop = reader.get_f32(),
            "LAYB" => gamedata.bg_layback_desktop = reader.get_f32(),
            "FOVX" => gamedata.bg_fov_desktop = reader.get_f32(),
            "XLTX" => gamedata.bg_offset_x_desktop = reader.get_f32(),
            "XLTY" => gamedata.bg_offset_y_desktop = reader.get_f32(),
            "XLTZ" => gamedata.bg_offset_z_desktop = reader.get_f32(),
            "SCLX" => gamedata.bg_scale_x_desktop = reader.get_f32(),
            "SCLY" => gamedata.bg_scale_y_desktop = reader.get_f32(),
            "SCLZ" => gamedata.bg_scale_z_desktop = reader.get_f32(),
            "EFSS" => {
                if version.u32() == 1080 && previous_tag != "BOTM" {
                    gamedata.is_10_8_0_beta1_to_beta4 = true;
                }
                gamedata.bg_enable_fss = Some(reader.get_bool())
            }
            "HOF0" => gamedata.bg_view_horizontal_offset_desktop = Some(reader.get_f32()),
            "VOF0" => gamedata.bg_view_vertical_offset_desktop = Some(reader.get_f32()),
            "WTX0" => gamedata.bg_window_top_x_offset_desktop = Some(reader.get_f32()),
            "WTY0" => gamedata.bg_window_top_y_offset_desktop = Some(reader.get_f32()),
            "WTZ0" => gamedata.bg_window_top_z_offset_desktop = Some(reader.get_f32()),
            "WBX0" => gamedata.bg_window_bottom_x_offset_desktop = Some(reader.get_f32()),
            "WBY0" => gamedata.bg_window_bottom_y_offset_desktop = Some(reader.get_f32()),
            "WBZ0" => gamedata.bg_window_bottom_z_offset_desktop = Some(reader.get_f32()),
            "VSM1" => gamedata.bg_view_mode_fullscreen = Some(reader.get_u32()),
            "ROTF" => gamedata.bg_rotation_fullscreen = reader.get_f32(),
            "INCF" => gamedata.bg_inclination_fullscreen = reader.get_f32(),
            "LAYF" => gamedata.bg_layback_fullscreen = reader.get_f32(),
            "FOVF" => gamedata.bg_fov_fullscreen = reader.get_f32(),
            "XLFX" => gamedata.bg_offset_x_fullscreen = reader.get_f32(),
            "XLFY" => gamedata.bg_offset_y_fullscreen = reader.get_f32(),
            "XLFZ" => gamedata.bg_offset_z_fullscreen = reader.get_f32(),
            "SCFX" => gamedata.bg_scale_x_fullscreen = reader.get_f32(),
            "SCFY" => gamedata.bg_scale_y_fullscreen = reader.get_f32(),
            "SCFZ" => gamedata.bg_scale_z_fullscreen = reader.get_f32(),
            "HOF1" => gamedata.bg_view_horizontal_offset_fullscreen = Some(reader.get_f32()),
            "VOF1" => gamedata.bg_view_vertical_offset_fullscreen = Some(reader.get_f32()),
            "WTX1" => gamedata.bg_window_top_x_offset_fullscreen = Some(reader.get_f32()),
            "WTY1" => gamedata.bg_window_top_y_offset_fullscreen = Some(reader.get_f32()),
            "WTZ1" => gamedata.bg_window_top_z_offset_fullscreen = Some(reader.get_f32()),
            "WBX1" => gamedata.bg_window_bottom_x_offset_fullscreen = Some(reader.get_f32()),
            "WBY1" => gamedata.bg_window_bottom_y_offset_fullscreen = Some(reader.get_f32()),
            "WBZ1" => gamedata.bg_window_bottom_z_offset_fullscreen = Some(reader.get_f32()),
            "VSM2" => gamedata.bg_view_mode_full_single_screen = Some(reader.get_u32()),
            "ROFS" => gamedata.bg_rotation_full_single_screen = Some(reader.get_f32()),
            "INFS" => gamedata.bg_inclination_full_single_screen = Some(reader.get_f32()),
            "LAFS" => gamedata.bg_layback_full_single_screen = Some(reader.get_f32()),
            "FOFS" => gamedata.bg_fov_full_single_screen = Some(reader.get_f32()),
            "XLXS" => gamedata.bg_offset_x_full_single_screen = Some(reader.get_f32()),
            "XLYS" => gamedata.bg_offset_y_full_single_screen = Some(reader.get_f32()),
            "XLZS" => gamedata.bg_offset_z_full_single_screen = Some(reader.get_f32()),
            "SCXS" => gamedata.bg_scale_x_full_single_screen = Some(reader.get_f32()),
            "SCYS" => gamedata.bg_scale_y_full_single_screen = Some(reader.get_f32()),
            "SCZS" => gamedata.bg_scale_z_full_single_screen = Some(reader.get_f32()),
            "HOF2" => {
                gamedata.bg_view_horizontal_offset_full_single_screen = Some(reader.get_f32())
            }
            "VOF2" => gamedata.bg_view_vertical_offset_full_single_screen = Some(reader.get_f32()),
            "WTX2" => gamedata.bg_window_top_x_offset_full_single_screen = Some(reader.get_f32()),
            "WTY2" => gamedata.bg_window_top_y_offset_full_single_screen = Some(reader.get_f32()),
            "WTZ2" => gamedata.bg_window_top_z_offset_full_single_screen = Some(reader.get_f32()),
            "WBX2" => {
                gamedata.bg_window_bottom_x_offset_full_single_screen = Some(reader.get_f32())
            }
            "WBY2" => {
                gamedata.bg_window_bottom_y_offset_full_single_screen = Some(reader.get_f32())
            }
            "WBZ2" => {
                gamedata.bg_window_bottom_z_offset_full_single_screen = Some(reader.get_f32())
            }
            "ORRP" => gamedata.override_physics = reader.get_u32(),
            "ORPF" => gamedata.override_physics_flipper = Some(reader.get_bool()),
            "GAVT" => gamedata.gravity = reader.get_f32(),
            "FRCT" => gamedata.friction = reader.get_f32(),
            "ELAS" => gamedata.elasticity = reader.get_f32(),
            "ELFA" => gamedata.elastic_falloff = reader.get_f32(),
            "PFSC" => gamedata.scatter = reader.get_f32(),
            "SCAT" => gamedata.default_scatter = reader.get_f32(),
            "NDGT" => gamedata.nudge_time = reader.get_f32(),
            "MPGC" => gamedata.plunger_normalize = reader.get_u32(),
            "MPDF" => gamedata.plunger_filter = reader.get_bool(),
            "PHML" => gamedata.physics_max_loops = reader.get_u32(),
            "REEL" => gamedata.render_em_reels = reader.get_bool(),
            "DECL" => gamedata.render_decals = reader.get_bool(),
            "OFFX" => gamedata.offset_x = reader.get_f32(),
            "OFFY" => gamedata.offset_y = reader.get_f32(),
            "ZOOM" => gamedata.zoom = reader.get_f32(),
            "SLPX" => gamedata.angle_tilt_max = reader.get_f32(),
            "SLOP" => gamedata.angle_tilt_min = reader.get_f32(),
            "MAXS" => gamedata.stereo_max_separation = reader.get_f32(),
            "ZPD" => gamedata.stereo_zero_parallax_displacement = reader.get_f32(),
            "STO" => gamedata.stereo_offset = reader.get_f32(),
            "OGST" => gamedata.overwrite_global_stereo3d = reader.get_bool(),
            "IMAG" => gamedata.image = reader.get_string(),
            "BIMG" => gamedata.backglass_image_full_desktop = reader.get_string(),
            "BIMF" => gamedata.backglass_image_full_fullscreen = reader.get_string(),
            "BIMS" => gamedata.backglass_image_full_single_screen = Some(reader.get_string()),
            "BIMN" => gamedata.image_backdrop_night_day = reader.get_bool(),
            "IMCG" => gamedata.image_color_grade = reader.get_string(),
            "BLIM" => gamedata.ball_image = reader.get_string(),
            "BLSM" => gamedata.ball_spherical_mapping = Some(reader.get_bool()),
            "BLIF" => gamedata.ball_image_front = reader.get_string(),
            "EIMG" => gamedata.env_image = reader.get_string(),
            "NOTX" => gamedata.notes = Some(reader.get_string()),
            "SSHT" => gamedata.screen_shot = reader.get_string(),
            "FBCK" => gamedata.display_backdrop = reader.get_bool(),
            "GLAS" => gamedata.glass_height = reader.get_f32(),
            "TBLH" => gamedata.table_height = reader.get_f32(),
            "PLMA" => gamedata.playfield_material = reader.get_string(),
            "BCLR" => gamedata.backdrop_color = reader.get_u32(),
            "TDFT" => gamedata.global_difficulty = reader.get_f32(),
            "LZAM" => gamedata.light_ambient = reader.get_u32(),
            "LZDI" => gamedata.light0_emission = reader.get_u32(),
            "LZHI" => gamedata.light_height = reader.get_f32(),
            "LZRA" => gamedata.light_range = reader.get_f32(),
            "LIES" => gamedata.light_emission_scale = reader.get_f32(),
            "ENES" => gamedata.env_emission_scale = reader.get_f32(),
            "GLES" => gamedata.global_emission_scale = reader.get_f32(),
            "AOSC" => gamedata.ao_scale = reader.get_f32(),
            "SSSC" => gamedata.ssr_scale = Some(reader.get_f32()),
            "SVOL" => gamedata.table_sound_volume = reader.get_f32(),
            "MVOL" => gamedata.table_music_volume = reader.get_f32(),
            "AVSY" => gamedata.table_adaptive_vsync = reader.get_i32(),
            "BREF" => gamedata.use_reflection_for_balls = reader.get_i32(),
            "PLST" => gamedata.playfield_reflection_strength = reader.get_f32(),
            "BTRA" => gamedata.use_trail_for_balls = reader.get_i32(),
            "BDMO" => gamedata.ball_decal_mode = reader.get_bool(),
            "BPRS" => gamedata.ball_playfield_reflection_strength = reader.get_f32(),
            "DBIS" => gamedata.default_bulb_intensity_scale_on_ball = Some(reader.get_f32()),
            "BTST" => {
                // TODO do we need this QuantizedUnsignedBits for some of the float fields?
                gamedata.ball_trail_strength = reader.get_u32();
            }
            "ARAC" => gamedata.user_detail_level = reader.get_u32(),
            "OGAC" => gamedata.overwrite_global_detail_level = reader.get_bool(),
            "OGDN" => gamedata.overwrite_global_day_night = reader.get_bool(),
            "GDAC" => gamedata.show_grid = reader.get_bool(),
            "REOP" => gamedata.reflect_elements_on_playfield = reader.get_bool(),
            "UAAL" => gamedata.use_aal = reader.get_i32(),
            "UFXA" => gamedata.use_fxaa = reader.get_i32(),
            "UAOC" => gamedata.use_ao = reader.get_i32(),
            "USSR" => gamedata.use_ssr = Some(reader.get_i32()),
            "BLST" => gamedata.bloom_strength = reader.get_f32(),
            "MASI" => gamedata.materials_size = reader.get_u32(),
            "MATE" => gamedata.materials = reader.get_record_data(false).to_vec(),
            "PHMA" => gamedata.materials_physics = Some(reader.get_record_data(false).to_vec()),
            // see https://github.com/vpinball/vpinball/blob/1a994086a6092733272fda36a2f449753a1ca21a/pintable.cpp#L4429
            "MATR" => {
                let data = reader.get_record_data(false).to_vec();
                gamedata
                    .materials_new
                    .get_or_insert_with(Vec::new)
                    .push(data);
            }
            "RPRB" => {
                let data = reader.get_record_data(false).to_vec();
                gamedata
                    .render_probes
                    .get_or_insert_with(Vec::new)
                    .push(data);
            }
            "SEDT" => gamedata.gameitems_size = reader.get_u32(),
            "SSND" => gamedata.sounds_size = reader.get_u32(),
            "SIMG" => gamedata.images_size = reader.get_u32(),
            "SFNT" => gamedata.fonts_size = reader.get_u32(),
            "SCOL" => gamedata.collections_size = reader.get_u32(),
            "NAME" => gamedata.name = reader.get_wide_string(),
            "CCUS" => gamedata.custom_colors = reader.get_record_data(false).to_vec(),
            "SECB" => gamedata.protection_data = Some(reader.get_record_data(false).to_vec()),
            "CODE" => {
                let len = reader.get_u32_no_remaining_update();
                // at least a the time of 1060, some code was still encoded in latin1
                gamedata.code = reader.get_str_with_encoding_no_remaining_update(len as usize);
            }
            other => {
                let data = reader.get_record_data(false);
                println!("unhandled tag {} {} bytes", other, data.len());
            }
        };
        previous_tag = tag;
    }
    gamedata
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn read_write_empty() {
        let game_data = GameData::default();
        let version: Version = Version::new(1074);
        let bytes = write_all_gamedata_records(&game_data, &version);
        let read_game_data = read_all_gamedata_records(&bytes, &version);

        assert_eq!(game_data, read_game_data);
    }

    #[test]
    fn read_write() {
        let gamedata = GameData {
            left: 1.0,
            right: 2.0,
            top: 3.0,
            bottom: 4.0,
            clmo: None,
            bg_view_mode_desktop: Some(1),
            bg_rotation_desktop: 1.0,
            bg_inclination_desktop: 2.0,
            bg_layback_desktop: 3.0,
            bg_fov_desktop: 4.0,
            bg_offset_x_desktop: 1.0,
            bg_offset_y_desktop: 2.0,
            bg_offset_z_desktop: 3.0,
            bg_scale_x_desktop: 3.3,
            bg_scale_y_desktop: 2.2,
            bg_scale_z_desktop: 1.1,
            bg_enable_fss: Some(true),
            bg_rotation_fullscreen: 1.0,
            bg_inclination_fullscreen: 2.0,
            bg_layback_fullscreen: 3.0,
            bg_fov_fullscreen: 4.0,
            bg_offset_x_fullscreen: 1.0,
            bg_offset_y_fullscreen: 2.0,
            bg_offset_z_fullscreen: 3.0,
            bg_scale_x_fullscreen: 3.3,
            bg_scale_y_fullscreen: 2.2,
            bg_scale_z_fullscreen: 1.1,
            bg_rotation_full_single_screen: Some(1.0),
            bg_inclination_full_single_screen: Some(2.0),
            bg_layback_full_single_screen: Some(3.0),
            bg_fov_full_single_screen: Some(4.0),
            bg_offset_x_full_single_screen: Some(1.0),
            bg_offset_y_full_single_screen: Some(2.0),
            bg_offset_z_full_single_screen: Some(3.0),
            bg_scale_x_full_single_screen: Some(3.3),
            bg_scale_y_full_single_screen: Some(2.2),
            bg_scale_z_full_single_screen: Some(1.1),
            override_physics: 1,
            override_physics_flipper: Some(true),
            gravity: 1.0,
            friction: 0.1,
            elasticity: 0.2,
            elastic_falloff: 0.3,
            scatter: 0.2,
            default_scatter: 0.1,
            nudge_time: 3.0,
            plunger_normalize: 105,
            plunger_filter: true,
            physics_max_loops: 30,
            render_em_reels: true,
            render_decals: true,
            offset_x: 50.0,
            offset_y: 60.0,
            zoom: 0.2,
            angle_tilt_max: 4.0,
            angle_tilt_min: 3.0,
            stereo_max_separation: 0.03,
            stereo_zero_parallax_displacement: 0.2,
            stereo_offset: 0.5,
            overwrite_global_stereo3d: true,
            image: String::from("test image"),
            backglass_image_full_desktop: String::from("test desktop"),
            backglass_image_full_fullscreen: String::from("test fullscreen"),
            backglass_image_full_single_screen: Some(String::from("test single screen")),
            image_backdrop_night_day: true,
            image_color_grade: String::from("test color grade"),
            ball_image: String::from("test ball image"),
            ball_spherical_mapping: Some(true),
            ball_image_front: String::from("test ball image"),
            env_image: String::from("test env image"),
            notes: Some(String::from("test notes")),
            screen_shot: String::from("test screenshot"),
            display_backdrop: true,
            glass_height: 234.0,
            table_height: 12.0,
            playfield_material: "material_pf".to_string(),
            backdrop_color: 0x333333ff,
            global_difficulty: 0.3,
            light_ambient: 0x11223344,
            light0_emission: 0xaabbccdd,
            light_height: 4000.0,
            light_range: 50000.0,
            light_emission_scale: 1.2,
            env_emission_scale: 1.23,
            global_emission_scale: 0.111,
            ao_scale: 0.9,
            ssr_scale: Some(0.5),
            table_sound_volume: 0.6,
            table_music_volume: 0.5,
            table_adaptive_vsync: 1,
            use_reflection_for_balls: 1,
            playfield_reflection_strength: 0.02,
            use_trail_for_balls: -3,
            ball_decal_mode: true,
            ball_playfield_reflection_strength: 2.0,
            default_bulb_intensity_scale_on_ball: Some(2.0),
            ball_trail_strength: quantize_unsigned(8, 0.55),
            user_detail_level: 9,
            overwrite_global_detail_level: true,
            overwrite_global_day_night: true,
            show_grid: false,
            reflect_elements_on_playfield: false,
            use_aal: -10,
            use_fxaa: -2,
            use_ao: -3,
            use_ssr: Some(-4),
            bloom_strength: 0.3,
            materials_size: 0,
            gameitems_size: 0,
            sounds_size: 0,
            images_size: 0,
            fonts_size: 0,
            collections_size: 0,
            materials: vec![],
            materials_physics: Some(vec![]),
            materials_new: None,
            render_probes: None,
            name: String::from("test name"),
            custom_colors: vec![1, 1, 2, 4], // [Color::RED; 16],
            protection_data: None,
            code: StringWithEncoding::from("test code wit some unicode: Ǣ"),
            bg_view_horizontal_offset_desktop: None,
            bg_view_vertical_offset_desktop: None,
            bg_window_top_x_offset_desktop: None,
            bg_window_top_y_offset_desktop: None,
            bg_window_top_z_offset_desktop: None,
            bg_window_bottom_x_offset_desktop: None,
            bg_window_bottom_y_offset_desktop: None,
            bg_window_bottom_z_offset_desktop: None,
            bg_view_mode_fullscreen: None,
            bg_view_horizontal_offset_fullscreen: None,
            bg_view_vertical_offset_fullscreen: None,
            bg_window_top_x_offset_fullscreen: None,
            bg_window_top_y_offset_fullscreen: None,
            bg_window_top_z_offset_fullscreen: None,
            bg_window_bottom_x_offset_fullscreen: None,
            bg_window_bottom_y_offset_fullscreen: None,
            bg_window_bottom_z_offset_fullscreen: None,
            bg_view_mode_full_single_screen: None,
            bg_view_horizontal_offset_full_single_screen: None,
            bg_view_vertical_offset_full_single_screen: None,
            bg_window_top_x_offset_full_single_screen: None,
            bg_window_top_y_offset_full_single_screen: None,
            bg_window_top_z_offset_full_single_screen: None,
            bg_window_bottom_x_offset_full_single_screen: None,
            bg_window_bottom_y_offset_full_single_screen: None,
            bg_window_bottom_z_offset_full_single_screen: None,
            is_10_8_0_beta1_to_beta4: false,
        };
        let version = Version::new(1074);
        let bytes = write_all_gamedata_records(&gamedata, &version);
        let read_game_data = read_all_gamedata_records(&bytes, &version);

        assert_eq!(gamedata, read_game_data);
    }
}
