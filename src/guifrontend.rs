mod menus;
use bevy::color::palettes::css::*;
use bevy::core_pipeline::{
    bloom::{BloomCompositeMode, BloomSettings},
    tonemapping::Tonemapping,
};
use bevy::ecs::system::SystemId;
use bevy::render::view::visibility;
use bevy::sprite::{MaterialMesh2dBundle, Wireframe2dConfig, Wireframe2dPlugin};
//use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle, Wireframe2dConfig, Wireframe2dPlugin};
use bevy::{input::common_conditions::*, prelude::*};
use bevy_asset::*;
use bevy_asset_loader::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use image::ImageReader;
use indicatif::{ProgressBar, ProgressStyle};
use menus::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{
    fs::File,
    io,
    io::Write,
    path::{Path, PathBuf},
    process::{exit, ExitStatus},
};

use crate::config;
use crate::config::{ResolvedConfig, VPinballConfig};
use crate::indexer::{find_vpx_files, IndexError, IndexedTable, Progress};
use crate::patcher::LineEndingsResult::{NoChanges, Unified};
use crate::patcher::{patch_vbs_file, unify_line_endings_vbs_file};
use crate::{
    indexer, info_diff, info_edit, info_gather, open_editor, run_diff, script_diff,
    vpx::{extractvbs, vbs_path_for, ExtractResult},
    DiffColor, ProgressBarProgress,
};
use bevy::utils::info;
use bevy::window::*;
use colored::Colorize;
use console::Emoji;
use is_executable::IsExecutable;
use pipelines_ready::*;
use std::ffi::{OsStr, OsString};

const LAUNCH: Emoji = Emoji("🚀", "[launch]");
const CRASH: Emoji = Emoji("💥", "[crash]");

const SEARCH: &str = "> Search";
const RECENT: &str = "> Recent";
const SEARCH_INDEX: usize = 0;
const RECENT_INDEX: usize = 1;
const HORIZONTAL: bool = false;
const VERTICAL: bool = true;

#[derive(Component)]
pub struct Wheel {
    pub item_number: i16,
    //pub image_handle: Handle<Image>,
    pub selected: bool,
    pub launch_path: PathBuf,
    //pub table_info: IndexedTable,
}

#[derive(Component)]
pub struct TextItemGold {
    pub item_number: i16,
    //pub image_handle: Handle<Image>,
    pub selected: bool,
    //  pub launch_path: PathBuf,
    //pub table_info: IndexedTable,
}

#[derive(Component)]
pub struct TextItemGhostWhite {
    pub item_number: i16,
    //pub image_handle: Handle<Image>,
    pub selected: bool,
    //  pub launch_path: PathBuf,
    //pub table_info: IndexedTable,
}

#[derive(Component, Debug)]
pub struct TableText {
    pub item_number: i16,
    pub tabletext: String,
    pub tableblurb: String,
    //pub has_wheel: bool,
}

#[derive(Component, Debug)]
pub struct TableBlurb {
    pub item_number: i16,
}

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

#[derive(Component, Debug)]
pub struct InfoBox {
    // infostring: String,
}

#[derive(Resource, Debug)]
pub struct Globals {
    pub wheel_size: f32,
    pub game_running: bool,
}

#[derive(Resource, Debug)]
pub struct DialogBox {
    pub title: String,
    pub text: String,
}

