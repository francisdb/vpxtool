use crate::guifrontend::{AssetPaths, DialogBox};
use crate::pipelines::PipelinesReady;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_asset::{AssetServer, RecursiveDependencyLoadState, UntypedHandle};
use bevy_egui::{egui, EguiContexts};

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum LoadingState {
    #[default]
    Initializing,
    Loading,
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
}

impl LoadingData {
    pub(crate) fn new(confirmation_frames_target: usize) -> Self {
        Self {
            loading_assets: Vec::new(),
            confirmation_frames_target,
            confirmation_frames_count: 0,
        }
    }
}

// TODO create a plugin that also pulls in the pipelines_ready plugin

// Monitors current loading status of assets.
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_loading_data(
    _commands: Commands,
    mut dialog: ResMut<DialogBox>,
    mut loading_data: ResMut<LoadingData>,
    mut game_state: ResMut<NextState<LoadingState>>,
    // mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    pipelines_ready: Res<PipelinesReady>,
    asset_paths: Res<AssetPaths>,
) {
    dialog.title = format!("Loading {}...", loading_data.loading_assets.len());
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
                dialog.text = asset_paths.paths.get(&id).cloned().unwrap();
                pop_list.push(index);
            }
        }

        // Remove all loaded assets from the loading_assets list.
        if !pop_list.is_empty() {
            info!("Removing {} loaded assets.", pop_list.len());
            // remove all items from the pop list
            for index in pop_list.iter().rev() {
                loading_data.loading_assets.remove(*index);
            }
            // loading_data.loading_assets.remove(pop_list[0]);
        }

        // If there are no more assets being monitored, and pipelines
        // are compiled, then start counting confirmation frames.
        // Once enough confirmations have passed, everything will be
        // considered to be fully loaded.
    } else {
        loading_data.confirmation_frames_count += 1;
        if loading_data.confirmation_frames_count == loading_data.confirmation_frames_target {
            game_state.set(LoadingState::Ready);
        }
    }
}

// Marker tag for loading screen components.
//#[derive(Component)]
//struct LoadingScreen;

// Spawns the necessary components for the loading screen.
pub(crate) fn load_loading_screen(
    _commands: Commands,
    dialog: ResMut<DialogBox>,
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let _text_style = TextFont {
        font_size: 80.0,
        ..default()
    };
    let window = window_query.single();

    let title = &dialog.title;
    let text = &dialog.text;

    let width = window.resolution.width();
    let height = window.resolution.height();
    let ctx = contexts.ctx_mut();
    let _raw_input = egui::RawInput::default();
    //let x = TextColor::from(GHOST_WHITE);

    // Check if the texture is loaded if let Some(texture) = textures.get(texture_handle) { // Display the image using egui egui::Window::new("Image Window").show(egui_context.ctx_mut(), |ui| { let texture_id = egui::TextureId::User(texture_handle.id); ui.image(texture_id, [texture.size.width as f32, texture.size.height as f32
    //let texture_handle: Handle<Texture> = asset_server.load("//usr/tables/wheels/blankwheel.png");
    let _x: Handle<Image> = asset_server.load("left-flipper.png");

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
        .current_pos(egui::Pos2::new((width / 3.0) - 10.0, height / 3.0))
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
    if loading_state.get() == &LoadingState::Loading {
        //      *loading_screen.get_single_mut().unwrap() = Visibility::Hidden;
        //     *loading_screen.get_single_mut().unwrap() = Visibility::Hidden;
    };
}
