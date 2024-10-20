use std::collections::HashSet;
use std::{
    fs::File,
    io,
    io::Write,
    path::{Path, PathBuf},
    process::{exit, ExitStatus},
};
use image::ImageReader;
use bevy::render::view::visibility;
use bevy::{input::common_conditions::*,prelude::*};
use bevy::color::palettes::css::*;
use bevy_asset_loader::prelude::*;
use bevy_asset::*;
use bevy::window::*;
use colored::Colorize;
use console::Emoji;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{FuzzySelect, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use is_executable::IsExecutable;
use std::ffi::{OsStr, OsString};
use crate::config;
use crate::config::ResolvedConfig;
use crate::indexer::{find_vpx_files, IndexError, IndexedTable, Progress};
use crate::patcher::LineEndingsResult::{NoChanges, Unified};
use crate::patcher::{patch_vbs_file, unify_line_endings_vbs_file};
use crate::{
    indexer, info_diff, info_edit, info_gather, open_editor, run_diff, script_diff,
    vpx::{extractvbs, vbs_path_for, ExtractResult},
    DiffColor, ProgressBarProgress,
};

const LAUNCH: Emoji = Emoji("🚀", "[launch]");
const CRASH: Emoji = Emoji("💥", "[crash]");

const SEARCH: &str = "> Search";
const RECENT: &str = "> Recent";
const SEARCH_INDEX: usize = 0;
const RECENT_INDEX: usize = 1;
const HORIZONTAL:bool = false;
const VERTICAL:bool = true;

#[derive(Component)]
pub struct Wheel {
    pub itemnumber: i16,
    pub image_handle: Handle<Image>,
    pub selected: bool,
    pub launchpath:PathBuf,
    pub vpxexecutable:PathBuf,
}

#[derive(Component,Debug)]
pub struct TableText {
    pub itemnumber: i16,
    pub has_wheel: bool,
    }

fn create_wheel(mut commands: Commands, asset_server: Res<AssetServer>, window_query: Query<&Window, With<PrimaryWindow>>,assets: Res<Assets<Image>>,)  
{
    //config: &ResolvedConfig,
    //vpx_files_with_tableinfo: &mut Vec<IndexedTable>,
    //vpinball_executable: &Path,
    //info: &IndexedTable,
    //info_str: &str,
  
   //// commands.spawn(SpriteBundle {
   ////     texture: asset_server.load("/usr/tables/wheels/Sing Along (Gottlieb 1967).png"),
    ////    ..default()
   //// });

    let (_config_path,loaded_config) = config::load_config().unwrap().unwrap();
    let vpx_files_with_tableinfo = frontend_index(&loaded_config, true, vec![]); 
    let vpx_files_with_tableinfo1 = frontend_index(&loaded_config, true, vec![]).unwrap(); 
    let mut temporary_path_name= PathBuf::from("");
    let roms = indexer::find_roms(loaded_config.global_pinmame_rom_folder());
    let roms1 = roms.unwrap();
    let tables: Vec<String> = vpx_files_with_tableinfo.unwrap()
    .iter()
    .map(|indexed| display_table_line_full(indexed, &roms1))
    .collect();
    let temporary_path_name="";   
    
    println!("Last table {:?}",&loaded_config.last_table);
    let window = window_query.get_single().unwrap();
    let mut width = window.width();
    let mut height = window.height();
    let table_path  = loaded_config.tables_folder;

    let mut orentation = HORIZONTAL;
    if height > width {orentation=VERTICAL;}
        else {orentation=HORIZONTAL};
    
    let mut scale = width/10.;
    let tables_len= tables.len();
    let mut entities=0.;
    let mut counter: usize=0;
    let mut xlocation =0;
    let locations = [-(width/2.)+scale,-(scale*2.),0.,(scale*2.),(width/2.) - (scale)];
    //let mut handles =[];

    let mut transform = Transform::from_xyz(0., 0., 0.);

    //let mut transform = Transform::from_xyz(0., -(height-(height/2.+(scale*2.))), 0.);
    //let mut transform = Transform::from_xyz(locations[xlocation], -(height-(height/2.+(scale*2.))), 0.);
    
    while counter < (tables_len)
        {
        if xlocation > 4 {xlocation = 0};

        let info = vpx_files_with_tableinfo1
        .get(counter)
        .unwrap()
        .clone();
        /*    match &info.wheel_path {
                Some(path)=> println!("{}",&path.as_os_str().to_string_lossy()),
                None => println!("NONE"),
            };
      */
        let mut haswheel = true;
        //let mut temporary_path_name= &info.wheel_path.unwrap();
        // create blank wheel
        let mut blank_path = table_path.clone().into_os_string();
        blank_path.push("/wheels/blankwheel.png");
        //blank_path.into();
        
        let mut temporary_path_name = match &info.wheel_path {
            // get handle from path
            Some(path) => {haswheel = false;PathBuf::from(path)},
            None => {haswheel = true; PathBuf::from(blank_path)},
            };
        // let mut temporary_table_name="None";
        //let mut handle =  asset_server.load(temporary_path_name);  
        let temporary_table_name = match &info.table_info.table_name {
            Some(tb) => &tb,
            None => "None",
            };

         let mut handle = asset_server.load(temporary_path_name.clone());
          // Normalizing the dimentions of wheels so they are all the same size.
         //  using imagesize crate as it is a very fast way to get the dimentions.

         match imagesize::size(&temporary_path_name) {
            Ok(size) => {
                // Normalize icons to 1/3 the screen height
                transform.scale = Vec3::new((height/3.)/(size.height as f32),
                                            (height/3.)/(size.height as f32), 100.0);
                        println!("Initializing:  {}",&temporary_path_name.as_os_str().to_string_lossy());},
                        Err(why) => println!("Error getting dimensions: {} {:?}",
                                                &temporary_path_name.as_os_str().to_string_lossy(),why)
            };

         commands.spawn( (SpriteBundle {
           // texture: asset_server.load("/usr/tables/wheels/Sing Along (Gottlieb 1967).png"),
            texture: handle.clone(),
            transform: transform,
            ..default()
            },
            Wheel {
                itemnumber: counter as i16,
                image_handle: handle.clone(),
                selected: false,
                launchpath: info.path,
                vpxexecutable: loaded_config.vpx_executable.clone(),
            },
            
        )); 

       commands.spawn( (
                // Create a TextBundle that has a Text with a single section.
                TextBundle::from_section(
                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                   
                        temporary_table_name,
                        TextStyle {
                        // This font is loaded and will be used instead of the default font.
                        font_size: 30.0,
                        color: GOLD.into(),
                        ..default()
                    },
                ) // Set the justification of the Text
                //.with_text_justify(JustifyText::Center)
                // Set the style of the TextBundle itself.
                .with_style(Style {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    left: Val::Px(20.),
                    top: Val::Px(height*0.025),//-(height-(height/2.+(scale*2.)))),
                   // right: Val::Px((0.)),
                    ..default()
                }),
                TableText {
                    itemnumber: counter as i16,
                    has_wheel: haswheel,
                },
              ));

         
       //let image = image::load(BufReader::new(File::open("foo.png")?), ImageFormat::Jpeg)?;
        counter += 1;
        xlocation +=1;
        entities +=1. ;
     
     };

     commands.spawn(Camera2dBundle::default());
     println!("Wheels loaded");
        
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


pub fn guiupdate(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>,time: Res<Time>, 
                                mut query: Query<(&mut Transform, &mut Wheel)>,
                                window_query: Query<&Window, With<PrimaryWindow>>,
                                mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
                                mut tabletext: Query<(&mut Style, &mut TableText)>,)
{
    let window = window_query.get_single().unwrap();

    let mut width = window.width();
    let mut height = window.height();

    let mut orentation = HORIZONTAL;
    if height > width {orentation=VERTICAL;}
        else {orentation=HORIZONTAL};
    
    let mut scale = width/10.;

    // arbitrary number to indicate there is no selected item.
    let mut selected_item:i16=-2;
    
    // set a flag indicating if we are ready to launch a game
    let mut launchit = false;

    // Count entities - there is probably a function call to do the same thing but this was
    // quick and dirty.
    let mut num =1;
    for (mut transform,
        mut wheel ) in query.iter_mut()
        {num +=1;}

    // Find current selection
    for (mut transform,
        mut wheel) in query.iter_mut()
        {
        match wheel.selected {
            true =>{selected_item = wheel.itemnumber;}, 
            _ => ()
            }
        }
        // If no selection, set it to item 3
        if selected_item == -2 {
            for (mut transform,
                 mut wheel) in query.iter_mut()
            {
                match wheel.itemnumber {
                    3 => {wheel.selected = true;selected_item=3;},
                    _ => ()
                }
            }};
         
            // check for keypress right shift select next item, left shift select previous item         
            for ev in keys.get_just_released() {
                    match ev{
                    KeyCode::ShiftRight => {selected_item +=1;},
                    KeyCode::ShiftLeft =>  {selected_item -=1;},
                    KeyCode::Enter => {launchit = true;},
                    KeyCode::KeyQ => {app_exit_events.send(bevy::app::AppExit::Success);},
                    KeyCode::Space => {println!("current table {}",selected_item);},
                           _ => { println!("Key press: ({:?})", ev); },
                    }};
                
           // Wrap around if one of the bounds are hit.
           if selected_item == num-1 {selected_item=0;}
           else {if selected_item == -1 {selected_item=num-2;};};

           // update currently selected item to new value
           for (mut transform, mut wheel) in query.iter_mut()
                {
                 if wheel.itemnumber != selected_item 
                    { wheel.selected = false;transform.translation = Vec3::new(0., width, 0.);}
                else {wheel.selected = true;
                    transform.translation = Vec3::new(0., -(height-(height/2.+(scale*2.))), 0.);
                //    println!("Selected {}",&wheel.launchpath.as_os_str().to_string_lossy());
                }
                };
   
            for (mut textstyle, mut items) in tabletext.iter_mut()
           {
            if items.itemnumber != selected_item
                {textstyle.display=Display::None;}
                else {
                        textstyle.display=Display::Block;
                }
   
            };

       if launchit {
       for (mut transform,  mut wheel) in query.iter_mut()
       {
            if wheel.itemnumber == selected_item {
                println!("Launching {}",wheel.launchpath.clone().into_os_string().to_string_lossy());
                launch(&wheel.launchpath, &wheel.vpxexecutable, None);}
       };
    }
     
}
 
    
   // if ctrl && shift && input.just_pressed(KeyCode::KeyA) {
     //   info!("Just pressed Ctrl + Shift + A!"); }

pub fn guifrontend(
    config: &ResolvedConfig,
    mut vpx_files_with_tableinfo: Vec<IndexedTable>,
    roms: &HashSet<String>,
    vpinball_executable: &Path,
) {
    let tables: Vec<String> = vpx_files_with_tableinfo
    .iter()
    .map(|indexed| display_table_line_full(indexed, roms))
    .collect();
    let path = "/usr/tables/wheels/Sing Along (Gottlieb 1967).png";
    
//    let options = eframe::NativeOptions {
//       viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 800.0]),
//       ..Default::default()
 //   };
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "VPXTOOL".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }))
        .insert_resource(ClearColor(Color::srgb(0.9, 0.3, 0.6)))
        .add_systems(Startup,create_wheel)
        .add_systems(Update,guiupdate)
     //   .add_systems(Update,create_wheel)
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

