mod display;
mod gradient;
mod input;
mod text;
mod ui;
mod vpinball;

use crate::display::{display_file_name, display_table_name};
use crate::input::{AlphabeticJumper, ShiftHandler};
use crate::text::{draw_text_box, render_text};
use crate::vpinball::Vpinball;
use anyhow::{Context, Result};
use log::*;
use sdl3::event::Event;
use sdl3::image::LoadTexture;
use sdl3::keyboard::Keycode;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::rect::Rect;
use sdl3::render::{Texture, TextureCreator};
use sdl3::surface::Surface;
use sdl3::ttf::Sdl3TtfContext;
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

const TITLE_BOTTOM_PADDING: u32 = 40;

#[cfg(target_os = "macos")]
const DEFAULT_FONT_PATH: &str = "/System/Library/Fonts/SFNS.ttf";

#[cfg(target_os = "windows")]
const DEFAULT_FONT_PATH: &str = "C:\\Windows\\Fonts\\Arial.ttf";

#[cfg(not(target_os = "macos"))]
const DEFAULT_FONT_PATH: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";

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

struct GameState {
    vpinball: Option<Vpinball>,
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
        loaded.sort_by_key(|indexed| display_table_name(indexed).to_lowercase());
        loaded
    };

    if index.is_empty() {
        warn!("No tables found in tables folder");
        return Ok(ExitCode::FAILURE);
    }
    let mut game_state = GameState { vpinball: None };
    // make sure sdl is freed when we're done by scoping
    {
        let sdl_context = sdl3::init().context("Failed to initialize SDL")?;
        let sdl_video = sdl_context.video()?;
        //let sdl_audio = sdl_context.audio()?;
        let sdl_ttf = sdl3::ttf::init()?;
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
        let backglass_texture_creator = backglass_canvas.texture_creator();

        info!("Renderer name: {}", playfield_canvas.renderer_name);

        let mut current_index = 0usize;
        let mut current_table = &tables[current_index];
        let mut last_table_index = None;
        let mut title_textures = Vec::new();
        let mut title_rects = Vec::new();

        // Rectangle to define the position and size of the table name text
        // let mut table_name_rect = Rect::new(0, 0, 0, 0);

        // TODO show some kind of generic texture?
        let empty_surface = Surface::new(100, 100, PixelFormat::from(32))?;

        let mut playfield_texture = load_texture(
            &playfield_texture_creator,
            &playfield_image_path(current_table),
            &empty_surface,
        )?;
        let mut backglass_texture = load_texture(
            &backglass_texture_creator,
            &b2sbackglass_image_path(current_table),
            &empty_surface,
        )?;

        // playfield_canvas.set_draw_color(Color::RGB(0, 255, 255));
        // playfield_canvas.clear();
        // playfield_canvas.present();
        let mut event_pump = sdl_context.event_pump()?;

        let mut last_time = Instant::now();
        let mut accumulator = 0u64;
        let mut past_time = Instant::now();

        const TARGET_FPS: u64 = 60;
        const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);

        let mut shift_handler = ShiftHandler::new();
        let mut alpha_jumper = AlphabeticJumper::new();

        'running: loop {
            let now = Instant::now();
            let delta_time = now.duration_since(past_time);
            past_time = now;

            playfield_canvas.clear();
            playfield_canvas.copy(&playfield_texture, None, None)?;

            backglass_canvas.clear();
            backglass_canvas.copy(&backglass_texture, None, None)?;

            for event in event_pump.poll_iter() {
                match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        game_state.vpinball = Some(Vpinball::launch(
                            &resolved_config.vpx_executable,
                            &current_table.path,
                        )?);
                    }
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

            // Handle shift key index changes
            let keystates = event_pump.keyboard_state();
            let index_change = shift_handler.update(now, delta_time, &keystates);
            if index_change != 0 {
                current_index =
                    (current_index as i16 + index_change).rem_euclid(tables.len() as i16) as usize;
                current_table = &tables[current_index];

                // Clear the current table index to force texture recreation
                last_table_index = None;
            }
            // Handle alphabetic jumps with Ctrl keys
            if keystates.is_scancode_pressed(sdl3::keyboard::Scancode::LCtrl) {
                if let Some(new_index) =
                    alpha_jumper.handle_jump(&tables, current_index, -1, |table| {
                        display_table_name(table)
                            .chars()
                            .next()
                            .unwrap_or_default()
                            .to_ascii_lowercase()
                    })
                {
                    current_index = new_index;
                    current_table = &tables[current_index];
                    last_table_index = None;
                }
            } else if keystates.is_scancode_pressed(sdl3::keyboard::Scancode::RCtrl) {
                if let Some(new_index) =
                    alpha_jumper.handle_jump(&tables, current_index, 1, |table| {
                        display_table_name(table)
                            .chars()
                            .next()
                            .unwrap_or_default()
                            .to_ascii_lowercase()
                    })
                {
                    current_index = new_index;
                    current_table = &tables[current_index];
                    last_table_index = None;
                }
            }

            // Only create textures when the table changes
            if last_table_index != Some(current_index) {
                // Load table images
                playfield_texture = load_texture(
                    &playfield_texture_creator,
                    &playfield_image_path(current_table),
                    &empty_surface,
                )?;
                backglass_texture = load_texture(
                    &backglass_texture_creator,
                    &b2sbackglass_image_path(current_table),
                    &empty_surface,
                )?;
                let (textures, rects) = render_title(
                    &sdl_ttf,
                    playfield_height,
                    &playfield_texture_creator,
                    current_table,
                    &Path::new(DEFAULT_FONT_PATH),
                )?;
                title_textures = textures;
                title_rects = rects;
                last_table_index = Some(current_index);
            }

            // Draw the text box
            draw_text_box(&mut playfield_canvas, &title_textures, &title_rects)?;
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
    }
    if let Some(mut vpinball) = game_state.vpinball {
        vpinball.kill()?;
    }

    Ok(ExitCode::SUCCESS)
}

