use std::io;
use std::{fmt::Debug, io::BufWriter};

use quick_xml::Writer;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct POV {
    pub desktop: ModePov,
    pub fullscreen: ModePov,
    pub fullsinglescreen: ModePov,
    pub customsettings: Option<Customsettings>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ModePov {
    #[serde(rename = "LayoutMode")]
    pub layout_mode: Option<u32>,
    pub inclination: f32,
    pub fov: f32,
    pub layback: f32,
    pub lookat: Option<f32>,
    pub rotation: f32,
    pub xscale: f32,
    pub yscale: f32,
    pub zscale: f32,
    pub xoffset: f32,
    pub yoffset: f32,
    pub zoffset: f32,
    #[serde(rename = "ViewHOfs")]
    pub view_hofs: Option<f32>,
    #[serde(rename = "ViewVOfs")]
    pub view_vofs: Option<f32>,
    #[serde(rename = "WindowTopXOfs")]
    pub window_top_xofs: Option<f32>,
    #[serde(rename = "WindowTopYOfs")]
    pub window_top_yofs: Option<f32>,
    #[serde(rename = "WindowTopZOfs")]
    pub window_top_zofs: Option<f32>,
    #[serde(rename = "WindowBottomXOfs")]
    pub window_bottom_xofs: Option<f32>,
    #[serde(rename = "WindowBottomYOfs")]
    pub window_bottom_yofs: Option<f32>,
    #[serde(rename = "WindowBottomZOfs")]
    pub window_bottom_zofs: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Customsettings {
    #[serde(rename = "SSAA")]
    pub ssaa: i32,
    #[serde(rename = "postprocAA")]
    pub postproc_aa: i32,
    #[serde(rename = "ingameAO")]
    pub ingame_ao: i32,
    #[serde(rename = "ScSpReflect")]
    pub sc_sp_reflect: i32,
    #[serde(rename = "FPSLimiter")]
    pub fps_limiter: i32,
    #[serde(rename = "OverwriteDetailsLevel")]
    pub overwrite_details_level: i32,
    #[serde(rename = "DetailsLevel")]
    pub details_level: i32,
    #[serde(rename = "BallReflection")]
    pub ball_reflection: i32,
    #[serde(rename = "BallTrail")]
    pub ball_trail: i32,
    #[serde(rename = "BallTrailStrength")]
    pub ball_trail_strength: f32,
    #[serde(rename = "OverwriteNightDay")]
    pub overwrite_night_day: i32,
    #[serde(rename = "NightDayLevel")]
    pub night_day_level: i32,
    #[serde(rename = "GameplayDifficulty")]
    pub gameplay_difficulty: f32,
    #[serde(rename = "PhysicsSet")]
    pub physics_set: i32,
    #[serde(rename = "IncludeFlipperPhysics")]
    pub include_flipper_physics: i32,
    #[serde(rename = "SoundVolume")]
    pub sound_volume: i32,
    #[serde(rename = "MusicVolume")]
    pub sound_music_volume: i32,
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<POV, io::Error> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    quick_xml::de::from_reader(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub fn save<P: AsRef<Path>>(path: P, pov: &POV) -> Result<(), io::Error> {
    let file = std::fs::File::create(path)?;
    let mut writer = Writer::new(BufWriter::new(file));
    // TODO is there a better way to do this?
    writer.write_serializable("POV", pov).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use super::*;
    use pretty_assertions::assert_eq;
    use testdir::testdir;

    #[test]
    fn test_write_read() -> std::io::Result<()> {
        let dir: PathBuf = testdir!();
        let test_pov_path = dir.join("test.pov");

        let pov = POV {
            desktop: ModePov {
                layout_mode: Some(0),
                inclination: 0.0,
                fov: 46.399986,
                layback: 0.0,
                lookat: Some(39.99996),
                rotation: 0.0,
                xscale: 1.0,
                yscale: 0.988,
                zscale: 1.0,
                xoffset: 0.0,
                yoffset: 46.00006,
                zoffset: -320.0,
                view_hofs: Some(0.0),
                view_vofs: Some(0.0),
                window_top_xofs: Some(0.0),
                window_top_yofs: Some(0.0),
                window_top_zofs: Some(370.54193),
                window_bottom_xofs: Some(0.0),
                window_bottom_yofs: Some(0.0),
                window_bottom_zofs: Some(138.95322),
            },
            fullscreen: ModePov {
                layout_mode: Some(2),
                inclination: 2.0,
                fov: 77.0,
                layback: 50.20001,
                lookat: Some(25.0),
                rotation: 270.0,
                xscale: 1.0540019,
                yscale: 1.2159998,
                zscale: 1.0,
                xoffset: 0.0,
                yoffset: 186.54193,
                zoffset: 952.1677,
                view_hofs: Some(-1.4901161e-8),
                view_vofs: Some(-11.100006),
                window_top_xofs: Some(-5.0),
                window_top_yofs: Some(-275.0),
                window_top_zofs: Some(365.54193),
                window_bottom_xofs: Some(1.0),
                window_bottom_yofs: Some(-34.0),
                window_bottom_zofs: Some(230.95322),
            },
            fullsinglescreen: ModePov {
                layout_mode: Some(0),
                inclination: 0.0,
                fov: 45.0,
                layback: 0.0,
                lookat: Some(52.0),
                rotation: 0.0,
                xscale: 1.0540019,
                yscale: 1.2159998,
                zscale: 1.0,
                xoffset: 0.0,
                yoffset: 186.54193,
                zoffset: 952.1677,
                view_hofs: Some(-1.4901161e-8),
                view_vofs: Some(-11.100006),
                window_top_xofs: Some(-5.0),
                window_top_yofs: Some(-275.0),
                window_top_zofs: Some(365.54193),
                window_bottom_xofs: Some(1.0),
                window_bottom_yofs: Some(-34.0),
                window_bottom_zofs: Some(230.95322),
            },
            customsettings: Some(Customsettings {
                ssaa: -1,
                postproc_aa: -1,
                ingame_ao: -1,
                sc_sp_reflect: -1,
                fps_limiter: -1,
                overwrite_details_level: 0,
                details_level: 5,
                ball_reflection: -1,
                ball_trail: -1,
                ball_trail_strength: 0.2,
                overwrite_night_day: 1,
                night_day_level: 63,
                gameplay_difficulty: 19.999998,
                physics_set: 0,
                include_flipper_physics: 0,
                sound_volume: 100,
                sound_music_volume: 100,
            }),
        };
        save(&test_pov_path, &pov)?;
        let loaded = load(&test_pov_path)?;
        assert_eq!(pov, loaded);
        Ok(())
    }

    #[test]
    fn test_load() -> std::io::Result<()> {
        let expected = POV {
            desktop: ModePov {
                layout_mode: Some(0),
                inclination: 0.0,
                fov: 46.399986,
                layback: 0.0,
                lookat: Some(39.99996),
                rotation: 0.0,
                xscale: 1.0,
                yscale: 0.988,
                zscale: 1.0,
                xoffset: 0.0,
                yoffset: 46.00006,
                zoffset: -320.0,
                view_hofs: Some(0.0),
                view_vofs: Some(0.0),
                window_top_xofs: Some(0.0),
                window_top_yofs: Some(0.0),
                window_top_zofs: Some(370.54193),
                window_bottom_xofs: Some(0.0),
                window_bottom_yofs: Some(0.0),
                window_bottom_zofs: Some(138.95322),
            },
            fullscreen: ModePov {
                layout_mode: Some(2),
                inclination: 2.0,
                fov: 77.0,
                layback: 50.20001,
                lookat: Some(25.0),
                rotation: 270.0,
                xscale: 1.0540019,
                yscale: 1.2159998,
                zscale: 1.0,
                xoffset: 0.0,
                yoffset: 186.54193,
                zoffset: 952.1677,
                view_hofs: Some(-1.4901161e-8),
                view_vofs: Some(-11.100006),
                window_top_xofs: Some(-5.0),
                window_top_yofs: Some(-275.0),
                window_top_zofs: Some(365.54193),
                window_bottom_xofs: Some(1.0),
                window_bottom_yofs: Some(-34.0),
                window_bottom_zofs: Some(230.95322),
            },
            fullsinglescreen: ModePov {
                layout_mode: Some(0),
                inclination: 0.0,
                fov: 45.0,
                layback: 0.0,
                lookat: Some(52.0),
                rotation: 0.0,
                xscale: 1.2,
                yscale: 1.1,
                zscale: 1.0,
                xoffset: 0.0,
                yoffset: 30.0,
                zoffset: -50.0,
                view_hofs: Some(0.0),
                view_vofs: Some(0.0),
                window_top_xofs: Some(0.0),
                window_top_yofs: Some(0.0),
                window_top_zofs: Some(370.54193),
                window_bottom_xofs: Some(0.0),
                window_bottom_yofs: Some(0.0),
                window_bottom_zofs: Some(138.95322),
            },
            customsettings: Some(Customsettings {
                ssaa: -1,
                postproc_aa: -1,
                ingame_ao: -1,
                sc_sp_reflect: -1,
                fps_limiter: -1,
                overwrite_details_level: 0,
                details_level: 5,
                ball_reflection: -1,
                ball_trail: -1,
                ball_trail_strength: 0.2,
                overwrite_night_day: 1,
                night_day_level: 63,
                gameplay_difficulty: 19.999998,
                physics_set: 0,
                include_flipper_physics: 0,
                sound_volume: 100,
                sound_music_volume: 100,
            }),
        };

        let pov = load("testdata/test.pov")?;

        assert_eq!(pov, expected);

        Ok(())
    }

    #[test]
    fn test_load_legacy() -> std::io::Result<()> {
        let expected = POV {
            desktop: ModePov {
                layout_mode: None,
                inclination: 40.59998,
                fov: 46.0,
                layback: 0.0,
                lookat: None,
                rotation: 0.0,
                xscale: 1.0,
                yscale: 0.938001,
                zscale: 1.0,
                xoffset: 0.0,
                yoffset: -26.800074,
                zoffset: -300.0,
                view_hofs: None,
                view_vofs: None,
                window_top_xofs: None,
                window_top_yofs: None,
                window_top_zofs: None,
                window_bottom_xofs: None,
                window_bottom_yofs: None,
                window_bottom_zofs: None,
            },
            fullscreen: ModePov {
                layout_mode: None,
                inclination: 0.0,
                fov: 27.0,
                layback: 76.0,
                lookat: None,
                rotation: 270.0,
                xscale: 1.0,
                yscale: 1.16,
                zscale: 1.0,
                xoffset: -10.0,
                yoffset: 0.0,
                zoffset: -200.0,
                view_hofs: None,
                view_vofs: None,
                window_top_xofs: None,
                window_top_yofs: None,
                window_top_zofs: None,
                window_bottom_xofs: None,
                window_bottom_yofs: None,
                window_bottom_zofs: None,
            },
            fullsinglescreen: ModePov {
                layout_mode: None,
                inclination: 54.0,
                fov: 32.0,
                layback: 0.0,
                lookat: None,
                rotation: 0.0,
                xscale: 1.0,
                yscale: 1.0,
                zscale: 1.0,
                xoffset: 0.0,
                yoffset: 0.0,
                zoffset: -250.0,
                view_hofs: None,
                view_vofs: None,
                window_top_xofs: None,
                window_top_yofs: None,
                window_top_zofs: None,
                window_bottom_xofs: None,
                window_bottom_yofs: None,
                window_bottom_zofs: None,
            },
            customsettings: None,
        };

        let pov = load("testdata/test.legacy.pov")?;

        assert_eq!(pov, expected);

        Ok(())
    }
}
