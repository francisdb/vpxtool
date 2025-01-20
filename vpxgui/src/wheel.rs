use crate::guifrontend::{
    AssetPaths, Config, Globals, TableText, TextItemGhostWhite, TextItemGold, VpxTables,
};
use crate::loading::{LoadingData, LoadingState};
use bevy::color::palettes::css::GHOST_WHITE;
use bevy::log::{debug, info, warn};
use bevy::math::Vec3;
use bevy::prelude::{
    default, AlignContent, Bundle, Commands, Component, Display, FlexDirection, NextState, Node,
    PositionType, Query, Res, ResMut, Sprite, Text, TextColor, TextFont, Transform, Val,
    Visibility, Window, With,
};
use bevy::window::PrimaryWindow;
use bevy_asset::AssetServer;
use image::ImageReader;
use std::path::{Path, PathBuf};

// #[derive(AssetCollection, Resource)]
// struct ImageAssets {
//     #[asset(key = "wheel")]
//     _wheel: Handle<Image>,
// }

#[derive(Component)]
pub struct Wheel {
    pub item_number: i16,
    //pub image_handle: Handle<Image>,
    pub selected: bool,
    pub launch_path: PathBuf,
    //pub table_info: IndexedTable,
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

#[derive(Bundle)]
struct MenuTextBundle {
    text: Text,
    text_font: TextFont,
    text_color: TextColor,
    text_bundle: Node,
    // display: Display,
    //position_type: PositionType,
    //  left: f32,
    //   top: f32,
    //   right: f32,
    table_text: TableText,
    text_item: TextItemGold,
}

#[derive(Bundle)]
struct MenuTextBundle1 {
    text: Text,
    text_font: TextFont,
    text_color: TextColor,
    text_bundle: Node,
    // display: Display,
    //position_type: PositionType,
    //  left: f32,
    //   top: f32,
    //   right: f32,
    table_text: TableText,
    text_item: TextItemGhostWhite,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_wheel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_data: ResMut<LoadingData>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut game_state: ResMut<NextState<LoadingState>>,
    // assets: Res<Assets<Image>>,
    config: Res<Config>,
    vpx_tables: Res<VpxTables>,
    mut asset_paths: ResMut<AssetPaths>,
    mut globals: ResMut<Globals>,
) {
    let _list_of_tables = &vpx_tables.indexed_tables;
    let tables = &vpx_tables.indexed_tables;

    let window = window_query.single();
    // Set default wheel size to a third of the window height
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

    for (counter, info) in tables.iter().enumerate() {
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

        let temporary_table_name = match &info.table_info.table_name {
            Some(tb) => tb,
            None => "None",
        };
        asset_paths
            .paths
            .insert(wheel_image_handle.id(), temporary_table_name.to_owned());

        // Wheel
        commands.spawn(WheelBundle {
            /*
                        Replace all uses of SpriteBundle with Sprite. There are several new convenience constructors: Sprite::from_image, Sprite::from_atlas_image, Sprite::from_color.

            WARNING: use of Handle<Image> and TextureAtlas as components on sprite entities will NO LONGER WORK. Use the fields on Sprite instead. I would have removed the Component impls from TextureAtlas and Handle<Image> except it is still used within ui. We should fix this moving forward with the migration.
                         */
            sprite: Sprite {
                // texture: asset_server.load("/usr/tables/wheels/Sing Along (Gottlieb 1967).png"),
                image: wheel_image_handle.clone(),
                ..default()
            },
            transform,
            visibility: Visibility::Hidden,
            wheel: Wheel {
                item_number: counter as i16,
                //image_handle: handle.clone(),
                selected: false,
                launch_path: info.path.clone(),
                //tableinfo: info.clone(),
            },
        });

        // Game Name

        // TODO move this MenuTextBundle(1) to it's own module?
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        commands.spawn(MenuTextBundle {
            // Create a TextBundle that has a Text with a single section.
            text: Text::new(temporary_table_name),
            text_font: TextFont {
                // This font is loaded and will be used instead of the default font.
                font_size: 20.0,
                // TextStyle has been renamed to TextFont and its color field has been moved to a separate component named TextColor which newtypes Color.
                ..default()
            },
            text_color: TextColor::from(GHOST_WHITE),
            text_bundle: Node {
                // Set the justification of the Text
                //.with_text_justify(JustifyText::Center)
                // Set the style of the TextBundle itself.
                display: Display::None,
                position_type: PositionType::Absolute,
                left: Val::Px(20.),
                //top: Val::Px(245.),
                top: Val::Px(window.height() * 0.025), //-(height-(height/2.+(scale*2.)))),
                right: Val::Px(0.),
                ..default()
            },
            table_text: TableText {
                item_number: counter as i16,
                table_text: match info.table_info.table_description.clone() {
                    Some(a) => a,
                    _ => "Empty".to_owned(),
                },
                table_blurb: match info.table_info.table_blurb.clone() {
                    Some(a) => a,
                    _ => "Empty".to_owned(),
                }, //has_wheel: haswheel,
            },
            text_item: TextItemGold {
                //item_number: counter as i16,
                //image_handle: handle.clone(),
                //selected: false,
            },
        });

        // game info text
        commands.spawn(MenuTextBundle1 {
            // Create a TextBundle that has a Text with a single section.
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            text: Text::new(temporary_table_name),
            text_font: TextFont {
                // This font is loaded and will be used instead of the default font.
                font_size: 20.0,
                ..default()
            },
            text_color: TextColor::from(GHOST_WHITE),
            // Set the justification of the Text
            //.with_text_justify(JustifyText::Center)
            // Set the style of the TextBundle itself.
            text_bundle: Node {
                flex_direction: FlexDirection::Row,
                align_content: AlignContent::FlexEnd,
                display: Display::None,
                position_type: PositionType::Absolute,
                left: Val::Px(20.),
                top: Val::Px(window.height() * 0.2), //-(height-(height/2.+(scale*2.)))),
                right: Val::Px(0.),
                ..default()
            },

            table_text: TableText {
                item_number: counter as i16,
                table_text: match info.table_info.table_description.clone() {
                    Some(a) => a,
                    _ => "Empty".to_owned(),
                },
                table_blurb: match info.table_info.table_blurb.clone() {
                    Some(a) => a,
                    _ => "Empty".to_owned(),
                },
            },
            text_item: TextItemGhostWhite {
                //item_number: counter as i16,
                //image_handle: handle.clone(),
                //selected: false,
            },
        });

        //let image = image::load(BufReader::new(File::open("foo.png")?), ImageFormat::Jpeg)?;
        //entities +=1.;
    }
    //let update = commands.register_one_shot_system(update_loading_data);
    //commands.run_system(update);
    info!("Wheels loaded");

    game_state.set(LoadingState::Loading);
}
