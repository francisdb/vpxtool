use crate::dmd::dmd_plugin;
use crate::event_channel::{ChannelExternalEvent, ExternalEvent, StreamSender};
use crate::flippers::flipper_plugin;
use crate::gradient_background::setup_gradient_background;
use crate::info::show_info;
use crate::list::{display_table_line, list_plugin, SelectedItem};
use crate::loading::{loading_plugin, mark_tables_loaded};
use crate::loading::{LoadingState, TableLoadingEvent};
use crate::menus::*;
use crate::music::{music_plugin, resume_music, suspend_music, ControlMusicEvent};
use crate::process::do_launch;
use crate::wheel::wheel_plugin;
use crate::windowing;
use crate::windowing::WindowingPlugin;
use bevy::prelude::*;
use bevy::window::*;
use bevy_egui::EguiPlugin;
use vpxtool_shared::config::ResolvedConfig;
use vpxtool_shared::indexer::IndexedTable;
use vpxtool_shared::vpinball_config::VPinballConfig;

#[derive(Resource)]
pub struct Config {
    pub config: ResolvedConfig,
}

#[derive(Resource)]
pub struct VpxConfig {
    pub config: VPinballConfig,
}

#[derive(Resource)]
pub struct VpxTables {
    pub indexed_tables: Vec<IndexedTable>,
}

#[derive(Resource, Debug)]
pub(crate) struct Globals {
    pub vpinball_running: bool,
}

#[allow(clippy::too_many_arguments)]
fn launcher(
    keys: Res<ButtonInput<KeyCode>>,
    mut control_music_event_writer: EventWriter<ControlMusicEvent>,
    stream_sender: Res<StreamSender>,
    config: Res<Config>,
    selected_item: Res<SelectedItem>,
    mut globals: ResMut<Globals>,
    //mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    tables: Res<VpxTables>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        if globals.vpinball_running {
            warn!("Visual Pinball already running");
            return;
        };

        if let Some(selected_item) = selected_item.index {
            //let mut playfield_window = window_query.single_mut();
            suspend_music(&mut control_music_event_writer);
            let table = tables.indexed_tables.get(selected_item).unwrap();
            //playfield_window.visible = false;
            globals.vpinball_running = true;
            do_launch(
                stream_sender.clone(),
                &table.path,
                &config.config.vpx_executable,
            );
        }
    }
}