fn render_title<'a>(
    sdl_ttf: &Sdl3TtfContext,
    playfield_height: u32,
    playfield_texture_creator: &'a TextureCreator<WindowContext>,
    current_table: &IndexedTable,
    font_path: &Path,
) -> Result<(Vec<Texture<'a>>, Vec<Rect>)> {
    let table_name = display_table_name(current_table);
    let file_name = display_file_name(current_table);

    // Pre-calculate text dimensions to position correctly
    let mut textures = Vec::new();
    let mut text_rects = Vec::new();

    // Create textures first
    let font_large = sdl_ttf.load_font(font_path, 48.0)?;
    let font_small = sdl_ttf.load_font(font_path, 24.0)?;

    let table_name_texture = render_text(
        &font_large,
        &table_name,
        Color::RGB(255, 255, 255),
        playfield_texture_creator,
    )?;

    let file_name_texture = render_text(
        &font_small,
        &file_name,
        Color::RGB(255, 255, 255),
        playfield_texture_creator,
    )?;

    // Calculate positions with proper spacing
    let margin = 20; // Space between text elements
    let table_height = table_name_texture.query().height as i32;
    let file_height = file_name_texture.query().height as i32;
    let total_height = table_height + margin + file_height;

    // Start position from the bottom with some padding
    let start_y = playfield_height as i32 - TITLE_BOTTOM_PADDING as i32 - total_height;

    // Create rectangles with the correct positions
    let table_rect = Rect::new(
        10,
        start_y,
        table_name_texture.query().width,
        table_name_texture.query().height,
    );

    let file_rect = Rect::new(
        10,
        start_y + table_height + margin, // Position after the table name plus margin
        file_name_texture.query().width,
        file_name_texture.query().height,
    );

    // Add in the correct order for rendering
    textures.push(table_name_texture);
    textures.push(file_name_texture);
    text_rects.push(table_rect);
    text_rects.push(file_rect);

    Ok((textures, text_rects))
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

fn b2sbackglass_image_path(current_table: &IndexedTable) -> PathBuf {
    current_table
        .path
        .parent()
        .unwrap()
        .join("captures")
        .join("backglass.png")
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
