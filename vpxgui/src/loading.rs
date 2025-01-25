use crate::event_channel::{ChannelExternalEvent, StreamSender};
use crate::guifrontend::Config;
use crate::pipelines::{PipelinesReady, PipelinesReadyPlugin};
use crate::wheel::{AssetPaths, LoadWheelsSystem};
use bevy::image::Image;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_asset::{AssetServer, RecursiveDependencyLoadState, UntypedHandle};
use bevy_egui::{egui, EguiContexts};
use shared::indexer;
use shared::indexer::VoidProgress;
use std::thread;

const SLOW_LOADING: bool = false;

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
    tables_loaded: bool,
}

impl LoadingData {
    pub(crate) fn new(confirmation_frames_target: usize) -> Self {
        Self {
            loading_assets: Vec::new(),
            confirmation_frames_target,
            confirmation_frames_count: 0,
            tables_loaded: false,
        }
    }
}

// TODO implement proper state handling
// eg
// app.add_systems(OnEnter(MyAppState::MainMenu), (
//     setup_main_menu_ui,
//     setup_main_menu_camera,
// ));
// app.add_systems(OnExit(MyAppState::MainMenu), (
//     despawn_main_menu,
// ));
// https://bevy-cheatbook.github.io/programming/states.html

// TODO create a plugin that also pulls in the pipelines_ready plugin
pub(crate) fn loading_plugin(app: &mut App) {
    app.add_plugins(PipelinesReadyPlugin);
    app.insert_resource(LoadingData::new(5));
    app.insert_resource(LoadingDialogBox {
        title: "Loading...".to_owned(),
        text: "blank".to_owned(),
    });
    app.add_systems(Startup, load_tables);
    app.add_systems(Update, display_loading_screen);
    app.add_systems(
        Update,
        load_loading_screen.run_if(
            in_state(LoadingState::LoadingTables).or(in_state(LoadingState::LoadingImages)),
        ),
    );
    app.add_systems(
        Update,
        update_loading_data.run_if(
            in_state(LoadingState::LoadingTables).or(in_state(LoadingState::LoadingImages)),
        ),
    );
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
) {
    match current_state.get() {
        LoadingState::Initializing => {
            // should never happen
        }
        LoadingState::LoadingTables => {
            dialog.title = "Loading tables...".to_owned();
            dialog.text = "Please wait...".to_owned();
        }
        LoadingState::LoadingImages => {
            dialog.title = format!("Loading images...");
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
                    next_state.set(LoadingState::Ready);
                }
            }
        }
        LoadingState::Ready => {
            // should never happen
        }
    }
}

pub(crate) fn mark_tables_loaded(
    loading_data: &mut ResMut<LoadingData>,
    commands: &mut Commands,
    load_wheels_system: &Res<LoadWheelsSystem>,
) {
    loading_data.tables_loaded = true;
    commands.run_system(load_wheels_system.0);
}

// Marker tag for loading screen components.
//#[derive(Component)]
//struct LoadingScreen;

// Spawns the necessary components for the loading screen.
fn load_loading_screen(
    _commands: Commands,
    dialog: ResMut<LoadingDialogBox>,
    mut contexts: EguiContexts,
    //asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // let _text_style = TextFont {
    //     font_size: 80.0,
    //     ..default()
    // };
    let window = window_query.single();

    let title = &dialog.title;
    let text = &dialog.text;

    let width = window.resolution.width();
    let height = window.resolution.height();
    let ctx = contexts.ctx_mut();
    //let _raw_input = egui::RawInput::default();
    //let x = TextColor::from(GHOST_WHITE);

    // Check if the texture is loaded if let Some(texture) = textures.get(texture_handle) { // Display the image using egui egui::Window::new("Image Window").show(egui_context.ctx_mut(), |ui| { let texture_id = egui::TextureId::User(texture_handle.id); ui.image(texture_id, [texture.size.width as f32, texture.size.height as f32
    //let texture_handle: Handle<Texture> = asset_server.load("//usr/tables/wheels/blankwheel.png");
    // let _x: Handle<Image> = asset_server.load("left-flipper.png");

    /*   commands.spawn(FlipperBundle1 {
            sprite: Sprite {
                image: asset_server.load("right-flipper.png"),
                ..default()
            },
            visibility: Visibility::Visible,

            transform: Transform {
                translation: Vec3::new(
                    100.0, 100.0,
                    //  window_width - (window_width * 0.60),
                    // window_height * 0.25 + 60.,
                    0.,
                ),
                scale: (Vec3::new(0.5, 0.5, 1.0)),
                rotation: Quat::from_rotation_z(0.25),
                ..default()
            },
            flipper1: Flipper1,
        });
    */

    egui::Area::new(egui::Id::new("my area"))
        .current_pos(egui::Pos2::new((width / 5.0) - 10.0, height / 3.0))
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(50.0)
                    .color(egui::Color32::WHITE),
            );
            ui.label(
                egui::RichText::new(text)
                    .size(20.0)
                    .color(egui::Color32::WHITE),
            );
            //ui.image(("file://assets/left-flipper.png"));

            //   if ui.button("Click me").clicked() {
            // take some action here
            //   }
        });
}

// Determines when to show the loading screen
pub(crate) fn display_loading_screen(
    // mut loading_screen: Query<&mut Visibility, With<LoadingScreen>>,
    loading_state: ResMut<State<LoadingState>>,
    //  loading_state: Res<LoadingState>,
) {
    //println!("loading state {:?}", loading_state.get());
    if loading_state.get() == &LoadingState::LoadingImages {
        //      *loading_screen.get_single_mut().unwrap() = Visibility::Hidden;
        //     *loading_screen.get_single_mut().unwrap() = Visibility::Hidden;
    };
}

fn load_tables(
    resolved_config: Res<Config>,
    stream_sender: Res<StreamSender>,
    mut next_state: ResMut<NextState<LoadingState>>,
) {
    next_state.set(LoadingState::LoadingTables);

    // perform below loading in a separate thread, then report back to bevy
    let tx = stream_sender.clone();
    let resolved_config = resolved_config.config.clone();
    let _vpinball_thread = thread::spawn(move || {
        let recursive = true;
        // TODO make a progress that sends events and update loading gui
        let progress = VoidProgress;
        let index_result = indexer::index_folder(
            recursive,
            &resolved_config.tables_folder,
            &resolved_config.tables_index_path,
            Some(&resolved_config.global_pinmame_rom_folder()),
            &progress,
            Vec::new(),
        );
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
