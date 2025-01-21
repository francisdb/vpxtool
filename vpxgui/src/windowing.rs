use crate::guifrontend::VpxConfig;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode, WindowResized, WindowResolution};
use shared::config::VPinballConfig;

pub struct WindowingPlugin;

impl Plugin for WindowingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, correct_window_size_and_position);
        app.add_systems(Update, (log_window_moved, log_window_resized));
    }
}
#[derive(Default)]
struct MovedWindows {
    windows: HashMap<Entity, IVec2>,
}

struct ResizedWindow {
    logical_width: f32,
    logical_height: f32,
}

#[derive(Default)]
struct ResizedWindows {
    windows: HashMap<Entity, ResizedWindow>,
}

use bevy::utils::HashMap;
use std::time::Duration;

fn log_window_moved(
    mut events: EventReader<WindowMoved>,
    window_query: Query<(Entity, &Window)>,
    time: Res<Time>,
    mut timer: Local<Timer>,
    mut moved_windows: Local<MovedWindows>,
) {
    timer.tick(time.delta());

    for event in events.read() {
        moved_windows.windows.insert(event.window, event.position);
        timer.reset();
        timer.set_duration(Duration::from_millis(500));
    }

    if timer.finished() {
        for (moved_window, position) in moved_windows.windows.iter() {
            if let Some((entity, window)) = window_query
                .iter()
                .find(|(entity, _)| entity == moved_window)
            {
                let name = window_name(entity, &window);
                info!("Window [{}] moved to {},{}", name, position.x, position.y);
            }
        }
        moved_windows.windows.clear();
    }
}

fn log_window_resized(
    mut events: EventReader<WindowResized>,
    window_query: Query<(Entity, &Window)>,
    time: Res<Time>,
    mut timer: Local<Timer>,
    mut resized_windows: Local<ResizedWindows>,
) {
    timer.tick(time.delta());

    for event in events.read() {
        resized_windows.windows.insert(
            event.window,
            ResizedWindow {
                logical_width: event.width,
                logical_height: event.height,
            },
        );
        timer.reset();
        timer.set_duration(Duration::from_millis(500));
    }

    if timer.finished() {
        for (resized_window, resized) in resized_windows.windows.iter() {
            if let Some((entity, window)) = window_query
                .iter()
                .find(|(entity, _)| entity == resized_window)
            {
                let scale_factor = window.scale_factor();
                let physical_width = resized.logical_width * scale_factor;
                let physical_height = resized.logical_height * scale_factor;
                let name = window_name(entity, &window);
                info!(
                    "Window [{}] resized to {}x{} (physical: {}x{})",
                    name,
                    resized.logical_width,
                    resized.logical_height,
                    physical_width,
                    physical_height
                );
            }
        }
        resized_windows.windows.clear();
    }
}

pub(crate) fn setup_playfield_window(vpinball_config: &VPinballConfig) -> Window {
    let mut position = WindowPosition::default();
    let mut mode = WindowMode::Fullscreen(MonitorSelection::Primary);
    let mut resolution = WindowResolution::default();
    if let Some(playfield) = vpinball_config.get_playfield_info() {
        if let (Some(x), Some(y)) = (playfield.x, playfield.y) {
            // For macOS with scale factor > 1 this is not correct but we don't know the scale
            // factor before the window is created. We will correct the position later using the
            // system "correct_mac_window_size".
            info!("Setting window position to x={}, y={}", x, y);
            let physical_x = x as i32;
            let physical_y = y as i32;
            position = WindowPosition::At(IVec2::new(physical_x, physical_y));
        }
        if let (Some(width), Some(height)) = (playfield.width, playfield.height) {
            resolution = WindowResolution::new(width as f32, height as f32);
        }
        mode = if playfield.fullscreen {
            WindowMode::Fullscreen(MonitorSelection::Primary)
        } else {
            WindowMode::Windowed
        };
    }
    info!(
        "Positioning window at {:?}, resolution {:?}",
        position, resolution
    );
    Window {
        name: Some("playfield".to_string()),
        title: "VPXTOOL".to_string(),
        // window_level: WindowLevel::AlwaysOnTop,
        resolution,
        mode, // WindowMode::Windowed,
        position,
        ..Default::default()
    }
}

pub fn correct_window_size_and_position(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    vpx_config: Res<VpxConfig>,
) {
    // only on Linux
    // #[cfg(target_os = "linux")] is annoying because it causes clippy to complain about dead code
    if cfg!(target_os = "linux") {
        // Under wayland the window size is not correct, we need to scale it down.
        // In vpinball the playfield window size is configured in physical pixels.
        // The window constructor will create a window with the size in logical pixels.
        let mut window = window_query.single_mut();
        if window.resolution.scale_factor() != 1.0 {
            info!(
                "Resizing window for Linux with scale factor {}",
                window.resolution.scale_factor(),
            );
            let vpinball_config = &vpx_config.config;
            if let Some(playfield) = vpinball_config.get_playfield_info() {
                if let (Some(physical_width), Some(physical_height)) =
                    (playfield.width, playfield.height)
                {
                    let logical_width = physical_width as f32 / window.resolution.scale_factor();
                    let logical_height = physical_height as f32 / window.resolution.scale_factor();
                    info!(
                        "Setting window size to {}x{}",
                        logical_width, logical_height
                    );
                    window.resolution.set(logical_width, logical_height);
                    if let (Some(x), Some(y)) = (playfield.x, playfield.y) {
                        info!("Setting window position to {}, {}", x, y);
                        window.position = WindowPosition::At(IVec2::new(x as i32, y as i32));
                    }
                    window.set_changed();
                }
            }
        }
    }

    // only on macOS
    // #[cfg(target_os = "macos")] is annoying because it causes clippy to complain about dead code
    if cfg!(target_os = "macos") {
        let mut window = window_query.single_mut();
        if window.resolution.scale_factor() != 1.0 {
            info!(
                "Repositioning window for macOS with scale factor {}",
                window.resolution.scale_factor(),
            );
            let vpinball_config = &vpx_config.config;
            if let Some(playfield) = vpinball_config.get_playfield_info() {
                if let (Some(logical_x), Some(logical_y)) = (playfield.x, playfield.y) {
                    // For macOS with scales factor > 1 this is not correct but we don't know the scale
                    // factor before the window is created.
                    let physical_x = logical_x as f32 * window.resolution.scale_factor();
                    let physical_y = logical_y as f32 * window.resolution.scale_factor();
                    info!("Setting window position to {}, {}", physical_x, physical_y,);
                    // this will apply the width as if it was set in logical pixels
                    window.position =
                        WindowPosition::At(IVec2::new(physical_x as i32, physical_y as i32));
                    window.set_changed();
                }
            }
        }
    }
}

fn window_name(entity: Entity, window: &&Window) -> String {
    match &window.name {
        Some(name) => name.clone(),
        None => format!("unnamed/{}", entity),
    }
}