fn correct_window_size_and_position(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    vpx_config: Res<VpxConfig>,
) {
    // only on Linux
    // #[cfg(target_os = "linux")] is annoying because it causes clippy to complain about dead code
    if cfg!(target_os = "linux") {
        // Under wayland the window size is not correct, we need to scale it down.
        // In vpinball the playfield window size is configured in physical pixels.
        // The window constructor will create a window with the size in logical pixels.
        let mut window = window_query.single_mut();
        if window.resolution.scale_factor() != 1.0 {
            info!(
                "Resizing window for Linux with scale factor {}",
                window.resolution.scale_factor(),
            );
            let vpinball_config = &vpx_config.config;
            if let Some(playfield) = vpinball_config.get_playfield_info() {
                if let (Some(physical_width), Some(physical_height)) =
                    (playfield.width, playfield.height)
                {
                    let logical_width = physical_width as f32 / window.resolution.scale_factor();
                    let logical_height = physical_height as f32 / window.resolution.scale_factor();
                    info!(
                        "Setting window size to {}x{}",
                        logical_width, logical_height
                    );
                    window.resolution.set(logical_width, logical_height);
                    window.set_changed();
                }
            }
        }
    }

    // only on macOS
    // #[cfg(target_os = "macos")] is annoying because it causes clippy to complain about dead code
    if cfg!(target_os = "macos") {
        let mut window = window_query.single_mut();
        if window.resolution.scale_factor() != 1.0 {
            info!(
                "Repositioning window for macOS with scale factor {}",
                window.resolution.scale_factor(),
            );
            let vpinball_config = &vpx_config.config;
            if let Some(playfield) = vpinball_config.get_playfield_info() {
                if let (Some(logical_x), Some(logical_y)) = (playfield.x, playfield.y) {
                    // For macOS with scales factor > 1 this is not correct but we don't know the scale
                    // factor before the window is created.
                    let physical_x = logical_x as f32 * window.resolution.scale_factor();
                    let physical_y = logical_y as f32 * window.resolution.scale_factor();
                    info!("Setting window position to {}, {}", physical_x, physical_y,);
                    // this will apply the width as if it was set in logical pixels
                    window.position =
                        WindowPosition::At(IVec2::new(physical_x as i32, physical_y as i32));
                    window.set_changed();
                }
            }
        }
    }
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
#[derive(Component)]
pub struct Flipper;

#[derive(Component)]
pub struct Flipper1;

#[derive(Bundle)]
struct FlipperBundle {
    sprite: Sprite,
    transform: Transform,
    // translate: Translate,
    //global_transform: GlobalTransform,
    //    visibility: Visibility,
    //    wheel: Wheel,
    //inherited_visibility: InheritedVisibility,
    visibility: Visibility,
    flipper: Flipper,
}

#[derive(Bundle)]
struct FlipperBundle1 {
    sprite: Sprite,
    transform: Transform,
    // translate: Translate,
    //global_transform: GlobalTransform,
    //    visibility: Visibility,
    //    wheel: Wheel,
    //inherited_visibility: InheritedVisibility,
    visibility: Visibility,
    flipper1: Flipper1,
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

fn create_wheel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_data: ResMut<LoadingData>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut game_state: ResMut<NextState<LoadingState>>,
    // assets: Res<Assets<Image>>,
    config: Res<Config>,
    vpx_tables: Res<VpxTables>,
    mut asset_paths: ResMut<AssetPaths>,
) {
    let level_data = LevelData {
        level_1_id: commands.register_system(gui_update),
    };
    commands.insert_resource(level_data);

    //config: &ResolvedConfig,
    //vpx_files_with_tableinfo: &mut Vec<IndexedTable>,
    //vpinball_executable: &Path,
    //info: &IndexedTable,
    //info_str: &str,

    //// commands.spawn(SpriteBundle {
    ////     texture: asset_server.load("/usr/tables/wheels/Sing Along (Gottlieb 1967).png"),
    ////    ..default()
    //// });
    ///
    //let (_config_path, loaded_config) = config::load_config().unwrap().unwrap();
    let vpx_files_with_tableinfo1 = frontend_index(&config.config, true, vec![]).unwrap();
    let roms = indexer::find_roms(&config.config.global_pinmame_rom_folder());
    let roms1 = roms.unwrap();
    let tables: Vec<String> = frontend_index(&config.config, true, vec![])
        .unwrap()
        .iter()
        .map(|indexed| display_table_line_full(indexed, &roms1))
        .collect();
    //let temporary_path_name="";

    let window = window_query.single();
    let width = window.width();
    let height = window.height();
    let table_path = &config.config.tables_folder;

    // let mut orentation = Horizontal;
    // if height > width {
    //     orentation = Vertical;
    // } else {
    //     orentation = Horizontal;
    // };

    //let mut scale = width/10.;
    let tables_len = tables.len();
    //let mut entities=0.;
    let mut counter: usize = 0;
    let mut xlocation = 0;
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
    let mut blank_path = table_path.clone().into_os_string();
    blank_path.push("/wheels/blankwheel.png");
    if !Path::new(&blank_path).exists() {
        // will be loaded from assets
        println!("Please copy the blankwheel.png to {:?}", blank_path);
        blank_path = PathBuf::from("blankwheel.png").into_os_string();
    }

    while counter < (tables_len) {
        if xlocation > 4 {
            xlocation = 0
        };
        let info = vpx_files_with_tableinfo1.get(counter).unwrap().clone();
        /*    match &info.wheel_path {
                  Some(path)=> println!("{}",&path.as_os_str().to_string_lossy()),
                  None => println!("NONE"),
              };
        */
        //let mut haswheel = true;
        //let mut temporary_path_name= &info.wheel_path.unwrap();
        //blank_path.into();

        let temporary_path_name = match &info.wheel_path {
            // get handle from path
            Some(path) => {
                //haswheel = false;
                PathBuf::from(path)
            }
            None => {
                //haswheel = true;
                PathBuf::from(blank_path.clone())
            }
        };
        // let mut temporary_table_name="None";
        //let mut handle =  asset_server.load(temporary_path_name);
        let temporary_table_name = match &info.table_info.table_name {
            Some(tb) => &tb,
            None => "None",
        };

        // let table_info = match &info.table_info.table_rules {
        //    Some(tb) => &tb,
        //    None => "None",
        //    };
        let handle = asset_server.load(temporary_path_name.clone());
        loading_data.loading_assets.push(handle.clone().into());
        // Normalizing the dimentions of wheels so they are all the same size.
        //  using imagesize crate as it is a very fast way to get the dimentions.
        let (mut wheel_width, mut wheel_height) = (0., 0.);

        // Set default wheel size
        commands.insert_resource(Globals {
            wheel_size: (height / 3.),
            game_running: false,
        });

        match imagesize::size(&temporary_path_name) {
            Ok(size) => {
                wheel_width = size.width as f32;
                wheel_height = size.height as f32;

                // wheel_size.wheel_size = (height / 3.) / (size.height as f32);
                // Normalize icons to 1/3 the screen height
                transform.scale = Vec3::new(
                    (height / 5.) / (size.height as f32),
                    (height / 5.) / (size.height as f32),
                    100.0,
                );

                println!(
                    "Initializing:  {}",
                    &temporary_path_name.as_os_str().to_string_lossy()
                );
            }
            Err(why) => println!(
                "Error getting dimensions: {} {:?}",
                &temporary_path_name.as_os_str().to_string_lossy(),
                why
            ),
        };

        asset_paths
            .paths
            .insert(handle.clone().id(), temporary_table_name.clone().to_owned());

        // Wheel
        commands.spawn(WheelBundle {
            /*
                        Replace all uses of SpriteBundle with Sprite. There are several new convenience constructors: Sprite::from_image, Sprite::from_atlas_image, Sprite::from_color.

            WARNING: use of Handle<Image> and TextureAtlas as components on sprite entities will NO LONGER WORK. Use the fields on Sprite instead. I would have removed the Component impls from TextureAtlas and Handle<Image> except it is still used within ui. We should fix this moving forward with the migration.
                         */
            sprite: Sprite {
                // texture: asset_server.load("/usr/tables/wheels/Sing Along (Gottlieb 1967).png"),
                image: handle.clone(),
                ..default()
            },
            transform: transform,
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
                top: Val::Px(height * 0.025), //-(height-(height/2.+(scale*2.)))),
                right: Val::Px((0.)),
                ..default()
            },
            table_text: TableText {
                item_number: counter as i16,
                tabletext: match info.table_info.table_description.clone() {
                    Some(a) => a,
                    _ => "Empty".to_owned(),
                },
                tableblurb: match info.table_info.table_blurb.clone() {
                    Some(a) => a,
                    _ => "Empty".to_owned(),
                }, //has_wheel: haswheel,
            },
            text_item: TextItemGold {
                item_number: counter as i16,
                //image_handle: handle.clone(),
                selected: false,
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
                top: Val::Px(height * 0.2), //-(height-(height/2.+(scale*2.)))),
                right: Val::Px((0.)),
                ..default()
            },

            table_text: TableText {
                item_number: counter as i16,
                tabletext: match info.table_info.table_description.clone() {
                    Some(a) => a,
                    _ => "Empty".to_owned(),
                },
                tableblurb: match info.table_info.table_blurb.clone() {
                    Some(a) => a,
                    _ => "Empty".to_owned(),
                },
            },
            text_item: TextItemGhostWhite {
                item_number: counter as i16,
                //image_handle: handle.clone(),
                selected: false,
            },
        });

        //let image = image::load(BufReader::new(File::open("foo.png")?), ImageFormat::Jpeg)?;
        counter += 1;
        xlocation += 1;
        //entities +=1.;
    }
    // commands.spawn((Camera2dBundle
    //                {..default()},));
    //let update = commands.register_one_shot_system(update_loading_data);
    //commands.run_system(update);
    println!("Wheels loaded");

    game_state.set(LoadingState::LevelLoading);
}

