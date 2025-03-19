use anyhow::{Context, Result};
use log::*;
use sdl3::event::Event;
use sdl3::image::LoadTexture;
use sdl3::keyboard::Keycode;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{Texture, TextureCreator};
use sdl3::surface::Surface;
use sdl3::video::WindowContext;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};
use vpxtool_shared::indexer::{IndexedTable, VoidProgress};
use vpxtool_shared::vpinball_config::{VPinballConfig, WindowType};
use vpxtool_shared::{config, indexer};

// TODO get these from the config
const DEFAULT_PLAYFIELD_WINDOW_WIDTH: u32 = 1080;
const DEFAULT_PLAYFIELD_WINDOW_HEIGHT: u32 = 1920;

const INITIAL_SHIFT_SPEED: f32 = 1.0;
const RAMP_UP_DURATION: f32 = 10.0; // seconds until max speed
const MAX_SHIFT_SPEED: f32 = 200.0; // Maximum jump per second

/// Handles shift key presses to change the current table index
/// The shift key can be held down to scroll through the tables
/// The speed of the scroll increases over time
struct ShiftHandler {
    shift_start: Option<Instant>,
    shift_applied: f32,
}

impl ShiftHandler {
    fn new() -> Self {
        Self {
            shift_start: None,
            shift_applied: 0.0,
        }
    }

    fn handle_shift(&mut self, now: Instant, delta_time: Duration, direction: i16) -> i16 {
        if self.shift_start.is_none() {
            self.shift_start = Some(now);
            return direction;
        }

        let held_duration = now.duration_since(self.shift_start.unwrap()).as_secs_f32();
        let speed = (INITIAL_SHIFT_SPEED
            + (held_duration / RAMP_UP_DURATION) * (MAX_SHIFT_SPEED - INITIAL_SHIFT_SPEED))
            .min(MAX_SHIFT_SPEED);

        self.shift_applied += delta_time.as_secs_f32() * speed * direction as f32;
        if self.shift_applied.abs() >= 1.0 {
            let index_change = self.shift_applied.floor() as i16 * direction;
            self.shift_applied = self.shift_applied.fract();
            return index_change;
        }

        0
    }

    fn update(
        &mut self,
        now: Instant,
        delta_time: Duration,
        keystates: &sdl3::keyboard::KeyboardState,
    ) -> i16 {
        if keystates.is_scancode_pressed(sdl3::keyboard::Scancode::LShift) {
            self.handle_shift(now, delta_time, -1)
        } else if keystates.is_scancode_pressed(sdl3::keyboard::Scancode::RShift) {
            self.handle_shift(now, delta_time, 1)
        } else {
            self.shift_start = None;
            self.shift_applied = 0.0;
            0
        }
    }
}

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
    let Some((_, resolved_config)) = config_opt else {
        warn!("No config file found. Run vpxtool to create one.");
        return Ok(ExitCode::FAILURE);
    };

    let vpinball_ini_path = resolved_config.vpinball_ini_file();
    let vpinball_config = if vpinball_ini_path.exists() {
        VPinballConfig::read(&vpinball_ini_path)?
    } else {
        warn!(
            "vpinball.ini not found at {:?}, using empty config",
            vpinball_ini_path
        );
        VPinballConfig::default()
    };

    let index = indexer::index_folder(
        true,
        &resolved_config.tables_folder,
        &resolved_config.tables_index_path,
        Some(&resolved_config.global_pinmame_rom_folder()),
        &VoidProgress,
        Vec::new(),
    )?;
    let tables = {
        let mut loaded = index.tables();
        loaded.sort_by_key(|indexed| display_table_line(indexed).to_lowercase());
        loaded
    };

    if index.is_empty() {
        warn!("No tables found in tables folder");
        return Ok(ExitCode::FAILURE);
    }

    let sdl_context = sdl3::init().context("Failed to initialize SDL")?;
    let sdl_video = sdl_context.video()?;
    //let sdl_audio = sdl_context.audio()?;
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

    let playfield_texture_creator = playfield_canvas.texture_creator();

    info!("Renderer name: {}", playfield_canvas.renderer_name);

    let mut current_index = 0usize;
    let mut current_table = &tables[current_index];

    // Rectangle to define the position and size of the table name text
    // let mut table_name_rect = Rect::new(0, 0, 0, 0);

    // TODO show some kind of generic texture?
    let empty_surface = Surface::new(100, 100, PixelFormat::from(32))?;

    let mut playfield_texture = load_texture(
        &playfield_texture_creator,
        &playfield_image_path(current_table),
        &empty_surface,
    )?;

    playfield_canvas.set_draw_color(Color::RGB(0, 255, 255));
    playfield_canvas.clear();
    playfield_canvas.present();
    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;

    let mut last_time = Instant::now();
    let mut accumulator = 0u64;
    let mut past_time = Instant::now();

    const TARGET_FPS: u64 = 60;
    const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);

    let mut shift_handler = ShiftHandler::new();

    'running: loop {
        let now = Instant::now();
        let delta_time = now.duration_since(past_time);
        past_time = now;

        i = (i + 1) % 255;
        playfield_canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        playfield_canvas.clear();

        // render the playfield texture
        playfield_canvas.copy(&playfield_texture, None, None)?;

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

        // get the keyboard state
        // Increment the shift_applied value based on the time the shift key has been held
        let keystates = event_pump.keyboard_state();
        let index_change = shift_handler.update(now, delta_time, &keystates);

        if index_change != 0 {
            current_index =
                (current_index as i16 + index_change).rem_euclid(tables.len() as i16) as usize;
            current_table = &tables[current_index];
            playfield_texture = load_texture(
                &playfield_texture_creator,
                &playfield_image_path(current_table),
                &empty_surface,
            )?;
        }

        playfield_canvas.present();
        backglass_canvas.present();

        accumulator += 1;
        if now.duration_since(last_time) > Duration::from_secs(1) {
            debug!("FPS: {}", accumulator);
            last_time = now;
            accumulator = 0;
        }

        let frame_duration = Instant::now().duration_since(now);
        if frame_duration < FRAME_TIME {
            std::thread::sleep(FRAME_TIME - frame_duration);
        }
    }

    Ok(ExitCode::SUCCESS)
}

// TODO this is something the indexer should provide
fn playfield_image_path(current_table: &IndexedTable) -> PathBuf {
    current_table
        .path
        .parent()
        .unwrap()
        .join("captures")
        .join("playfield.png")
}

fn load_texture<'a>(
    creator: &'a TextureCreator<WindowContext>,
    path: &Path,
    empty_surface: &Surface,
) -> Result<Texture<'a>> {
    let texture = if path.exists() {
        // load png texture
        let texture = creator.load_texture(path)?;
        info!(
            "Loaded texture from {:?} {} x {}",
            path,
            texture.width(),
            texture.height()
        );
        texture
    } else {
        warn!("Texture not found: {:?}", path);
        // create a dummy texture
        empty_surface.as_texture(creator)?
    };
    Ok(texture)
}

pub(crate) fn display_table_line(table: &IndexedTable) -> String {
    let file_name = table
        .path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    Some(table.table_info.table_name.to_owned())
        .filter(|s| !s.clone().unwrap_or_default().trim().is_empty())
        .map(|s| {
            match s {
                Some(name) => capitalize_first_letter(&name),
                None => capitalize_first_letter(&file_name),
            }
            // TODO we probably want to show both the file name and the table name
        })
        .unwrap_or(file_name)
}
fn capitalize_first_letter(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
}
