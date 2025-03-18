use anyhow::{Context, Result};
use log::*;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use std::process::ExitCode;
use std::time::Duration;
use vpxtool_shared::config;
use vpxtool_shared::vpinball_config::{VPinballConfig, WindowType};

// TODO get these from the config
const DEFAULT_PLAYFIELD_WINDOW_WIDTH: u32 = 1080;
const DEFAULT_PLAYFIELD_WINDOW_HEIGHT: u32 = 1920;

fn main() -> ExitCode {
    // Initialize with INFO level by default, can be overridden with RUST_LOG env var
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None) // Disable timestamps
        .format_target(false) // Disable module path
        .init();
    run().unwrap_or_else(|err| {
        error!("Unhandled error: {}", err);
        ExitCode::FAILURE
    })
}

fn run() -> Result<ExitCode> {
    info!("Initializing vpxtool frontend");
    let config_opt = config::load_config().context("failed to load config")?;
    let Some((_, config)) = config_opt else {
        warn!("No config file found. Run vpxtool to create one.");
        return Ok(ExitCode::FAILURE);
    };

    let vpinball_ini_path = config.vpinball_ini_file();
    let vpinball_config = if vpinball_ini_path.exists() {
        VPinballConfig::read(&vpinball_ini_path)?
    } else {
        warn!(
            "vpinball.ini not found at {:?}, using empty config",
            vpinball_ini_path
        );
        VPinballConfig::default()
    };

    let sdl_context = sdl3::init().context("Failed to initialize SDL")?;
    let sdl_video = sdl_context.video()?;
    let sdl_audio = sdl_context.audio()?;
    // let sdl_ttf = sdl3::ttf::init()?;
    // sdl_image context not required
    // for now we don't do video, needs vlc or ffmpeg

    let playfield_config = vpinball_config.get_window_info(WindowType::Playfield);
    let playfield_width = playfield_config
        .as_ref()
        .and_then(|w| w.width)
        .unwrap_or(DEFAULT_PLAYFIELD_WINDOW_WIDTH);
    let playfield_height = playfield_config
        .as_ref()
        .and_then(|w| w.height)
        .unwrap_or(DEFAULT_PLAYFIELD_WINDOW_HEIGHT);
    let playfield_x = playfield_config.as_ref().and_then(|w| w.x).unwrap_or(0);
    let playfield_y = playfield_config.as_ref().and_then(|w| w.y).unwrap_or(0);
    let playfield_window = sdl_video
        .window("Playfield", playfield_width, playfield_height)
        .position(playfield_x as i32, playfield_y as i32)
        .borderless()
        .build()
        .context("Failed to create primary window")?;

    let backglass_config = vpinball_config.get_window_info(WindowType::B2SBackglass);
    let backglass_width = backglass_config
        .as_ref()
        .and_then(|w| w.width)
        .unwrap_or(DEFAULT_PLAYFIELD_WINDOW_WIDTH);
    let backglass_height = backglass_config
        .as_ref()
        .and_then(|w| w.height)
        .unwrap_or(DEFAULT_PLAYFIELD_WINDOW_HEIGHT);
    let backglass_x = backglass_config.as_ref().and_then(|w| w.x).unwrap_or(0);
    let backglass_y = backglass_config.as_ref().and_then(|w| w.y).unwrap_or(0);
    let backglass_window = sdl_video
        .window("Backglass", backglass_width, backglass_height)
        .position(backglass_x as i32, backglass_y as i32)
        .borderless()
        .build()
        .context("Failed to create backglass window")?;

    let mut playfield_canvas = playfield_window.into_canvas();
    let mut backglass_canvas = backglass_window.into_canvas();

    playfield_canvas.set_draw_color(Color::RGB(0, 255, 255));
    playfield_canvas.clear();
    playfield_canvas.present();
    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        playfield_canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        playfield_canvas.clear();

        backglass_canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        backglass_canvas.clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        playfield_canvas.present();
        backglass_canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(ExitCode::SUCCESS)
}