fn create_flippers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    let window_width = window.width();
    let window_height = window.height();
    commands.spawn(FlipperBundle {
        sprite: Sprite {
            // texture: asset_server.load("/usr/tables/wheels/Sing Along (Gottlieb 1967).png"),
            image: asset_server.load("left-flipper.png"),
            ..default()
        },
        visibility: Visibility::Hidden,

        transform: Transform {
            translation: Vec3::new(
                window_width - (window_width * 0.60) - 225.,
                (window_height * 0.25) + 60.,
                0.,
            ),
            scale: (Vec3::new(0.5, 0.5, 1.0)),
            rotation: Quat::from_rotation_z(-0.25),
            ..default()
        },
        flipper: Flipper,
    });

    commands.spawn(FlipperBundle1 {
        sprite: Sprite {
            image: asset_server.load("right-flipper.png"),
            ..default()
        },
        visibility: Visibility::Hidden,

        transform: Transform {
            translation: Vec3::new(
                window_width - (window_width * 0.60),
                window_height * 0.25 + 60.,
                0.,
            ),
            scale: (Vec3::new(0.5, 0.5, 1.0)),
            rotation: Quat::from_rotation_z(0.25),
            ..default()
        },
        flipper1: Flipper1,
    });
}

enum TableOption {
    Launch,
    LaunchFullscreen,
    LaunchWindowed,
    ForceReload,
    InfoShow,
    InfoEdit,
    InfoDiff,
    ExtractVBS,
    EditVBS,
    PatchVBS,
    UnifyLineEndings,
    ShowVBSDiff,
    CreateVBSPatch,
    // ClearNVRAM,
}

impl TableOption {
    const ALL: [TableOption; 13] = [
        TableOption::Launch,
        TableOption::LaunchFullscreen,
        TableOption::LaunchWindowed,
        TableOption::ForceReload,
        TableOption::InfoShow,
        TableOption::InfoEdit,
        TableOption::InfoDiff,
        TableOption::ExtractVBS,
        TableOption::EditVBS,
        TableOption::PatchVBS,
        TableOption::UnifyLineEndings,
        TableOption::ShowVBSDiff,
        TableOption::CreateVBSPatch,
        // TableOption::ClearNVRAM,
    ];

    fn from_index(index: usize) -> Option<TableOption> {
        match index {
            0 => Some(TableOption::Launch),
            1 => Some(TableOption::LaunchFullscreen),
            2 => Some(TableOption::LaunchWindowed),
            3 => Some(TableOption::ForceReload),
            4 => Some(TableOption::InfoShow),
            5 => Some(TableOption::InfoEdit),
            6 => Some(TableOption::InfoDiff),
            7 => Some(TableOption::ExtractVBS),
            8 => Some(TableOption::EditVBS),
            9 => Some(TableOption::PatchVBS),
            10 => Some(TableOption::UnifyLineEndings),
            11 => Some(TableOption::ShowVBSDiff),
            12 => Some(TableOption::CreateVBSPatch),
            // 13 => Some(TableOption::ClearNVRAM),
            _ => None,
        }
    }

    fn display(&self) -> String {
        match self {
            TableOption::Launch => "Launch".to_string(),
            TableOption::LaunchFullscreen => "Launch fullscreen".to_string(),
            TableOption::LaunchWindowed => "Launch windowed".to_string(),
            TableOption::ForceReload => "Force reload".to_string(),
            TableOption::InfoShow => "Info > Show".to_string(),
            TableOption::InfoEdit => "Info > Edit".to_string(),
            TableOption::InfoDiff => "Info > Diff".to_string(),
            TableOption::ExtractVBS => "VBScript > Extract".to_string(),
            TableOption::EditVBS => "VBScript > Edit".to_string(),
            TableOption::PatchVBS => "VBScript > Patch typical standalone issues".to_string(),
            TableOption::UnifyLineEndings => "VBScript > Unify line endings".to_string(),
            TableOption::ShowVBSDiff => "VBScript > Diff".to_string(),
            TableOption::CreateVBSPatch => "VBScript > Create patch file".to_string(),
            // TableOption::ClearNVRAM => "Clear NVRAM".to_string(),
        }
    }
}