fn quit_on_q_or_window_closed(
    keys: Res<ButtonInput<KeyCode>>,
    mut window_events: EventReader<WindowEvent>,
    mut app_exit_event_writer: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        app_exit_event_writer.send(AppExit::Success);
    }
    // closing any window closes the app
    for event in window_events.read() {
        if let WindowEvent::WindowCloseRequested(_) = event {
            app_exit_event_writer.send(AppExit::Success);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_external_events(
    mut reader: EventReader<ExternalEvent>,
    mut music_event_writer: EventWriter<ControlMusicEvent>,
    mut table_loading_event_writer: EventWriter<TableLoadingEvent>,
    mut globals: ResMut<Globals>,
    mut vpx_tables: ResMut<VpxTables>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut next_state: ResMut<NextState<LoadingState>>,
) {
    for event in reader.read() {
        match &event.0 {
            ChannelExternalEvent::VpxDone => {
                info!("Event: {:?}", event);
                globals.vpinball_running = false;
                resume_music(&mut music_event_writer);
                let mut window = window_query.single_mut();
                // info!("Window visibility: {}", window.visible);
                // info!("Showing window");
                // window.visible = true;
                // bring window to front
                // window.window_level = WindowLevel::AlwaysOnTop;
                // request focus
                window.focused = true;
            }
            ChannelExternalEvent::TablesLoaded(tables) => {
                // TODO this clone is rather heavy, how to avoid?
                vpx_tables.indexed_tables = tables.clone();
                vpx_tables
                    .indexed_tables
                    .sort_by_key(|indexed| display_table_line(indexed).to_lowercase());
                mark_tables_loaded(&mut next_state);
            }
            ChannelExternalEvent::ProgressLength(length) => {
                table_loading_event_writer.send(TableLoadingEvent::Length(*length));
            }
            ChannelExternalEvent::ProgressPosition(position) => {
                table_loading_event_writer.send(TableLoadingEvent::Position(*position));
            }
            ChannelExternalEvent::ProgressFinishAndClear => {
                table_loading_event_writer.send(TableLoadingEvent::FinishAndClear);
            }
        }
    }
}

pub fn guifrontend(config: ResolvedConfig) {
    let tables: Vec<IndexedTable> = Vec::new();
    let vpinball_ini_path = config.vpinball_ini_file();
    let vpinball_config = VPinballConfig::read(&vpinball_ini_path).unwrap();
    let playfield_window = windowing::setup_playfield_window(&vpinball_config);

    let mut app = App::new();
    app.insert_resource(Config { config })
        .insert_resource(VpxConfig {
            config: vpinball_config,
        })
        .insert_resource(VpxTables {
            indexed_tables: tables,
        })
        .insert_resource(Globals {
            vpinball_running: false,
        })
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(playfield_window),
            ..Default::default()
        }))
        .add_plugins(WindowingPlugin)
        .add_plugins(crate::event_channel::plugin)
        .add_plugins(music_plugin)
        .add_plugins((wheel_plugin, flipper_plugin, dmd_plugin, list_plugin))
        .add_plugins(loading_plugin)
        .add_plugins(crate::gradient_background::plugin)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup_gradient_background)
        .add_systems(Update, quit_on_q_or_window_closed)
        .add_systems(Update, handle_external_events)
        .add_systems(Update, launcher.run_if(in_state(LoadingState::Ready)))
        .add_systems(Update, dmd_update.run_if(in_state(LoadingState::Ready)))
        .add_systems(Update, show_info.run_if(in_state(LoadingState::Ready)))
        .init_state::<LoadingState>();

    // only for development
    #[cfg(debug_assertions)]
    {
        app.add_plugins(bevy_mini_fps::fps_plugin!());
        app.add_plugins(crate::debug_window_labels::plugin)
            //app.add_plugins(bevy_dev_tools::ui_debug_overlay::DebugUiPlugin)
            .add_systems(Update, toggle_overlay);
        app.add_plugins(bevy_dev_tools::fps_overlay::FpsOverlayPlugin {
            config: bevy_dev_tools::fps_overlay::FpsOverlayConfig {
                text_config: TextFont {
                    font: Default::default(),
                    font_size: 12.0,
                    ..Default::default()
                },
                text_color: Color::WHITE,
                enabled: false,
            },
        })
        .add_systems(Update, toggle_fps);
    }

    app.run();
}

#[cfg(debug_assertions)]
// The system that will enable/disable the debug outlines around the nodes
fn toggle_overlay(
    input: Res<ButtonInput<KeyCode>>,
    //mut options: ResMut<bevy_dev_tools::ui_debug_overlay::UiDebugOptions>,
    mut window_name_options: ResMut<crate::debug_window_labels::WindowNameOptions>,
) {
    info_once!("The debug outlines are enabled, press Space to turn them on/off");
    if input.just_pressed(KeyCode::KeyD) {
        // The toggle method will enable the debug_overlay if disabled and disable if enabled
        //options.toggle();
        window_name_options.enabled = !window_name_options.enabled;
    }
}

#[cfg(debug_assertions)]
// The system that will enable/disable the debug outlines around the nodes
fn toggle_fps(
    input: Res<ButtonInput<KeyCode>>,
    mut options: ResMut<bevy_dev_tools::fps_overlay::FpsOverlayConfig>,
) {
    info_once!("The debug outlines are enabled, press Space to turn them on/off");
    if input.just_pressed(KeyCode::KeyF) {
        // The toggle method will enable the debug_overlay if disabled and disable if enabled
        options.enabled = !options.enabled;
    }
}
