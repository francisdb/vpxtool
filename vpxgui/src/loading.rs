use crate::event_channel::{ChannelExternalEvent, StreamSender};
use crate::guifrontend::Config;
use crate::pipelines::{PipelinesReady, PipelinesReadyPlugin};
use crate::wheel::{AssetPaths, LoadWheelsSystem};
use bevy::image::Image;
use bevy::prelude::*;
use bevy_asset::{AssetServer, RecursiveDependencyLoadState, UntypedHandle};
use crossbeam_channel::Sender;
use shared::indexer;
use shared::indexer::Progress;
use std::thread;

const SLOW_LOADING: bool = false;

/// Marker tag for top-level loading screen components.
#[derive(Component)]
struct PartOfLoadingScreen;

#[derive(Component)]
struct LoadingScreenTitle;

#[derive(Component)]
struct LoadingScreenText;

#[derive(Resource, Debug)]
struct LoadingDialogBox {
    pub title: String,
    pub text: String,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum LoadingState {
    #[default]
    Initializing,
    LoadingTables,
    LoadingImages,
    Ready,
}

#[derive(Event)]
pub(crate) enum TableLoadingEvent {
    Length(u64),
    Position(u64),
    FinishAndClear,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct LoadingData {
    // This will hold the currently unloaded/loading assets.
    pub(crate) loading_assets: Vec<UntypedHandle>,
    // Number of frames that everything needs to be ready for.
    // This is to prevent going into the fully loaded state in instances
    // where there might be a some frames between certain loading/pipelines action.
    confirmation_frames_target: usize,
    // Current number of confirmation frames.
    confirmation_frames_count: usize,
    num_tables: u64,
    loaded_tables: u64,
}

impl LoadingData {
    pub(crate) fn new(confirmation_frames_target: usize) -> Self {
        Self {
            loading_assets: Vec::new(),
            confirmation_frames_target,
            confirmation_frames_count: 0,
            num_tables: 0,
            loaded_tables: 0,
        }
    }
}

pub(crate) fn loading_plugin(app: &mut App) {
    app.add_plugins(PipelinesReadyPlugin);
    app.add_event::<TableLoadingEvent>();
    app.insert_resource(LoadingData::new(5));
    app.insert_resource(LoadingDialogBox {
        title: "Loading".to_owned(),
        text: "".to_owned(),
    });
    app.add_systems(Startup, load_tables);
    app.add_systems(
        Update,
        (update_loading_data, update_loading_screen).chain().run_if(
            in_state(LoadingState::LoadingTables).or(in_state(LoadingState::LoadingImages)),
        ),
    );
    app.add_systems(OnEnter(LoadingState::LoadingTables), setup_loading_screen);
    app.add_systems(OnEnter(LoadingState::LoadingImages), load_images);
    app.add_systems(OnExit(LoadingState::LoadingImages), despawn_loading_screen);
}

// Monitors current loading status of assets.
#[allow(clippy::too_many_arguments)]
fn update_loading_data(
    mut dialog: ResMut<LoadingDialogBox>,
    mut loading_data: ResMut<LoadingData>,
    mut next_state: ResMut<NextState<LoadingState>>,
    current_state: ResMut<State<LoadingState>>,
    asset_server: Res<AssetServer>,
    pipelines_ready: Res<PipelinesReady>,
    asset_paths: Res<AssetPaths>,
    mut table_loading_event_reader: EventReader<TableLoadingEvent>,
) {
    for event in table_loading_event_reader.read() {
        match event {
            TableLoadingEvent::Length(length) => {
                loading_data.num_tables = *length;
            }
            TableLoadingEvent::Position(position) => {
                loading_data.loaded_tables = *position;
            }
            TableLoadingEvent::FinishAndClear => {
                // we might want to do something here
            }
        }
    }

    match current_state.get() {
        LoadingState::Initializing => {
            // should never happen
        }
        LoadingState::LoadingTables => {
            dialog.title = "Loading tables".to_owned();
            if loading_data.num_tables != 0 {
                dialog.text = format!("{}/{}", loading_data.loaded_tables, loading_data.num_tables);
            } else {
                dialog.text = "scanning tables folder...".to_owned();
            }
        }
        LoadingState::LoadingImages => {
            dialog.title = "Loading images".to_string();
            if !loading_data.loading_assets.is_empty() || !pipelines_ready.0 {
                // If we are still loading assets / pipelines are not fully compiled,
                // we reset the confirmation frame count.
                loading_data.confirmation_frames_count = 0;

                // Go through each asset and verify their load states.
                // Any assets that are loaded are then added to the pop list for later removal.
                let mut pop_list: Vec<usize> = Vec::new();
                for (index, asset) in loading_data.loading_assets.iter().enumerate() {
                    // log asset name
                    // info!("asset {:?}", asset);
                    if let Some((_, _, RecursiveDependencyLoadState::Loaded)) =
                        asset_server.get_load_states(asset)
                    {
                        let id = asset.id().typed_unchecked::<Image>();
                        // Since for example the default asset is shared this will repeatedly the last
                        // path that was loaded.
                        // info!("loading {}", asset_paths.paths.get(&id).cloned().unwrap());
                        dialog.text = format!(
                            "{} {}",
                            loading_data.loading_assets.len(),
                            asset_paths.paths.get(&id).cloned().unwrap()
                        );
                        pop_list.push(index);
                    }
                }

                // Remove all loaded assets from the loading_assets list.
                if !pop_list.is_empty() {
                    debug!("Removing {} loaded assets.", pop_list.len());
                    // remove all items from the pop list
                    if SLOW_LOADING {
                        for index in pop_list.iter().rev().take(1) {
                            loading_data.loading_assets.remove(*index);
                        }
                    } else {
                        for index in pop_list.iter().rev() {
                            loading_data.loading_assets.remove(*index);
                        }
                    }
                }

                // If there are no more assets being monitored, and pipelines
                // are compiled, then start counting confirmation frames.
                // Once enough confirmations have passed, everything will be
                // considered to be fully loaded.
            } else {
                loading_data.confirmation_frames_count += 1;
                if loading_data.confirmation_frames_count == loading_data.confirmation_frames_target
                {
                    info!("All assets loaded.");
                    mark_images_loaded(&mut next_state);
                }
            }
        }
        LoadingState::Ready => {
            // should never happen
        }
    }
}

fn mark_images_loaded(next_state: &mut ResMut<NextState<LoadingState>>) {
    next_state.set(LoadingState::Ready);
}

pub(crate) fn mark_tables_loaded(next_state: &mut NextState<LoadingState>) {
    next_state.set(LoadingState::LoadingImages);
    // TODO request all images to be loaded here?
}

fn load_images(mut commands: Commands, load_wheels_system: Res<LoadWheelsSystem>) {
    commands.run_system(load_wheels_system.0);
}

fn setup_loading_screen(mut commands: Commands, dialog: ResMut<LoadingDialogBox>) {
    let title = &dialog.title;
    let text = &dialog.text;

    // outer node with 2 cells below each other
    commands
        .spawn((
            // center the loading screen
            Node {
                display: Display::Flex,
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            PartOfLoadingScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    // flex box for title and text
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    // title
                    parent.spawn((
                        Text::new(title),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor::from(Color::WHITE),
                        Node {
                            display: Display::Flex,
                            ..default()
                        },
                        LoadingScreenTitle,
                    ));
                    // text
                    parent.spawn((
                        Text::new(text),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor::from(Color::WHITE),
                        Node {
                            display: Display::Flex,
                            ..default()
                        },
                        LoadingScreenText,
                    ));
                });
        });
}

#[allow(clippy::type_complexity)]
fn update_loading_screen(
    dialog: ResMut<LoadingDialogBox>,
    mut set: ParamSet<(
        Query<&mut Text, With<LoadingScreenTitle>>,
        Query<&mut Text, With<LoadingScreenText>>,
    )>,
) {
    for mut text in &mut set.p0() {
        **text = dialog.title.clone();
    }
    for mut text in &mut set.p1() {
        **text = dialog.text.clone();
    }
}

fn despawn_loading_screen(mut commands: Commands, query: Query<Entity, With<PartOfLoadingScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // remove resources
    commands.remove_resource::<LoadingDialogBox>();
    commands.remove_resource::<LoadingData>();
}

struct EventSendingProgress {
    sender: Sender<ChannelExternalEvent>,
}
impl Progress for EventSendingProgress {
    fn set_length(&self, len: u64) {
        self.sender
            .send(ChannelExternalEvent::ProgressLength(len))
            .unwrap();
    }