pub fn frontend_index(
    resolved_config: &ResolvedConfig,
    recursive: bool,
    force_reindex: Vec<PathBuf>,
) -> Result<Vec<IndexedTable>, IndexError> {
    let pb = ProgressBar::hidden();
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{bar:.cyan/blue}] {pos}/{human_len} ({eta})",
        )
        .unwrap(),
    );
    let progress = ProgressBarProgress::new(pb);
    let index = indexer::index_folder(
        recursive,
        &resolved_config.tables_folder,
        &resolved_config.tables_index_path,
        &progress,
        force_reindex,
    );
    progress.finish_and_clear();
    let index = index?;

    let mut tables: Vec<IndexedTable> = index.tables();
    tables.sort_by_key(|indexed| display_table_line(indexed).to_lowercase());
    Ok(tables)
}

pub fn gui_update(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut dialog: ResMut<DialogBox>,

    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut set: ParamSet<(
        Query<
            (
                &mut TableText,
                &mut TextFont,
                &mut Node,
                &mut TextColor,
                &mut Text,
            ),
            With<TextItemGold>,
        >,
        Query<(&mut TableBlurb, &mut Node), With<TextItemGhostWhite>>,
    )>,
    mut query: ParamSet<(
        Query<(&mut Visibility, &mut Wheel, &mut Transform), With<Wheel>>,
        Query<(&mut Transform, &mut Visibility), With<Flipper>>,
        Query<(&mut Transform, &mut Visibility), With<Flipper1>>,
    )>,
    music_box_query: Query<&AudioSink>,
    mut contexts: EguiContexts,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut globals: ResMut<Globals>,
) {
    let (_config_path, loaded_config) = config::load_config().unwrap().unwrap();
    let mut window = window_query.get_single().unwrap().clone();
    window.window_level = WindowLevel::Normal;
    let mut wtitle = " ".to_owned();
    let mut gametext = " ".to_owned();
    let mut gameblurb = " ".to_owned();

    let width = window.width();
    let height = window.height();

    //let mut orentation = HORIZONTAL;
    // if height > width {orentation=VERTICAL;}
    //    else {orentation=HORIZONTAL};

    let mut scale = width / 10.;

    // arbitrary number to indicate there is no selected item.
    let mut selected_item: i16 = -2;

    // set a flag indicating if we are ready to launch a game
    let mut launchit = false;

    // Count entities
    let mut num = 1;
    num += query.p0().iter().count() as i16;

    // Find current selection
    for (visibility, wheel, transform) in query.p0().iter() {
        if wheel.selected {
            selected_item = wheel.item_number;
        }
    }
    // If no selection, set it to item 0
    if selected_item == -2 {
        for (visibility, mut wheel, transform) in query.p0().iter_mut() {
            if wheel.item_number == 0 {
                wheel.selected = true;
                selected_item = 0;
            }
        }
    };

    if let Ok(sink) = music_box_query.get_single() {
        if keys.just_pressed(KeyCode::Equal) {
            sink.set_volume(sink.volume() + 0.1);
        } else if keys.just_pressed(KeyCode::Minus) {
            sink.set_volume(sink.volume() - 0.1);
        } else if keys.just_pressed(KeyCode::KeyM) {
            if sink.is_paused() {
                sink.play()
            } else {
                sink.pause()
            }
            //   } else if keys.just_pressed(KeyCode::KeyN) {
            //       sink.play();
        } else if keys.just_pressed(KeyCode::Digit1) {
            if globals.game_running {
                globals.game_running = false;
            } else {
                globals.game_running = true;
            }
        } else if keys.just_pressed(KeyCode::ShiftRight) {
            selected_item += 1;
        } else if keys.just_pressed(KeyCode::ShiftLeft) {
            selected_item -= 1;
        } else if keys.just_pressed(KeyCode::Enter) {
            launchit = true;
        } else if keys.just_pressed(KeyCode::KeyQ) {
            app_exit_events.send(bevy::app::AppExit::Success);
        } else if keys.just_pressed(KeyCode::Space) {
            println!("current table {}", selected_item);
        }

        // Wrap around if one of the bounds are hit.
        if selected_item == num - 1 {
            selected_item = 0;
        } else if selected_item == -1 {
            selected_item = num - 2;
        }

        // for (mut visibility, mut wheel, mut transform) in query.iter_mut() {}

        // update currently selected item to new value
        for (mut visibility, mut wheel, mut transform) in query.p0().iter_mut() {
            if wheel.item_number != selected_item {
                wheel.selected = false;
                *visibility = Visibility::Hidden;
            //                    transform.translation = Vec3::new(0., width, 0.);
            } else {
                wheel.selected = true;
                *visibility = Visibility::Visible;
                // *transform = Transform::from_xyz(0., 0., 0.);
                let wsize = globals.wheel_size;
                transform.translation = Vec3::new(0., (-(height / 2.0)) + (wsize / 2.) + 20., 0.);
                //transform.translation = Vec3::new(0., -(height - (height / 2.75 + (scale * 2.))), 0.);
                //    println!("Selected {}",&wheel.launchpath.as_os_str().to_string_lossy());
            }
        }

        for (mut transform, mut visibility) in query.p1().iter_mut() {
            let wsize = globals.wheel_size;

            transform.translation = Vec3::new(
                ((wsize / 3.0) * -1.0),
                ((-(height / 2.)) + (wsize / 4.)),
                0.,
            );
            *visibility = Visibility::Visible;
        }

        for (mut transform, mut visibility) in query.p2().iter_mut() {
            let wsize = globals.wheel_size;

            transform.translation =
                Vec3::new((wsize / 3.0), ((-(height / 2.0)) + (wsize / 4.)), 0.);
            *visibility = Visibility::Visible;
        }

        // change name of game
        for (mut items, mut font, mut textstyle, mut color, text) in set.p0().iter_mut() {
            if items.item_number != selected_item {
                textstyle.display = Display::None;
                *color = TextColor::from(GHOST_WHITE);
            } else {
                *color = TextColor::from(GHOST_WHITE);
                font.font_size = 20.0;
                gametext = items.tabletext.clone();
                gameblurb = items.tableblurb.clone();
                textstyle.display = Display::Block;
                wtitle = text.to_string();
            }
        }

        // table scroll
        let mut counter = 11;
        let mut names = [0; 21];

        // item # less than 10
        for count in 2..=11 {
            if num + (selected_item - counter) < num - 1 {
                names[count - 2] = num + (selected_item - counter);
            } else if selected_item - counter > num {
                names[count - 2] = num - (selected_item - counter)
            } else {
                names[count - 2] = (selected_item + 1) - counter;
            };
            counter -= 1;
            // item number over num-10
            // item number not over 10 or less than num-10
        }
        names[10] = selected_item;

        counter = 0;
        for count in 12..=22 {
            if (selected_item + counter) < num - 1 {
                names[count - 2] = (selected_item + counter);
            } else if selected_item + counter + 3 > num {
                names[count - 2] = (selected_item + counter - num) + 1
            }
            //        else  {names[count-2] = (selected_item+1)-counter;};
            counter += 1;
        }
        counter = 0;

        //   let mut wtitle = &gametext;
        let mut wtext = &gameblurb;

        // clear all game name assets
        for (items, mut fontsize, mut textstyle, mut color, _text) in set.p0().iter_mut() {
            if num > 21 {
                textstyle.display = Display::None;
                fontsize.font_size = 20.0;
                *color = TextColor::from(GHOST_WHITE);
            } else {
                textstyle.display = Display::Block;
                fontsize.font_size = 20.0;
                *color = TextColor::from(GHOST_WHITE);

                textstyle.top = Val::Px(255. + (((counter as f32) + 1.) * 20.));
                counter += 1;
            }
        }

        if num > 21 {
            for _name in names {
                for (items, mut fontsize, mut text_style, mut color, text) in set.p0().iter_mut() {
                    for (index, item) in names.iter().enumerate().take(9 + 1) {
                        if items.item_number == *item {
                            //wtitle = items;
                            *color = TextColor::from(GHOST_WHITE);
                            text_style.top = Val::Px(25. + (((index as f32) + 1.) * 20.));
                            fontsize.font_size = 15.0;
                            text_style.display = Display::Block;
                            //        if items.itemnumber == selected_item {textstyle.color:GOLD.into(); }
                        }
                    }
                    for (index, item) in names.iter().enumerate().skip(10) {
                        if items.item_number == *item {
                            fontsize.font_size = 25.0;
                            *color = TextColor::from(GOLD);
                            text_style.top = Val::Px(255. + (((index as f32) - 10.5) * 20.));
                            text_style.display = Display::Block;
                            break;
                        }
                    }

                    for (index, item) in names.iter().enumerate().skip(11) {
                        if items.item_number == *item {
                            *color = TextColor::from(GHOST_WHITE);
                            fontsize.font_size = 15.0;
                            text_style.top = Val::Px(255. + (((index as f32) - 10.) * 20.));
                            text_style.display = Display::Block;
                            //        if items.itemnumber == selected_item {textstyle.color:GOLD.into(); }
                        }
                    }
                }
            }
        }
        //  counter += 1;

        if globals.game_running {
            create_info_box(
                commands,
                keys,
                meshes,
                materials,
                &window.clone(),
                contexts,
                wtitle,
                gametext.to_owned(),
            );
        };

        if launchit {
            //if globals.game_running {
            //    println!("Game running");
            //    return;
            //};
            let mut game_running = globals.game_running;
            //   globals.game_running = true;
            let mut ispaused: bool = false;
            if let Ok(sink) = music_box_query.get_single() {
                ispaused = sink.is_paused();
                sink.pause();
            };
            for (visibility, wheel, transform) in query.p0().iter() {
                if wheel.item_number == selected_item {
                    println!(
                        "Launching {}",
                        wheel.launch_path.clone().into_os_string().to_string_lossy()
                    );
                    println!("Hide window");
                    window.visible = false;

                    let (tx, rx) = mpsc::channel();
                    let tx = tx.clone();
                    let path = wheel.launch_path.clone();
                    let mut global = globals.game_running.clone();
                    let (_config_path, loaded_config) = config::load_config().unwrap().unwrap();
                    let executable = loaded_config.vpx_executable; // .executable.clone();

                    let pin_thread = std::thread::spawn(move || {
                        launch(&path, &executable, None);
                        thread::sleep(Duration::from_millis(2 as u64));

                        println!("Vpinball done, sending event");
                        match tx.send(1) {
                            Ok(_tx1) => tx.send(1).unwrap(),
                            _ => (),
                        };

                        window.visible = true;
                        true
                    });
                    //  if pin_thread.join().unwrap() == true {
                    //      globals.game_running = false;
                    //  }
                    //  println!("game_running {}", globals.game_running);
                }
            }
            if let Ok(sink) = music_box_query.get_single() {
                if !ispaused {
                    sink.play();
                }
            };
        }
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum LoadingState {
    #[default]
    LevelIntitializing,
    LevelLoading,
    LevelReady,
    LevelMenu,
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(key = "wheel")]
    wheel: Handle<Image>,
}

#[derive(Resource, Debug, Default)]
struct LoadingData {
    // This will hold the currently unloaded/loading assets.
    loading_assets: Vec<UntypedHandle>,
    // Number of frames that everything needs to be ready for.
    // This is to prevent going into the fully loaded state in instances
    // where there might be a some frames between certain loading/pipelines action.
    confirmation_frames_target: usize,
    // Current number of confirmation frames.
    confirmation_frames_count: usize,
}

impl LoadingData {
    fn new(confirmation_frames_target: usize) -> Self {
        Self {
            loading_assets: Vec::new(),
            confirmation_frames_target,
            confirmation_frames_count: 0,
        }
    }
}

// This resource will hold the level related systems ID for later use.
#[derive(Resource)]
struct LevelData {
    level_1_id: SystemId,
}

#[derive(Resource, Default)]
pub struct AssetPaths {
    pub paths: HashMap<AssetId<Image>, String>,
}

#[derive(Resource)]
pub struct AssetPath {
    pub handle: Handle<Image>,
    pub path: OsString,
}

// Marker component for easier deletion of entities.
#[derive(Component)]
struct LevelComponents;

// Removes all currently loaded level assets from the game World.
fn unload_current_level(
    mut commands: Commands,
    // mut loading_state: ResMut<LoadingState>,
    entities: Query<Entity, With<LevelComponents>>,
) {
    // *loading_state = LoadingState::LevelLoading;
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// Monitors current loading status of assets.
fn update_loading_data(
    mut commands: Commands,
    mut dialog: ResMut<DialogBox>,
    mut loading_data: ResMut<LoadingData>,
    mut game_state: ResMut<NextState<LoadingState>>,
    // mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    pipelines_ready: Res<PipelinesReady>,
    mut level_data: Res<LevelData>,
    asset_paths: Res<AssetPaths>,
) {
    dialog.title = "Loading...".to_owned();
    //dialog.text = "test".to_owned();
    if !loading_data.loading_assets.is_empty() || !pipelines_ready.0 {
        // If we are still loading assets / pipelines are not fully compiled,
        // we reset the confirmation frame count.
        loading_data.confirmation_frames_count = 0;

        // Go through each asset and verify their load states.
        // Any assets that are loaded are then added to the pop list for later removal.
        let mut pop_list: Vec<usize> = Vec::new();
        for (index, asset) in loading_data.loading_assets.iter().enumerate() {
            if let Some(state) = asset_server.get_load_states(asset) {
                if let bevy::asset::RecursiveDependencyLoadState::Loaded = state.2 {
                    let id = asset.id().typed_unchecked::<Image>();
                    dialog.text = asset_paths.paths.get(&id).cloned().unwrap();
                    pop_list.push(index);
                }
            }
        }

        // Remove all loaded assets from the loading_assets list.
        if !pop_list.is_empty() {
            println!("pop list {:?}", pop_list[0]);
            loading_data.loading_assets.remove(pop_list[0]);
        }

        // If there are no more assets being monitored, and pipelines
        // are compiled, then start counting confirmation frames.
        // Once enough confirmations have passed, everything will be
        // considered to be fully loaded.
    } else {
        loading_data.confirmation_frames_count += 1;
        if loading_data.confirmation_frames_count == loading_data.confirmation_frames_target {
            game_state.set(LoadingState::LevelReady);
        }
    }
}

// Marker tag for loading screen components.
#[derive(Component)]
struct LoadingScreen;

// Spawns the necessary components for the loading screen.
fn load_loading_screen(
    mut commands: Commands,
    mut dialog: ResMut<DialogBox>,
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let text_style = TextFont {
        font_size: 80.0,
        ..default()
    };
    let window = window_query.single();

    let title = &dialog.title;
    let text = &dialog.text;

    let width = window.resolution.width();
    let height = window.resolution.height();
    let mut ctx = contexts.ctx_mut();
    let raw_input = egui::RawInput::default();
    //let x = TextColor::from(GHOST_WHITE);

    // Check if the texture is loaded if let Some(texture) = textures.get(texture_handle) { // Display the image using egui egui::Window::new("Image Window").show(egui_context.ctx_mut(), |ui| { let texture_id = egui::TextureId::User(texture_handle.id); ui.image(texture_id, [texture.size.width as f32, texture.size.height as f32
    //let texture_handle: Handle<Texture> = asset_server.load("//usr/tables/wheels/blankwheel.png");
    let x: Handle<Image> = asset_server.load("left-flipper.png");

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
        .current_pos(egui::Pos2::new((width / 3.0) - 10.0, (height / 3.0)))
        .show(&ctx, |ui| {
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
fn display_loading_screen(
    // mut loading_screen: Query<&mut Visibility, With<LoadingScreen>>,
    mut loading_state: ResMut<State<LoadingState>>,
    //  loading_state: Res<LoadingState>,
) {
    //println!("loading state {:?}", loading_state.get());
    match loading_state.get() {
        LoadingState::LevelLoading => {
            //      *loading_screen.get_single_mut().unwrap() = Visibility::Hidden;
            //     *loading_screen.get_single_mut().unwrap() = Visibility::Hidden;
        }
        //LoadingState::LevelReady => *loading_screen.get_single_mut().unwrap() = Visibility::Hidden,
        _ => {}
    };
}

mod pipelines_ready {
    use bevy::{prelude::*, render::render_resource::*, render::*};

    pub struct PipelinesReadyPlugin;
    impl Plugin for PipelinesReadyPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(PipelinesReady::default());

            // In order to gain access to the pipelines status, we have to
            // go into the `RenderApp`, grab the resource from the main App
            // and then update the pipelines status from there.
            // Writing between these Apps can only be done through the
            // `ExtractSchedule`.
            app.sub_app_mut(bevy::render::RenderApp)
                .add_systems(ExtractSchedule, update_pipelines_ready);
        }
    }

    #[derive(Resource, Debug, Default)]
    pub struct PipelinesReady(pub bool);

    fn update_pipelines_ready(mut main_world: ResMut<MainWorld>, pipelines: Res<PipelineCache>) {
        if let Some(mut pipelines_ready) = main_world.get_resource_mut::<PipelinesReady>() {
            pipelines_ready.0 = pipelines.waiting_pipelines().count() == 0;
        }
    }
}

fn level_selection(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    level_data: Res<LevelData>,
    // loading_state: Res<LoadingState>,
) {
    // Only trigger a load if the current level is fully loaded.
    /*    if let LoadingState::LevelReady = loading_state.as_ref() {
        commands.run_system(level_data.level_1_id);
    }
    */
}

pub fn guifrontend(
    config: ResolvedConfig,
    vpx_files_with_tableinfo: Vec<IndexedTable>,
    //roms: &HashSet<String>,
    //vpinball_executable: &Path,
) {
    // let tables: Vec<String> = vpx_files_with_tableinfo
    //     .iter()
    //     .map(|indexed| display_table_line_full(indexed, roms))
    //     .collect();
    // let path = "/usr/tables/wheels/Sing Along (Gottlieb 1967).png";

    //    let options = eframe::NativeOptions {
    //       viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 800.0]),
    //       ..Default::default()
    //   };

    let vpinball_ini_path = config.vpinball_ini_file();
    let vpinball_config = VPinballConfig::read(&vpinball_ini_path).unwrap();
    let mut position = WindowPosition::default();
    let mut mode = WindowMode::Fullscreen(MonitorSelection::Primary);
    let mut resolution = WindowResolution::default();
    if let Some(playfield) = vpinball_config.get_playfield_info() {
        if let (Some(x), Some(y)) = (playfield.x, playfield.y) {
            // For macOS with scale factor > 1 this is not correct but we don't know the scale
            // factor before the window is created. We will correct the position later using the
            // system "correct_mac_window_size".
            let physical_x = x as i32;
            let physical_y = y as i32;
            position = WindowPosition::At(IVec2::new(physical_x, physical_y));
        }
        if let (Some(width), Some(height)) = (playfield.width, playfield.height) {
            resolution = WindowResolution::new(width as f32, height as f32);
        }
        mode = if playfield.fullscreen {
            WindowMode::Fullscreen(MonitorSelection::Primary)
        } else {
            WindowMode::Windowed
        };
    }
    println!(
        "Positioning window at {:?}, resolution {:?}",
        position, resolution
    );

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "VPXTOOL".to_string(),
                // window_level: WindowLevel::AlwaysOnTop,
                resolution,
                mode, // WindowMode::Windowed,
                position,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(AssetPaths {
            paths: HashMap::new(),
        })
        .insert_resource(Config { config })
        .insert_resource(VpxConfig {
            config: vpinball_config,
        })
        .insert_resource(VpxTables {
            indexed_tables: vpx_files_with_tableinfo,
        })
        .add_plugins(EguiPlugin)
        .add_plugins(PipelinesReadyPlugin)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .insert_resource(Globals {
            wheel_size: 100.0, // will be updated when loading wheels
            game_running: false,
        })
        .insert_resource(DialogBox {
            title: "Loading...".to_owned(),
            text: "blank".to_owned(),
        })
        //       .insert_resource(ClearColor(Color::srgb(0.9, 0.3, 0.6)))
        .add_event::<StreamEvent>()
        // TODO why does this happen so late?
        .add_systems(Startup, correct_window_size_and_position)
        .add_systems(Startup, setup)
        .add_systems(Startup, (create_wheel, create_flippers))
        .insert_resource(LoadingData::new(5))
        //       .insert_resource(ClearColor(Color::srgb(0.9, 0.3, 0.6)))
        .add_systems(
            Update,
            (load_loading_screen).run_if(in_state(LoadingState::LevelLoading)),
        )
        .add_systems(Startup, play_background_audio)
        //.add_systems(Update, gui_update)
        //.add_systems(Update,(guiupdate,update_loading_data, level_selection,display_loading_screen),)
        //.add_systems(
        //    Update,
        //    (update_loading_data, display_loading_screen),
        //)
        //.add_systems(Update, volume_system)
        //   .add_systems(Update,create_wheel)
        .add_systems(
            Update,
            //(display_loading_screen, read_stream, spawn_text, move_text),
            (display_loading_screen, read_stream),
        )
        .add_systems(
            Update,
            update_loading_data.run_if(in_state(LoadingState::LevelLoading)),
        )
        .add_systems(
            Update,
            gui_update.run_if(in_state(LoadingState::LevelReady)),
        )
        .init_state::<LoadingState>()
        .run();
    /*     eframe::run_native(
            "Image Viewer",
            options,
            Box::new(|cc| {
                // This gives us image support:
                egui_extras::install_image_loaders(&cc.egui_ctx);
                Ok(Box::<MyApp>::default())
            }),
        );
    */
}

fn play_background_audio(asset_server: Res<AssetServer>, mut commands: Commands) {
    // Create an entity dedicated to playing our background music
    let initialsettings = PlaybackSettings {
        mode: bevy::audio::PlaybackMode::Loop,
        paused: true,
        ..default()
    };

    commands.spawn(AudioBundle {
        source: bevy::prelude::AudioPlayer(asset_server.load("Pinball.ogg")),
        settings: initialsettings,
    });
}

/*fn volume_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    music_box_query: Query<&AudioSink, With<MusicBox>>
) {
    if let Ok(sink) = music_box_query.get_single() {
        if keyboard_input.just_pressed(KeyCode::Equal) {
            sink.set_volume(sink.volume() + 0.1);
        } else if keyboard_input.just_pressed(KeyCode::Minus) {
            sink.set_volume(sink.volume() - 0.1);
        }
    }
} */

#[derive(Resource, Deref)]
struct StreamReceiver(Receiver<u32>);

#[derive(Resource, Deref)]
struct StreamSender(Sender<u32>);

#[derive(Event)]
struct StreamEvent(u32);

use crossbeam_channel::{bounded, Receiver, Sender};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    let (tx, rx) = bounded::<u32>(10);

    commands.insert_resource(StreamSender(tx));
    commands.insert_resource(StreamReceiver(rx));
}

// This system reads from the receiver and sends events to Bevy
fn read_stream(
    mut window: Query<&mut Window>,
    receiver: Res<StreamReceiver>,
    mut events: EventWriter<StreamEvent>,
) {
    let mut window = window.single_mut();
    for from_stream in receiver.try_iter() {
        println!("Window visibility: {}", window.visible);
        println!("Showing window");
        window.visible = true;
        // bring window to front
        // window.window_level = WindowLevel::AlwaysOnTop;
        // request focus
        window.focused = true;
        events.send(StreamEvent(from_stream));
    }
}

/*  fn spawn_text(mut commands: Commands, mut reader: EventReader<StreamEvent>) {
    let text_style = TextFont::default();

    for (per_frame, event) in reader.read().enumerate() {
        commands.spawn(Text2d {
            text: Text::from_section(event.0.to_string(), text_style.clone())
                .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(per_frame as f32 * 100.0, 300.0, 0.0),
            ..default()
        });
    }
}

fn move_text(
    mut commands: Commands,
    mut texts: Query<(Entity, &mut Transform), With<Text>>,
    time: Res<Time>,
) {
    for (entity, mut position) in &mut texts {
        position.translation -= Vec3::new(0.0, 100.0 * time.delta_secs(), 0.0);
        if position.translation.y < -300.0 {
            commands.entity(entity).despawn();
        }
    }
}

*/

fn launch(selected_path: &PathBuf, vpinball_executable: &Path, fullscreen: Option<bool>) {
    println!("Launching {}", selected_path.display());

    if !vpinball_executable.is_executable() {
        report_and_exit(format!(
            "Unable to launch table, {} is not executable",
            vpinball_executable.display()
        ));
    }

    match launch_table(selected_path, vpinball_executable, fullscreen) {
        Ok(status) => match status.code() {
            Some(0) => {
                //println!("Table exited normally");
            }
            Some(11) => {
                eprintln!("Visual Pinball exited with segfault, you might want to report this to the vpinball team.");
            }
            Some(139) => {
                eprintln!("Visual Pinball exited with segfault, you might want to report this to the vpinball team.");
            }
            Some(code) => {
                eprintln!("Visual Pinball exited with code {}", code);
            }
            None => {
                eprintln!("Visual Pinball exited with unknown code");
            }
        },
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                report_and_exit(format!(
                    "Unable to launch table, vpinball executable not found at {}",
                    vpinball_executable.display()
                ));
            } else {
                report_and_exit(format!("Unable to launch table: {:?}", e));
            }
        }
    }
}

