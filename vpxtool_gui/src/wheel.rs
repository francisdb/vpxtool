use crate::guifrontend::{Config, VpxTables};
use crate::list::SelectedItem;
use crate::loading::{LoadingData, LoadingState};
use bevy::ecs::system::SystemId;
use bevy::image::Image;
use bevy::log::{debug, info};
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_asset::{AssetId, AssetServer};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Resource, Default)]
pub struct AssetPaths {
    pub paths: HashMap<AssetId<Image>, String>,
}

#[derive(Resource, Default)]
pub(crate) struct WheelInfo {
    pub wheel_size: f32,
}

#[derive(Component)]
pub struct Wheel {
    pub item_number: usize,
}

#[derive(Resource)]
pub(crate) struct LoadWheelsSystem(pub(crate) SystemId);

#[derive(Bundle)]
struct WheelBundle {
    sprite: Sprite,
    transform: Transform,
    //global_transform: GlobalTransform,
    visibility: Visibility,
    wheel: Wheel,
    //inherited_visibility: InheritedVisibility,
    //view_visibility: ViewVisibility,
}

pub(crate) fn wheel_plugin(app: &mut App) {
    app.insert_resource(AssetPaths::default());
    app.insert_resource(WheelInfo::default());
    //app.add_systems(Startup, create_wheels);
    let load_system = app.register_system(create_wheels);
    app.insert_resource(LoadWheelsSystem(load_system));
    app.add_systems(
        Update,
        update_selected_wheel.run_if(in_state(LoadingState::Ready)),
    );
}

pub const BOTTOM_MARGIN: f32 = 40.;

fn update_selected_wheel(
    mut wheel_query: Query<(&mut Visibility, &mut Sprite, &Wheel, &mut Transform)>,
    mut wheel_info: ResMut<WheelInfo>,
    selected_item_res: Res<SelectedItem>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // TODO only update if window size changes?
    if let Ok(window) = window_query.get_single() {
        let window_height = window.height();
        wheel_info.wheel_size = derive_wheel_size(window);
        let wheel_size = wheel_info.wheel_size;
        let selected_item = selected_item_res.index.unwrap_or(0);
        // update currently selected item to new value
        for (mut visibility, mut sprite, wheel, mut transform) in wheel_query.iter_mut() {
            if wheel.item_number != selected_item {
                *visibility = Visibility::Hidden;
            } else {
                sprite.custom_size = Some(Vec2::new(wheel_size, wheel_size));
                *visibility = Visibility::Visible;
                transform.translation = Vec3::new(
                    0.,
                    (-(window_height / 2.0)) + (wheel_size / 2.) + BOTTOM_MARGIN,
                    0.,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_wheels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_data: ResMut<LoadingData>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut game_state: ResMut<NextState<LoadingState>>,
    config: Res<Config>,
    vpx_tables: Res<VpxTables>,
    mut asset_paths: ResMut<AssetPaths>,
    mut wheel_info: ResMut<WheelInfo>,
    mut wheel_query: Query<Entity, With<Wheel>>,
) {
    info!("Creating wheels...");

    // remove any existing wheels
    for entity in wheel_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }

    let tables = &vpx_tables.indexed_tables;

    let window = window_query.single();
    wheel_info.wheel_size = derive_wheel_size(window);
    let wheel_size = wheel_info.wheel_size;

    // Create blank wheel
    // tries [table_path]/wheels/blankwheel.png first
    // fallbacks to assets/blankwheel.png
    let mut blank_path = config.config.tables_folder.clone();
    blank_path.push("/wheels/blankwheel.png");
    if !Path::new(&blank_path).exists() {
        // will be loaded from assets
        blank_path = PathBuf::from("blankwheel.png");
    }

    for (table_index, info) in tables.iter().enumerate() {
        let wheel_path = match &info.wheel_path {
            // get handle from path
            Some(path) => path.clone(),
            None => blank_path.clone(),
        };
        let wheel_image_handle = asset_server.load(wheel_path.clone());
        loading_data
            .loading_assets
            .push(wheel_image_handle.clone().into());

        let transform = Transform::from_xyz(0., 0., 0.);

        debug!(
            "Wheel asset for table {} = {} {}",
            info.path.display(),
            wheel_path.display(),
            wheel_image_handle.id(),
        );

        let table_name = match &info.table_info.table_name {
            Some(name) => name,
            None => "None",
        };
        asset_paths
            .paths
            .insert(wheel_image_handle.id(), table_name.to_owned());

        commands.spawn(WheelBundle {
            sprite: Sprite {
                image: wheel_image_handle.clone(),
                custom_size: Some(Vec2::new(wheel_size, wheel_size)),
                ..default()
            },
            transform,
            visibility: Visibility::Hidden,
            wheel: Wheel {
                item_number: table_index,
            },
        });
    }
    info!("Wheels assets loading...");
    game_state.set(LoadingState::LoadingImages);
}

fn derive_wheel_size(window: &Window) -> f32 {
    window.height() / 3.
}
