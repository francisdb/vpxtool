use crate::guifrontend::{Config, Globals, VpxTables};
use crate::list::SelectedItem;
use crate::loading::{LoadingData, LoadingState};
use bevy::image::Image;
use bevy::log::{debug, info, warn};
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_asset::{AssetId, AssetServer};
use image::ImageReader;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Resource, Default)]
pub struct AssetPaths {
    pub paths: HashMap<AssetId<Image>, String>,
}

#[derive(Component)]
pub struct Wheel {
    pub item_number: usize,
}

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
    app.insert_resource(AssetPaths {
        paths: HashMap::new(),
    });
    app.add_systems(Startup, create_wheels);
    app.add_systems(Update, update_selected_wheel);
}

fn update_selected_wheel(
    mut wheel_query: Query<(&mut Visibility, &Wheel, &mut Transform)>,
    mut globals: ResMut<Globals>,
    selected_item_res: Res<SelectedItem>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let selected_item = selected_item_res.index.unwrap_or(0);
    let window = window_query.single();
    let height = window.height();
    globals.wheel_size = window.height() / 3.;
    let wheel_size = globals.wheel_size;
    // update currently selected item to new value
    for (mut visibility, wheel, mut transform) in wheel_query.iter_mut() {
        if wheel.item_number != selected_item {
            *visibility = Visibility::Hidden;
            // transform.translation = Vec3::new(0., width, 0.);
        } else {
            *visibility = Visibility::Visible;
            // *transform = Transform::from_xyz(0., 0., 0.);
            transform.translation = Vec3::new(0., (-(height / 2.0)) + (wheel_size / 2.) + 20., 0.);
            //transform.translation = Vec3::new(0., -(height - (height / 2.75 + (scale * 2.))), 0.);
            //    println!("Selected {}",&wheel.launchpath.as_os_str().to_string_lossy());
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn create_wheels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_data: ResMut<LoadingData>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut game_state: ResMut<NextState<LoadingState>>,
    config: Res<Config>,
    vpx_tables: Res<VpxTables>,
    mut asset_paths: ResMut<AssetPaths>,
    mut globals: ResMut<Globals>,
) {
    let tables = &vpx_tables.indexed_tables;

    let window = window_query.single();
    // Set default wheel size to a third of the window height
    // TODO move this from globals to a specific resource for this module
    globals.wheel_size = window.height() / 3.;

    // let mut orentation = Horizontal;
    // if height > width {
    //     orentation = Vertical;
    // } else {
    //     orentation = Horizontal;
    // };

    //let mut scale = width/10.;
    //let mut entities=0.;
    //let locations = [
    //    -(width/2.)+scale,
    //    -(scale*2.),
    //    0.,
    //    (scale*2.),
    //    (width/2.) - (scale),
    // ];
    //let mut handles =[];

    let mut transform = Transform::from_xyz(0., 0., 0.);

    //let mut transform = Transform::from_xyz(0., -(height-(height/2.+(scale*2.))), 0.);
    //let mut transform = Transform::from_xyz(locations[xlocation], -(height-(height/2.+(scale*2.))), 0.);

    // Create blank wheel
    // tries [table_path]/wheels/blankwheel.png first
    // fallbacks to assets/blankwheel.png
    let mut blank_path = config.config.tables_folder.clone();
    blank_path.push("/wheels/blankwheel.png");
    if !Path::new(&blank_path).exists() {
        // will be loaded from assets
        warn!("Please copy the blankwheel.png to {:?}", blank_path);
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
        // Normalizing the dimensions of wheels so they are all the same size.
        //  using imagesize crate as it is a very fast way to get the dimensions.
        let (_wheel_width, _wheel_height) = (0., 0.);

        // TODO below code is blocking, should be offloaded to a thread?
        let wheel_height = if wheel_path.exists() {
            let image = ImageReader::open(&wheel_path)
                .unwrap()
                .into_dimensions()
                .unwrap();
            let (_wheel_width, wheel_height) = image;
            wheel_height
        } else {
            1000
        };
        // wheel_size.wheel_size = (height / 3.) / (size.height as f32);
        // Normalize icons to 1/3 the screen height
        transform.scale = Vec3::new(
            (window.height() / 5.) / (wheel_height as f32),
            (window.height() / 5.) / (wheel_height as f32),
            100.0,
        );

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

        // Wheel
        commands.spawn(WheelBundle {
            sprite: Sprite {
                image: wheel_image_handle.clone(),
                ..default()
            },
            transform,
            visibility: Visibility::Hidden,
            wheel: Wheel {
                item_number: table_index,
            },
        });
    }
    info!("Wheels loaded");

    game_state.set(LoadingState::Loading);
}
