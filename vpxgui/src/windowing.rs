use crate::guifrontend::VpxConfig;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;
use bevy::window::{
    PrimaryWindow, WindowBackendScaleFactorChanged, WindowCreated, WindowMode, WindowRef,
    WindowResized, WindowResolution,
};
use shared::vpinball_config::WindowType;
use shared::vpinball_config::{VPinballConfig, WindowInfo};
use std::time::Duration;

/// Layers are used to assign meshes to a specific camera/window
/// For UI elements this works differently, they are assigned to a camera using the TargetCamera component
/// see https://github.com/bevyengine/bevy/issues/12468
pub(crate) const BACKGLASS_LAYER: RenderLayers = RenderLayers::layer(1);
pub(crate) const B2SDMD_LAYER: RenderLayers = RenderLayers::layer(2);
pub(crate) const PINMAME_LAYER: RenderLayers = RenderLayers::layer(3);
pub(crate) const FLEXDMD_LAYER: RenderLayers = RenderLayers::layer(4);

pub(crate) const PUPBACKGLASS_LAYER: RenderLayers = RenderLayers::layer(5);

pub(crate) const PUPDMD_LAYER: RenderLayers = RenderLayers::layer(6);
pub(crate) const PUPFULLDMD_LAYER: RenderLayers = RenderLayers::layer(7);
pub(crate) const PUPTOPPER_LAYER: RenderLayers = RenderLayers::layer(8);

#[derive(Component)]
pub(crate) struct PlayfieldCamera;

#[derive(Component)]
pub(crate) struct DMDCamera;

pub struct WindowingPlugin;

impl Plugin for WindowingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_windows);
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
                let name = window_name(entity, window);
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
                "Window [{}] Scale factor changed to {}",
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
                let name = window_name(entity, window);
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

fn layer_for_window_type(window_type: WindowType) -> RenderLayers {
    match window_type {
        WindowType::B2SBackglass => BACKGLASS_LAYER,
        WindowType::B2SDMD => B2SDMD_LAYER,
        WindowType::PinMAME => PINMAME_LAYER,
        WindowType::FlexDMD => FLEXDMD_LAYER,
        WindowType::PUPBackglass => PUPBACKGLASS_LAYER,
        WindowType::PUPDMD => PUPDMD_LAYER,
        WindowType::PUPFullDMD => PUPFULLDMD_LAYER,
        WindowType::PUPTopper => PUPTOPPER_LAYER,
        other => {
            warn!("No layer set up for WindowType: {:?}", other);
            BACKGLASS_LAYER
        }
    }
}

pub(crate) fn setup_windows(
    mut commands: Commands,
    vpx_config: Res<VpxConfig>,
    primary_window_query: Query<Entity, With<PrimaryWindow>>,
) {
    setup_playfield_window2(&mut commands, &vpx_config, primary_window_query);

    setup_other_window(&mut commands, &vpx_config, WindowType::B2SBackglass);
    setup_other_window(&mut commands, &vpx_config, WindowType::B2SDMD);
    setup_other_window(&mut commands, &vpx_config, WindowType::PinMAME);
    setup_other_window(&mut commands, &vpx_config, WindowType::FlexDMD);
    setup_other_window(&mut commands, &vpx_config, WindowType::PUPBackglass);
    setup_other_window(&mut commands, &vpx_config, WindowType::PUPDMD);
    setup_other_window(&mut commands, &vpx_config, WindowType::PUPFullDMD);
    setup_other_window(&mut commands, &vpx_config, WindowType::PUPTopper);
}

fn setup_other_window(commands: &mut Commands, vpx_config: &VpxConfig, window_type: WindowType) {
    if let Some(window_info) = vpx_config.config.get_window_info(window_type) {
        info!("Window [{}] vpinball config {:?}", window_type, window_info);
        let mut window = Window {
            name: Some(window_type.to_string()),
            title: format!("Vpxtool - {}", window_type),
            resizable: true,
            focused: false,
            decorations: false,
            skip_taskbar: true,
            resize_constraints: WindowResizeConstraints {
                min_width: 64.0,
                min_height: 64.0,
                max_width: f32::INFINITY,
                max_height: f32::INFINITY,
            },
            ..default()
        };
        setup_window(&window_info, &mut window, window_type);

        info!("Window [{}] spawn", window_type);
        let vpx_window_info = VpxWindowInfo {
            window_info: window_info.clone(),
        };
        let window_entity = commands.spawn((window, vpx_window_info)).id();
        let render_layers = layer_for_window_type(window_type);
        let window_camera = commands
            .spawn((
                DMDCamera,
                Camera2d,
                Camera {
                    target: RenderTarget::Window(WindowRef::Entity(window_entity)),
                    ..default()
                },
                render_layers,
            ))
            .id();

        #[cfg(debug_assertions)]
        label_window(commands, window_camera, &window_type.to_string());
    } else {
        info!("Window [{}] disabled", window_type);
    }
}