    fn set_position(&self, i: u64) {
        self.sender
            .send(ChannelExternalEvent::ProgressPosition(i))
            .unwrap();
    }

    fn finish_and_clear(&self) {
        self.sender
            .send(ChannelExternalEvent::ProgressFinishAndClear)
            .unwrap();
    }
}

fn load_tables(
    resolved_config: Res<Config>,
    stream_sender: Res<StreamSender>,
    mut next_state: ResMut<NextState<LoadingState>>,
) {
    mark_ready_for_loading_tables(&mut next_state);

    // perform below loading in a separate thread, then report back to bevy
    let tx = stream_sender.clone();
    let resolved_config = resolved_config.config.clone();
    let _vpinball_thread = thread::spawn(move || {
        let recursive = true;
        // TODO make a progress that sends events and update loading gui
        let progress = EventSendingProgress { sender: tx.clone() };
        let index_result = indexer::index_folder(
            recursive,
            &resolved_config.tables_folder,
            &resolved_config.tables_index_path,
            Some(&resolved_config.global_pinmame_rom_folder()),
            &progress,
            Vec::new(),
        );
        progress.finish_and_clear();
        match index_result {
            Ok(index) => {
                info!("{} tables loaded", index.len());
                tx.send(ChannelExternalEvent::TablesLoaded(index.tables()))
                    .unwrap();
            }
            Err(e) => {
                error!("Error loading tables: {:?}", e);
                tx.send(ChannelExternalEvent::TablesLoaded(Vec::new()))
                    .unwrap();
            }
        }
    });
}

fn mark_ready_for_loading_tables(next_state: &mut ResMut<NextState<LoadingState>>) {
    next_state.set(LoadingState::LoadingTables);
}