fn report_and_exit(msg: String) -> ! {
    eprintln!("CRASH {}", msg);
    exit(1);
}

fn launch_table(
    selected_path: &PathBuf,
    vpinball_executable: &Path,
    fullscreen: Option<bool>,
) -> io::Result<ExitStatus> {
    // start process ./VPinballX_GL -play [table path]
    let mut cmd = std::process::Command::new(vpinball_executable);
    match fullscreen {
        Some(true) => {
            cmd.arg("-EnableTrueFullscreen");
        }
        Some(false) => {
            cmd.arg("-DisableTrueFullscreen");
        }
        None => (),
    }
    cmd.arg("-play");
    cmd.arg(selected_path);
    let mut child = cmd.spawn()?;
    let result = child.wait()?;
    Ok(result)
}

fn display_table_line(table: &IndexedTable) -> String {
    let file_name = table
        .path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    Some(table.table_info.table_name.to_owned())
        .filter(|s| !s.clone().unwrap_or_default().is_empty())
        .map(|s| {
            format!(
                "{} {}",
                capitalize_first_letter(s.unwrap_or_default().as_str()),
                (format!("({})", file_name)).dimmed()
            )
        })
        .unwrap_or(file_name)
}

fn display_table_line_full(table: &IndexedTable, roms: &HashSet<String>) -> String {
    let base = display_table_line(table);
    let gamename_suffix = match &table.game_name {
        Some(name) => {
            let rom_found = table.local_rom_path.is_some() || roms.contains(&name.to_lowercase());
            if rom_found {
                format!(" - [{}]", name.dimmed())
            } else if table.requires_pinmame {
                format!(" - {} [{}]", Emoji("⚠️", "!"), &name)
                    .yellow()
                    .to_string()
            } else {
                format!(" - [{}]", name.dimmed())
            }
        }
        None => "".to_string(),
    };
    let b2s_suffix = match &table.b2s_path {
        Some(_) => " ▀".dimmed(),
        None => "".into(),
    };
    format!("{}{}{}", base, gamename_suffix, b2s_suffix)
}

fn capitalize_first_letter(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
}