fn setup_playfield_window2(
    commands: &mut Commands,
    vpx_config: &VpxConfig,
    primary_window_query: Query<Entity, With<PrimaryWindow>>,
) {
    let playfield_window_camera = commands.spawn((PlayfieldCamera, Camera2d)).id();
    #[cfg(debug_assertions)]
    label_window(commands, playfield_window_camera, "Playfield");
    // add VpxWindowInfo to playfield window, don't think we can do this in the WindowPlugin
    if let Some(playfield_info) = vpx_config.config.get_window_info(WindowType::Playfield) {
        let vpx_window_info = VpxWindowInfo {
            window_info: playfield_info.clone(),
        };
        commands
            .entity(primary_window_query.single())
            .insert(vpx_window_info);
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
        decorations: false,
        ..Default::default()
    };
    if let Some(playfield_info) = vpinball_config.get_window_info(WindowType::Playfield) {
        setup_window(&playfield_info, &mut window, WindowType::Playfield);
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
                correct_window_size_and_position(window_entity, &info, &mut window);
            }
        }
    }
}

fn correct_window_size_and_position(
    window_entity: Entity,
    vpx_info: &&VpxWindowInfo,
    window: &mut Mut<Window>,
) {
    // #[cfg(target_os = "linux")] is annoying because it causes clippy to complain about dead code
    if cfg!(target_os = "linux") {
        // For Linux in the vpinball config, the window sizes are configured in physical pixels.
        // The bevy window constructor requires setting the resolution in logical pixels.
        // So we created the window with a small size to avoid repositioning.
        // Therefore, on startup/created we again configure the window size to the physical size.
        info!(
            "Window [{}] resolution: {}x{} scale factor {}",
            window_name(window_entity, &window),
            window.resolution.width(),
            window.resolution.height(),
            window.resolution.scale_factor(),
        );
        if window.resolution.scale_factor() != 1.0 {
            let window_name = window_name(window_entity, &window);
            info!(
                "Window [{}] Resizing for Linux with scale factor {}",
                window_name,
                window.resolution.scale_factor(),
            );
            let info = &vpx_info.window_info;
            if let (Some(physical_width), Some(physical_height)) = (info.width, info.height) {
                window
                    .resolution
                    .set_physical_resolution(physical_width, physical_height);
                if let (Some(x), Some(y)) = (info.x, info.y) {
                    info!("Window [{}] Setting position to {}, {}", window_name, x, y);
                    window.position = WindowPosition::At(IVec2::new(x as i32, y as i32));
                }
                //window.set_changed();
            }
        }
    }

    // only on macOS
    // #[cfg(target_os = "macos")] is annoying because it causes clippy to complain about dead code
    if cfg!(target_os = "macos") {
        // On macOS the window sizes are configured in logical pixels which corresponds to the
        // size that is configured in the vpinball config. Nothing to do there.
        // But the position is configured in logical pixels. We need to correct the position
        // for the scale factor.
        if window.resolution.scale_factor() != 1.0 {
            let window_name = window_name(window_entity, &window);
            info!(
                "Window [{}] Repositioning for macOS with scale factor {}",
                window_name,
                window.resolution.scale_factor(),
            );
            let info = &vpx_info.window_info;
            if let (Some(logical_x), Some(logical_y)) = (info.x, info.y) {
                // For macOS with scales factor > 1 this is not correct but we don't know the scale
                // factor before the window is created.
                let physical_x = logical_x as f32 * window.resolution.scale_factor();
                let physical_y = logical_y as f32 * window.resolution.scale_factor();
                info!(
                    "Window [{}] setting position to {}, {}",
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

fn setup_window(window_info: &WindowInfo, window: &mut Window, window_type: WindowType) {
    let position = if let (Some(x), Some(y)) = (window_info.x, window_info.y) {
        // For macOS with scale factor > 1 the x and y coordinates in the config
        // are logical coordinates. But we don't know the scale
        // factor before the window is created. We will correct the position later
        let physical_x = x as i32;
        let physical_y = y as i32;
        WindowPosition::At(IVec2::new(physical_x, physical_y))
    } else {
        WindowPosition::default()
    };

    // When on macOS we have the correct logical resolution defined in the vpinball config
    let resolution = if cfg!(not(target_os = "linux")) {
        window.resolution = WindowResolution::default();
        if let (Some(width), Some(height)) = (window_info.width, window_info.height) {
            WindowResolution::new(width as f32, height as f32)
        } else {
            WindowResolution::default()
        }
    } else {
        // TODO validate on windows
        // TODO since these sizes are seen as logical sizes, we start with a very small size no to have repositioning
        // Then on the WindowCreated event we will resize the window to the correct size
        // see https://github.com/bevyengine/bevy/issues/17563
        WindowResolution::new(100.0, 100.0)
    };

    let mode = if window_info.fullscreen {
        WindowMode::Fullscreen(MonitorSelection::Primary)
    } else {
        WindowMode::Windowed
    };

    info!(
        "Window [{}] Positioning at {:?}, resolution {:?}, mode {:?}",
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
