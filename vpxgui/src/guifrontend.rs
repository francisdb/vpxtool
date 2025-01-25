use crate::dmd::dmd_plugin;
use crate::event_channel::{ChannelExternalEvent, ExternalEvent, StreamSender};
use crate::flippers::flipper_plugin;
use crate::info::show_info;
use crate::list::{display_table_line, list_plugin, SelectedItem};
use crate::loading::LoadingState;
use crate::loading::{loading_plugin, mark_tables_loaded, LoadingData};
use crate::menus::*;
use crate::music::{music_plugin, resume_music, suspend_music, ControlMusicEvent};
use crate::pipelines::PipelinesReadyPlugin;
use crate::process::do_launch;
use crate::wheel::{wheel_plugin, LoadWheelsSystem};
use crate::windowing;
use crate::windowing::WindowingPlugin;
use bevy::prelude::*;
use bevy::window::*;
use bevy_egui::EguiPlugin;
use bevy_mini_fps::fps_plugin;
use shared::config::{ResolvedConfig, VPinballConfig};
use shared::indexer::IndexedTable;

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
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    tables: Res<VpxTables>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        if globals.vpinball_running {
            warn!("Visual Pinball already running");
            return;
        };

        if let Some(selected_item) = selected_item.index {
            let mut window = window_query.single_mut();
            suspend_music(&mut control_music_event_writer);
            let table = tables.indexed_tables.get(selected_item).unwrap();
            info!("Hide window");
            window.visible = false;
            globals.vpinball_running = true;
            do_launch(
                stream_sender.clone(),
                &table.path,
                &config.config.vpx_executable,
            );
        }
    }
}

fn quit_on_q(
    keys: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        app_exit_events.send(bevy::app::AppExit::Success);
    }
}

fn gui_update(_time: Res<Time>, window_query: Query<&Window, With<PrimaryWindow>>) {
    let mut window = window_query.get_single().unwrap().clone();
    window.window_level = WindowLevel::Normal;
}

#[allow(clippy::too_many_arguments)]
fn handle_external_events(
    mut reader: EventReader<ExternalEvent>,
    mut event_writer: EventWriter<ControlMusicEvent>,
    mut globals: ResMut<Globals>,
    mut vpx_tables: ResMut<VpxTables>,
    mut loading_data: ResMut<LoadingData>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
    load_wheels_system: Res<LoadWheelsSystem>,
) {
    for event in reader.read() {
        match &event.0 {
            ChannelExternalEvent::VpxDone => {
                info!("Event: {:?}", event);
                globals.vpinball_running = false;
                resume_music(&mut event_writer);
                let mut window = window_query.single_mut();
                info!("Window visibility: {}", window.visible);
                info!("Showing window");
                window.visible = true;
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
                mark_tables_loaded(&mut loading_data, &mut commands, &load_wheels_system);
            }
        }
    }
}

pub fn guifrontend(config: ResolvedConfig) {
    let tables: Vec<IndexedTable> = Vec::new();
    let vpinball_ini_path = config.vpinball_ini_file();
    let vpinball_config = VPinballConfig::read(&vpinball_ini_path).unwrap();
    let window = windowing::setup_playfield_window(&vpinball_config);

    App::new()
        .insert_resource(Config { config })
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
            primary_window: Some(window),
            ..Default::default()
        }))
        .add_plugins(WindowingPlugin)
        .add_plugins(fps_plugin!())
        .add_plugins(crate::event_channel::plugin)
        .add_plugins(music_plugin)
        .add_plugins((wheel_plugin, flipper_plugin, dmd_plugin, list_plugin))
        .add_plugins(loading_plugin)
        .add_plugins(crate::gradient_background::plugin)
        .add_plugins(EguiPlugin)
        .add_plugins(PipelinesReadyPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, quit_on_q)
        .add_systems(Update, handle_external_events)
        .add_systems(Update, gui_update.run_if(in_state(LoadingState::Ready)))
        .add_systems(Update, launcher.run_if(in_state(LoadingState::Ready)))
        .add_systems(Update, dmd_update.run_if(in_state(LoadingState::Ready)))
        .add_systems(Update, show_info.run_if(in_state(LoadingState::Ready)))
        .init_state::<LoadingState>()
        .run();

    // TODO do we want to create sets of systems that are run_if(in_state(LoadingState::Ready))?
    //   does that work with plugins?
    // https://bevy-cheatbook.github.io/programming/run-conditions.html
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
