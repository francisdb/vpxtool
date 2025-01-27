use crate::guifrontend::VpxConfig;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;
use bevy::window::{
    WindowBackendScaleFactorChanged, WindowCreated, WindowMode, WindowRef, WindowResized,
    WindowResolution,
};
use shared::vpinball_config::WindowType;
use shared::vpinball_config::{VPinballConfig, WindowInfo};
use std::time::Duration;

/// Layers are used to assign meshes to a specific camera/window
/// For UI elements this works differently, they are assigned to a camera using the TargetCamera component
/// see https://github.com/bevyengine/bevy/issues/12468
pub(crate) const DMD_LAYER: RenderLayers = RenderLayers::layer(1);

#[derive(Component)]
pub(crate) struct PlayfieldCamera;

#[derive(Component)]
pub(crate) struct DMDCamera;

pub struct WindowingPlugin;

impl Plugin for WindowingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (setup_windows, correct_window_size_and_position).chain(),
        );
        app.add_systems(
            Update,
            (log_window_moved, log_window_resized, resize_on_crated),
        );
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
    mut scale_events: EventReader<WindowBackendScaleFactorChanged>,
    window_query: Query<(Entity, &Window)>,
    time: Res<Time>,
    mut timer: Local<Timer>,
    mut resized_windows: Local<ResizedWindows>,
) {
    timer.tick(time.delta());

    for event in scale_events.read() {
        if let Some((entity, scaled_window)) = window_query
            .iter()
            .find(|(entity, _)| entity == &event.window)
        {
            let name = window_name(entity, scaled_window);
            info!(
                "Scale factor for {} changed to {}",
                name, event.scale_factor
            );
        }
    }

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

pub(crate) fn setup_windows(mut commands: Commands, vpx_config: Res<VpxConfig>) {
    let playfield_window_camera = commands.spawn((PlayfieldCamera, Camera2d)).id();

    #[cfg(debug_assertions)]
    label_window(&mut commands, playfield_window_camera, "Playfield");

    if let Some(window_info) = vpx_config.config.get_window_info(&WindowType::B2SBackglass) {
        let mut window = Window {
            name: Some("backglass".to_owned()),
            title: "Vpxtool - Backglass".to_owned(),
            resizable: false,
            focused: false,
            decorations: false,
            skip_taskbar: true,
            ..default()
        };
        setup_window(&window_info, &mut window, &WindowType::B2SBackglass);

        info!("Creating backglass window");
        let vpx_window_info = VpxWindowInfo {
            window_info: window_info.clone(),
        };
        let backglass_window = commands.spawn((window, vpx_window_info)).id();
        let backglass_window_camera = commands
            .spawn((
                DMDCamera,
                Camera2d,
                Camera {
                    target: RenderTarget::Window(WindowRef::Entity(backglass_window)),
                    ..default()
                },
                DMD_LAYER,
            ))
            .id();

        #[cfg(debug_assertions)]
        label_window(&mut commands, backglass_window_camera, "Backglass");
    }
}

fn label_window(commands: &mut Commands, window_camera: Entity, name: &str) {
    let window_label_node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(12.0),
        left: Val::Px(12.0),
        ..default()
    };
    commands.spawn((
        Text::new(name),
        TextFont::from_font_size(8.0),
        window_label_node,
        TargetCamera(window_camera),
    ));
}

pub(crate) fn setup_playfield_window(vpinball_config: &VPinballConfig) -> Window {
    let mut window = Window {
        name: Some("playfield".to_string()),
        title: "Vpxtool - Playfield".to_string(),
        ..Default::default()
    };
    if let Some(playfield_info) = vpinball_config.get_window_info(&WindowType::Playfield) {
        setup_window(&playfield_info, &mut window, &WindowType::Playfield);
    }
    window
}

#[derive(Component)]
pub(crate) struct VpxWindowInfo {
    window_info: WindowInfo,
}

/// NOTE: This is NOT triggered on startup for the primary window creation.
fn resize_on_crated(
    mut window_created_event_reader: EventReader<WindowCreated>,
    mut window_query: Query<(Entity, &VpxWindowInfo, &mut Window)>,
) {
    for event in window_created_event_reader.read() {
        // find the window
        for (window_entity, info, mut window) in window_query.iter_mut() {
            if window_entity == event.window {
                let window_name = window_name(window_entity, &window);
                info!(
                    "Window {} created: {:?} scale_factor {}",
                    window_name,
                    event,
                    &window.scale_factor()
                );

                if cfg!(target_os = "linux") {
                    // TODO document wayland vs x11
                    // In vpinball the window sizes are configured in physical pixels.
                    // The bevy window constructor requires setting the resolution in physical pixels.
                    // However it seems that this size is used as the logical size once the window is created.
                    // Therefore on startup we again configure the window size to the physical size.

                    if let (Some(width), Some(height)) =
                        (info.window_info.width, info.window_info.height)
                    {
                        info!(
                            "Setting window {} physical resolution to {}x{}",
                            window_name, width, height
                        );
                        window.resolution.set_physical_resolution(width, height);
                    }
                }

                if cfg!(target_os = "macos") {
                    // TODO implement
                }
            }
        }
    }
}

