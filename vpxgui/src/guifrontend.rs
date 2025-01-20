use bevy::color::palettes::css::*;

use crate::dmd::dmd_plugin;
use crate::flippers::flipper_plugin;
use crate::info::show_info;
use crate::list::table_selection;
use crate::loading::loading_plugin;
use crate::loading::{LoadingData, LoadingState};
use crate::menus::*;
use crate::music::{music_plugin, resume_music, suspend_music, ControlMusicEvent};
use crate::pipelines::PipelinesReadyPlugin;
use crate::process::{do_launch, VpxEvent};
use crate::wheel::{wheel_plugin, TextItemGold, Wheel};
use crate::windowing;
use crate::windowing::WindowingPlugin;
use bevy::prelude::*;
use bevy::window::*;
use bevy_egui::EguiPlugin;
use bevy_mini_fps::fps_plugin;
use shared::config::{ResolvedConfig, VPinballConfig};
use shared::indexer::IndexedTable;

#[derive(Component, Debug)]
pub struct TableText {
    pub item_number: i16,
    pub table_text: String,
    pub table_blurb: String,
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

#[derive(Resource, Debug)]
pub(crate) struct Globals {
    pub wheel_size: f32,
    pub game_running: bool,
    pub selected_item: Option<i16>,
}

fn launcher(
    keys: Res<ButtonInput<KeyCode>>,
    mut control_music_event_writer: EventWriter<ControlMusicEvent>,
    stream_sender: Res<crate::process::StreamSender>,
    config: Res<Config>,
    mut globals: ResMut<Globals>,
    wheels: Query<&mut Wheel>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        if globals.game_running {
            warn!("Game already running");
            return;
        };

        if let Some(selected_item) = globals.selected_item {
            let mut window = window_query.single_mut();

            suspend_music(&mut control_music_event_writer);
            for wheel in wheels.iter() {
                if wheel.item_number == selected_item {
                    info!("Hide window");
                    window.visible = false;
                    globals.game_running = true;
                    do_launch(
                        stream_sender.clone(),
                        &wheel.launch_path,
                        &config.config.vpx_executable,
                    );
                }
            }
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

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn gui_update(
    _time: Res<Time>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut set: ParamSet<(
        Query<(&mut TableText, &mut TextFont, &mut Node, &mut TextColor), With<TextItemGold>>,
        Query<(&mut Visibility, &mut Wheel, &mut Transform), With<Wheel>>,
    )>,
    globals: Res<Globals>,
) {
    let mut window = window_query.get_single().unwrap().clone();
    window.window_level = WindowLevel::Normal;

    // from here on no more changes to selected_item, make it immutable
    let selected_item = globals.selected_item.unwrap_or(0);

    // change name of game
    for (items, mut font, mut textstyle, mut color) in set.p0().iter_mut() {
        if items.item_number != selected_item {
            textstyle.display = Display::None;
            *color = TextColor::from(GHOST_WHITE);
        } else {
            *color = TextColor::from(GHOST_WHITE);
            font.font_size = 20.0;
            textstyle.display = Display::Block;
        }
    }

    // table scroll
    let mut counter = 11;
    let mut names = [0; 21];

    // Count entities
    let mut num = 1;
    num += set.p1().iter().count() as i16;

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
            names[count - 2] = selected_item + counter;
        } else if selected_item + counter + 3 > num {
            names[count - 2] = (selected_item + counter - num) + 1
        }
        //        else  {names[count-2] = (selected_item+1)-counter;};
        counter += 1;
    }
    counter = 0;

    // clear all game name assets
    for (_items, mut fontsize, mut textstyle, mut color) in set.p0().iter_mut() {
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
            for (items, mut fontsize, mut text_style, mut color) in set.p0().iter_mut() {
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
}

fn resume_after_play(
    mut reader: EventReader<VpxEvent>,
    mut event_writer: EventWriter<ControlMusicEvent>,
    mut globals: ResMut<Globals>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    for event in reader.read() {
        info!("Event: {:?}", event);
        globals.game_running = false;
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
}

/* #[derive(Resource)]
pub struct AssetPath {
    pub handle: Handle<Image>,
    pub path: OsString,
} */

// Marker component for easier deletion of entities.
//#[derive(Component)]
//struct LevelComponents;

// Removes all currently loaded level assets from the game World.
/*fn unload_current_level(
    mut commands: Commands,
    // mut loading_state: ResMut<LoadingState>,
    entities: Query<Entity, With<LevelComponents>>,
) {
    // *loading_state = LoadingState::LevelLoading;
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}*/

/* fn level_selection(
    commands: Commands,
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
*/

pub fn guifrontend(config: ResolvedConfig, vpx_files_with_tableinfo: Vec<IndexedTable>) {
    //    let options = eframe::NativeOptions {
    //       viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 800.0]),
    //       ..Default::default()
    //   };

    let mut tables: Vec<IndexedTable> = vpx_files_with_tableinfo;
    tables.sort_by_key(|indexed| display_table_line(indexed).to_lowercase());

    let vpinball_ini_path = config.vpinball_ini_file();
    let vpinball_config = VPinballConfig::read(&vpinball_ini_path).unwrap();
    let window = windowing::setup_playfield_window(&vpinball_config);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..Default::default()
        }))
        .add_plugins(WindowingPlugin)
        .add_plugins(fps_plugin!())
        .add_plugins(music_plugin)
        .add_plugins((wheel_plugin, flipper_plugin, dmd_plugin))
        .add_plugins(loading_plugin)
        .insert_resource(Config { config })
        .insert_resource(VpxConfig {
            config: vpinball_config,
        })
        .insert_resource(VpxTables {
            indexed_tables: tables,
        })
        .add_plugins(EguiPlugin)
        .add_plugins(PipelinesReadyPlugin)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .insert_resource(Globals {
            wheel_size: 100.0, // will be updated when loading wheels
            game_running: false,
            selected_item: None,
        })
        //       .insert_resource(ClearColor(Color::srgb(0.9, 0.3, 0.6)))
        .add_event::<VpxEvent>()
        .add_systems(Startup, setup)
        .insert_resource(LoadingData::new(5))
        //       .insert_resource(ClearColor(Color::srgb(0.9, 0.3, 0.6)))
        .add_systems(Update, quit_on_q)
        //.add_systems(Update,(guiupdate,update_loading_data, level_selection,display_loading_screen),)
        //.add_systems(
        //    Update,
        //    (update_loading_data, display_loading_screen),
        //)
        //.add_systems(Update, volume_system)
        //   .add_systems(Update,create_wheel)
        .add_systems(Update, resume_after_play)
        .add_systems(Update, gui_update.run_if(in_state(LoadingState::Ready)))
        .add_systems(Update, launcher.run_if(in_state(LoadingState::Ready)))
        .add_systems(
            Update,
            table_selection.run_if(in_state(LoadingState::Ready)),
        )
        .add_systems(Update, dmd_update.run_if(in_state(LoadingState::Ready)))
        .add_systems(Update, show_info.run_if(in_state(LoadingState::Ready)))
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

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
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
                "{} ({})",
                capitalize_first_letter(s.unwrap_or_default().as_str()),
                file_name
            )
        })
        .unwrap_or(file_name)
}

fn capitalize_first_letter(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
}