/*fn table_menu(
    config: &ResolvedConfig,
    vpx_files_with_tableinfo: &mut Vec<IndexedTable>,
    vpinball_executable: &Path,
    info: &IndexedTable,
    info_str: &str,
) {
    let selected_path = &info.path;
    match choose_table_option(info_str) {
        Some(TableOption::Launch) => {
            launch(selected_path, vpinball_executable, None);
        }
        Some(TableOption::LaunchFullscreen) => {
            launch(selected_path, vpinball_executable, Some(true));
        }
        Some(TableOption::LaunchWindowed) => {
            launch(selected_path, vpinball_executable, Some(false));
        }
        Some(TableOption::ForceReload) => {
            match frontend_index(config, true, vec![selected_path.clone()]) {
                Ok(index) => {
                    vpx_files_with_tableinfo.clear();
                    vpx_files_with_tableinfo.extend(index);
                }
                Err(err) => {
                    let msg = format!("Unable to reload tables: {:?}", err);
                    prompt(msg.truecolor(255, 125, 0).to_string());
                }
            }
        }
        Some(TableOption::EditVBS) => {
            let path = vbs_path_for(selected_path);
            let result = if path.exists() {
                open_editor(&path, Some(config))
            } else {
                extractvbs(selected_path, false, None);
                open_editor(&path, Some(config))
            };
            match result {
                Ok(_) => {
                    println!("Launched editor for {}", path.display());
                }
                Err(err) => {
                    let msg = format!("Unable to edit VBS: {}", err);
                    prompt(msg.truecolor(255, 125, 0).to_string());
                }
            }
        }
        Some(TableOption::ExtractVBS) => match extractvbs(selected_path, false, None) {
            ExtractResult::Extracted(path) => {
                prompt(format!("VBS extracted to {}", path.to_string_lossy()));
            }
            ExtractResult::Existed(path) => {
                let msg = format!("VBS already exists at {}", path.to_string_lossy());
                prompt(msg.truecolor(255, 125, 0).to_string());
            }
        },
        Some(TableOption::ShowVBSDiff) => match script_diff(selected_path) {
            Ok(diff) => {
                prompt(diff);
            }
            Err(err) => {
                let msg = format!("Unable to diff VBS: {}", err);
                prompt(msg.truecolor(255, 125, 0).to_string());
            }
        },
        Some(TableOption::PatchVBS) => {
            let vbs_path = match extractvbs(selected_path, false, Some("vbs")) {
                ExtractResult::Existed(path) => path,
                ExtractResult::Extracted(path) => path,
            };
            match patch_vbs_file(&vbs_path) {
                Ok(applied) => {
                    if applied.is_empty() {
                        prompt("No patches applied.".to_string());
                    } else {
                        applied.iter().for_each(|patch| {
                            println!("Applied patch: {}", patch);
                        });
                        prompt(format!(
                            "Patched VBS file at {}",
                            vbs_path.to_string_lossy()
                        ));
                    }
                }
                Err(err) => {
                    let msg = format!("Unable to patch VBS: {}", err);
                    prompt(msg.truecolor(255, 125, 0).to_string());
                }
            }
        }
        Some(TableOption::UnifyLineEndings) => {
            let vbs_path = match extractvbs(selected_path, false, Some("vbs")) {
                ExtractResult::Existed(path) => path,
                ExtractResult::Extracted(path) => path,
            };
            match unify_line_endings_vbs_file(&vbs_path) {
                Ok(NoChanges) => {
                    prompt("No changes applied as file has correct line endings".to_string());
                }
                Ok(Unified) => {
                    prompt(format!(
                        "Unified line endings in VBS file at {}",
                        vbs_path.to_string_lossy()
                    ));
                }
                Err(err) => {
                    let msg = format!("Unable to patch VBS: {}", err);
                    prompt(msg.truecolor(255, 125, 0).to_string());
                }
            }
        }
        Some(TableOption::CreateVBSPatch) => {
            let original_path = match extractvbs(selected_path, true, Some("vbs.original")) {
                ExtractResult::Existed(path) => path,
                ExtractResult::Extracted(path) => path,
            };
            let vbs_path = vbs_path_for(selected_path);
            let patch_path = vbs_path.with_extension("vbs.patch");

            match run_diff(&original_path, &vbs_path, DiffColor::Never) {
                Ok(diff) => {
                    let mut file = File::create(patch_path).unwrap();
                    file.write_all(&diff).unwrap();
                }
                Err(err) => {
                    let msg = format!("Unable to diff VBS: {}", err);
                    prompt(msg.truecolor(255, 125, 0).to_string());
                }
            }
        }
        Some(TableOption::InfoShow) => match info_gather(selected_path) {
            Ok(info) => {
                prompt(info);
            }
            Err(err) => {
                let msg = format!("Unable to gather table info: {}", err);
                prompt(msg.truecolor(255, 125, 0).to_string());
            }
        },
        Some(TableOption::InfoEdit) => match info_edit(selected_path, Some(config)) {
            Ok(path) => {
                println!("Launched editor for {}", path.display());
            }
            Err(err) => {
                let msg = format!("Unable to edit table info: {}", err);
                prompt(msg.truecolor(255, 125, 0).to_string());
            }
        },
        Some(TableOption::InfoDiff) => match info_diff(selected_path) {
            Ok(diff) => {
                prompt(diff);
            }
            Err(err) => {
                let msg = format!("Unable to diff info: {}", err);
                prompt(msg.truecolor(255, 125, 0).to_string());
            }
        },
        None => (),
    }
}
*/
fn prompt<S: Into<String>>(msg: S) {
    Input::<String>::new()
        .with_prompt(format!("{} - Press enter to continue.", msg.into()))
        .default("".to_string())
        .show_default(false)
        .interact()
        .unwrap();
}

fn choose_table_option(table_name: &str) -> Option<TableOption> {
    // iterate over table options
    let selections = TableOption::ALL
        .iter()
        .map(|option| option.display())
        .collect::<Vec<String>>();

    let selection_opt = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(table_name)
        .default(0)
        .items(&selections[..])
        .interact_opt()
        .unwrap();

    selection_opt.and_then(TableOption::from_index)
}

fn launch(selected_path: &PathBuf, vpinball_executable: &Path, fullscreen: Option<bool>) {
    println!("{} {}", LAUNCH, selected_path.display());

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
                prompt(format!("{} Visual Pinball exited with segfault, you might want to report this to the vpinball team.", CRASH));
            }
            Some(139) => {
                prompt(format!("{} Visual Pinball exited with segfault, you might want to report this to the vpinball team.", CRASH));
            }
            Some(code) => {
                prompt(format!(
                    "{} Visual Pinball exited with code {}",
                    CRASH, code
                ));
            }
            None => {
                prompt("Visual Pinball exited with unknown code");
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
    eprintln!("{CRASH} {}", msg);
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