pub fn correct_window_size_and_position(
    mut window_query: Query<(Entity, &mut Window)>,
    vpx_config: Res<VpxConfig>,
) {
    // #[cfg(target_os = "linux")] is annoying because it causes clippy to complain about dead code
    if cfg!(target_os = "linux") {
        // TODO document wayland vs x11
        // In vpinball the window sizes are configured in physical pixels.
        // The bevy window constructor requires setting the resolution in physical pixels.
        // However it seems that this size is used as the logical size once the window is created.
        // Therefore on startup we again configure the window size to the physical size.
        for (window_entity, mut window) in window_query.iter_mut() {
            info!(
                "Window {} resolution: {}x{} scale factor {}",
                window_name(window_entity, &window),
                window.resolution.width(),
                window.resolution.height(),
                window.resolution.scale_factor(),
            );
            if window.resolution.scale_factor() != 1.0 {
                let window_name = window_name(window_entity, &window);
                info!(
                    "Resizing window {} for Linux with scale factor {}",
                    window_name,
                    window.resolution.scale_factor(),
                );
                let vpinball_config = &vpx_config.config;
                if let Some(playfield) = vpinball_config.get_window_info(&WindowType::Playfield) {
                    if let (Some(physical_width), Some(physical_height)) =
                        (playfield.width, playfield.height)
                    {
                        window
                            .resolution
                            .set_physical_resolution(physical_width, physical_height);
                        if let (Some(x), Some(y)) = (playfield.x, playfield.y) {
                            info!("Setting window {} position to {}, {}", window_name, x, y);
                            window.position = WindowPosition::At(IVec2::new(x as i32, y as i32));
                        }
                        //window.set_changed();
                    }
                }
            }
        }
    }

    // only on macOS
    // #[cfg(target_os = "macos")] is annoying because it causes clippy to complain about dead code
    if cfg!(target_os = "macos") {
        let (window_entity, mut window) = window_query.single_mut();
        if window.resolution.scale_factor() != 1.0 {
            let window_name = window_name(window_entity, &window);
            info!(
                "Repositioning window {} for macOS with scale factor {}",
                window_name,
                window.resolution.scale_factor(),
            );
            let vpinball_config = &vpx_config.config;
            if let Some(playfield) = vpinball_config.get_window_info(&WindowType::Playfield) {
                if let (Some(logical_x), Some(logical_y)) = (playfield.x, playfield.y) {
                    // For macOS with scales factor > 1 this is not correct but we don't know the scale
                    // factor before the window is created.
                    let physical_x = logical_x as f32 * window.resolution.scale_factor();
                    let physical_y = logical_y as f32 * window.resolution.scale_factor();
                    info!(
                        "Setting window {} position to {}, {}",
                        window_name, physical_x, physical_y,
                    );
                    // this will apply the width as if it was set in logical pixels
                    window.position =
                        WindowPosition::At(IVec2::new(physical_x as i32, physical_y as i32));
                    window.set_changed();
                }
            }
        }
    }
}

fn setup_window(window_info: &WindowInfo, window: &mut Window, window_type: &WindowType) {
    let position = if let (Some(x), Some(y)) = (window_info.x, window_info.y) {
        // For macOS with scale factor > 1 this is not correct, but we don't know the scale
        // factor before the window is created. We will correct the position later using the
        // system "correct_mac_window_size".
        let physical_x = x as i32;
        let physical_y = y as i32;
        WindowPosition::At(IVec2::new(physical_x, physical_y))
    } else {
        WindowPosition::default()
    };

    // TODO get the scaling factor for the primary monitor using winit
    // https://docs.rs/winit/0.22.2/winit/monitor/struct.MonitorHandle.html#method.scale_factor

    let resolution = if let (Some(width), Some(height)) = (window_info.width, window_info.height) {
        WindowResolution::new(width as f32, height as f32)
    } else {
        WindowResolution::default()
    };
    let mode = if window_info.fullscreen {
        WindowMode::Fullscreen(MonitorSelection::Primary)
    } else {
        WindowMode::Windowed
    };

    info!(
        "Positioning window {:?} window at {:?}, resolution {:?}, mode {:?}",
        window_type, position, resolution, mode
    );

    window.position = position;
    window.resolution = resolution;
    window.mode = mode;
}

fn window_name(entity: Entity, window: &Window) -> String {
    match &window.name {
        Some(name) => name.clone(),
        None => format!("unnamed/{}", entity),
    }
}
