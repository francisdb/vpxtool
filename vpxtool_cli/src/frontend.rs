use crate::backglass::find_hole;
use crate::patcher::LineEndingsResult::{NoChanges, Unified};
use crate::patcher::{patch_vbs_file, unify_line_endings_vbs_file};
use crate::{
    DiffColor, ProgressBarProgress, confirm, info_diff, info_edit, info_gather, open_editor,
    run_diff, script_diff, strip_cr_lf,
    vpx::{ExtractResult, extractvbs, ini_path_for, vbs_path_for},
};
use base64::Engine;
use colored::Colorize;
use console::Emoji;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{FuzzySelect, Input, MultiSelect, Select};
use image::{DynamicImage, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use is_executable::IsExecutable;
use pinmame_nvram::dips::{get_all_dip_switches, set_dip_switches};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::{
    fs::File,
    io,
    io::Write,
    path::{Path, PathBuf},
    process::{ExitStatus, exit},
};
use vpxtool_shared::config::ResolvedConfig;
use vpxtool_shared::indexer;
use vpxtool_shared::indexer::{IndexError, IndexedTable, Progress};
use vpxtool_shared::vpinball_config::{VPinballConfig, WindowInfo, WindowType};
use xcap::XCapError;

const LAUNCH: Emoji = Emoji("🚀", "[launch]");
const CRASH: Emoji = Emoji("💥", "[crash]");

const SEARCH: &str = "> Search";
const RECENT: &str = "> Recent";
const SEARCH_INDEX: usize = 0;
const RECENT_INDEX: usize = 1;

#[derive(PartialEq, Eq)]
enum TableOption {
    Launch,
    LaunchFullscreen,
    LaunchWindowed,
    ForceReload,
    CaptureWindows,
    InfoShow,
    InfoEdit,
    InfoDiff,
    ExtractVBS,
    EditVBS,
    PatchVBS,
    UnifyLineEndings,
    ShowVBSDiff,
    CreateVBSPatch,
    DIPSwitches,
    NVRAMClear,
    B2SAutoPositionDMD,
    EditINI,
}

impl TableOption {
    const ALL: [TableOption; 18] = [
        TableOption::Launch,
        TableOption::LaunchFullscreen,
        TableOption::LaunchWindowed,
        TableOption::ForceReload,
        TableOption::CaptureWindows,
        TableOption::InfoShow,
        TableOption::InfoEdit,
        TableOption::InfoDiff,
        TableOption::ExtractVBS,
        TableOption::EditVBS,
        TableOption::PatchVBS,
        TableOption::UnifyLineEndings,
        TableOption::ShowVBSDiff,
        TableOption::CreateVBSPatch,
        TableOption::DIPSwitches,
        TableOption::NVRAMClear,
        TableOption::B2SAutoPositionDMD,
        TableOption::EditINI,
    ];

    fn from_index(index: usize) -> Option<TableOption> {
        match index {
            0 => Some(TableOption::Launch),
            1 => Some(TableOption::LaunchFullscreen),
            2 => Some(TableOption::LaunchWindowed),
            3 => Some(TableOption::ForceReload),
            4 => Some(TableOption::CaptureWindows),
            5 => Some(TableOption::InfoShow),
            6 => Some(TableOption::InfoEdit),
            7 => Some(TableOption::InfoDiff),
            8 => Some(TableOption::ExtractVBS),
            9 => Some(TableOption::EditVBS),
            10 => Some(TableOption::PatchVBS),
            11 => Some(TableOption::UnifyLineEndings),
            12 => Some(TableOption::ShowVBSDiff),
            13 => Some(TableOption::CreateVBSPatch),
            14 => Some(TableOption::DIPSwitches),
            15 => Some(TableOption::NVRAMClear),
            16 => Some(TableOption::B2SAutoPositionDMD),
            17 => Some(TableOption::EditINI),
            _ => None,
        }
    }

    fn display(&self) -> String {
        match self {
            TableOption::Launch => "Launch".to_string(),
            TableOption::LaunchFullscreen => "Launch fullscreen".to_string(),
            TableOption::LaunchWindowed => "Launch windowed".to_string(),
            TableOption::ForceReload => "Force reload".to_string(),
            TableOption::CaptureWindows => "Capture windows".to_string(),
            TableOption::InfoShow => "Info > Show".to_string(),
            TableOption::InfoEdit => "Info > Edit".to_string(),
            TableOption::InfoDiff => "Info > Diff".to_string(),
            TableOption::ExtractVBS => "VBScript > Extract".to_string(),
            TableOption::EditVBS => "VBScript > Edit".to_string(),
            TableOption::PatchVBS => "VBScript > Patch typical standalone issues".to_string(),
            TableOption::UnifyLineEndings => "VBScript > Unify line endings".to_string(),
            TableOption::ShowVBSDiff => "VBScript > Diff".to_string(),
            TableOption::CreateVBSPatch => "VBScript > Create patch file".to_string(),
            TableOption::DIPSwitches => "DIP Switches".to_string(),
            TableOption::NVRAMClear => "NVRAM > Clear".to_string(),
            TableOption::B2SAutoPositionDMD => "Backglass > Auto-position DMD".to_string(),
            TableOption::EditINI => "INI > Edit".to_string(),
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
        Some(&resolved_config.global_pinmame_rom_folder()),
        &progress,
        force_reindex,
    );
    progress.finish_and_clear();
    let index = index?;

    let mut tables: Vec<IndexedTable> = index.tables();
    tables.sort_by_key(|indexed| display_table_line(indexed).to_lowercase());
    Ok(tables)
}

pub fn frontend(
    config: &ResolvedConfig,
    mut vpx_files_with_tableinfo: Vec<IndexedTable>,
    vpinball_executable: &Path,
) {
    let mut main_selection_opt = None;
    loop {
        let tables: Vec<String> = vpx_files_with_tableinfo
            .iter()
            .map(display_table_line_full)
            .collect();

        let mut selections = vec![SEARCH.bold().to_string(), RECENT.bold().to_string()];
        selections.extend(tables.clone());

        main_selection_opt = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a table")
            .default(main_selection_opt.unwrap_or(0))
            .items(&selections[..])
            .interact_opt()
            .unwrap();

        match main_selection_opt {
            Some(selection) => {
                // search

                match selection {
                    SEARCH_INDEX => {
                        // show a fuzzy search
                        let selected = FuzzySelect::with_theme(&ColorfulTheme::default())
                            .with_prompt("Search a table:")
                            .items(&tables)
                            .interact_opt()
                            .unwrap();

                        if let Some(selected_index) = selected {
                            let info = vpx_files_with_tableinfo
                                .get(selected_index)
                                .unwrap()
                                .clone();
                            let info_str = display_table_line_full(&info);
                            table_menu(
                                config,
                                &mut vpx_files_with_tableinfo,
                                vpinball_executable,
                                &info,
                                &info_str,
                            );
                        }
                    }
                    RECENT_INDEX => {
                        // take the last 10 most recent tables
                        let mut recent: Vec<IndexedTable> = vpx_files_with_tableinfo.clone();
                        recent.sort_by_key(|indexed| indexed.last_modified);
                        let last_modified = recent.iter().rev().take(50).collect::<Vec<_>>();
                        let last_modified_str: Vec<String> = last_modified
                            .iter()
                            .map(|indexed| display_table_line_full(indexed))
                            .collect();

                        let selected = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Select a table")
                            .items(&last_modified_str)
                            .default(0)
                            .interact_opt()
                            .unwrap();

                        if let Some(selected_index) = selected {
                            let info = last_modified.get(selected_index).unwrap();
                            let info_str = display_table_line_full(info);
                            table_menu(
                                config,
                                &mut vpx_files_with_tableinfo,
                                vpinball_executable,
                                info,
                                &info_str,
                            );
                        }
                    }
                    _ => {
                        let index = selection - 2;

                        let info = vpx_files_with_tableinfo.get(index).unwrap().clone();
                        let info_str = display_table_line_full(&info);
                        table_menu(
                            config,
                            &mut vpx_files_with_tableinfo,
                            vpinball_executable,
                            &info,
                            &info_str,
                        );
                    }
                }
            }
            None => break,
        };
    }
}

fn table_menu(
    config: &ResolvedConfig,
    vpx_files_with_tableinfo: &mut Vec<IndexedTable>,
    vpinball_executable: &Path,
    info: &IndexedTable,
    info_str: &str,
) {
    let selected_path = &info.path;
    let mut exit = false;
    let mut option = None;
    while !exit {
        option = choose_table_option(info_str, option);
        match option {
            Some(TableOption::Launch) => {
                launch(selected_path, vpinball_executable, None);
                exit = true;
            }
            Some(TableOption::LaunchFullscreen) => {
                launch(selected_path, vpinball_executable, Some(true));
                exit = true;
            }
            Some(TableOption::LaunchWindowed) => {
                launch(selected_path, vpinball_executable, Some(false));
                exit = true;
            }
            Some(TableOption::ForceReload) => {
                match frontend_index(config, true, vec![selected_path.clone()]) {
                    Ok(index) => {
                        vpx_files_with_tableinfo.clear();
                        vpx_files_with_tableinfo.extend(index);
                        // exit to not have to
                        //  * check if the table is still in the list
                        //  * check if the info_str has changed
                        exit = true;
                    }
                    Err(err) => {
                        prompt_error(&format!("Unable to reload tables: {:?}", err));
                    }
                }
            }
            Some(TableOption::CaptureWindows) => match capture_windows(info) {
                Ok(count) => {
                    prompt(&format!("Captured {} windows", count));
                }
                Err(err) => {
                    prompt_error(&format!("Unable to capture windows: {:?}", err));
                }
            },
            Some(TableOption::EditVBS) => {
                let path = vbs_path_for(selected_path);
                let result = if path.exists() {
                    open_editor(&path, Some(config))
                } else {
                    extractvbs(selected_path, None, false)
                        .and_then(|_| open_editor(&path, Some(config)))
                };
                match result {
                    Ok(_) => {
                        println!("Launched editor for {}", path.display());
                    }
                    Err(err) => {
                        let msg = format!("Unable to edit VBS: {}", err);
                        prompt(&msg.truecolor(255, 125, 0).to_string());
                    }
                }
            }
            Some(TableOption::ExtractVBS) => match extractvbs(selected_path, None, false) {
                Ok(ExtractResult::Extracted(path)) => {
                    prompt(&format!("VBS extracted to {}", path.to_string_lossy()));
                }
                Ok(ExtractResult::Existed(path)) => {
                    let msg = format!("VBS already exists at {}", path.to_string_lossy());
                    prompt(&msg.truecolor(255, 125, 0).to_string());
                }
                Err(err) => {
                    let msg = format!("Unable to extract VBS: {}", err);
                    prompt(&msg.truecolor(255, 125, 0).to_string());
                }
            },
            Some(TableOption::ShowVBSDiff) => match script_diff(selected_path) {
                Ok(diff) => {
                    prompt(&diff);
                }
                Err(err) => {
                    let msg = format!("Unable to diff VBS: {}", err);
                    prompt(&msg.truecolor(255, 125, 0).to_string());
                }
            },
            Some(TableOption::PatchVBS) => {
                let vbs_path = match extractvbs(selected_path, None, false) {
                    Ok(ExtractResult::Existed(path)) => path,
                    Ok(ExtractResult::Extracted(path)) => path,
                    Err(err) => {
                        let msg = format!("Unable to extract VBS: {}", err);
                        prompt(&msg.truecolor(255, 125, 0).to_string());
                        return;
                    }
                };
                match patch_vbs_file(&vbs_path) {
                    Ok(applied) => {
                        if applied.is_empty() {
                            prompt("No patches applied.");
                        } else {
                            applied.iter().for_each(|patch| {
                                println!("Applied patch: {}", patch);
                            });
                            prompt(&format!(
                                "Patched VBS file at {}",
                                vbs_path.to_string_lossy()
                            ));
                        }
                    }
                    Err(err) => {
                        let msg = format!("Unable to patch VBS: {}", err);
                        prompt(&msg.truecolor(255, 125, 0).to_string());
                    }
                }
            }
            Some(TableOption::UnifyLineEndings) => {
                let vbs_path = vbs_path_for(selected_path);
                let vbs_path = match extractvbs(selected_path, Some(vbs_path), false) {
                    Ok(ExtractResult::Existed(path)) => path,
                    Ok(ExtractResult::Extracted(path)) => path,
                    Err(err) => {
                        let msg = format!("Unable to extract VBS: {}", err);
                        prompt(&msg.truecolor(255, 125, 0).to_string());
                        return;
                    }
                };
                match unify_line_endings_vbs_file(&vbs_path) {
                    Ok(NoChanges) => {
                        prompt("No changes applied as file has correct line endings");
                    }
                    Ok(Unified) => {
                        prompt(&format!(
                            "Unified line endings in VBS file at {}",
                            vbs_path.to_string_lossy()
                        ));
                    }
                    Err(err) => {
                        let msg = format!("Unable to patch VBS: {}", err);
                        prompt(&msg.truecolor(255, 125, 0).to_string());
                    }
                }
            }
            Some(TableOption::CreateVBSPatch) => {
                let vbs_path = selected_path.with_extension("vbs.original");
                let original_path = match extractvbs(selected_path, Some(vbs_path), true) {
                    Ok(ExtractResult::Existed(path)) => path,
                    Ok(ExtractResult::Extracted(path)) => path,
                    Err(err) => {
                        let msg = format!("Unable to extract VBS: {}", err);
                        prompt(&msg.truecolor(255, 125, 0).to_string());
                        return;
                    }
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
                        prompt(&msg.truecolor(255, 125, 0).to_string());
                    }
                }
            }
            Some(TableOption::InfoShow) => match info_gather(selected_path) {
                Ok(info) => {
                    prompt(&info);
                }
                Err(err) => {
                    let msg = format!("Unable to gather table info: {}", err);
                    prompt(&msg.truecolor(255, 125, 0).to_string());
                }
            },
            Some(TableOption::InfoEdit) => match info_edit(selected_path, Some(config)) {
                Ok(path) => {
                    println!("Launched editor for {}", path.display());
                }
                Err(err) => {
                    let msg = format!("Unable to edit table info: {}", err);
                    prompt_error(&msg);
                }
            },
            Some(TableOption::InfoDiff) => match info_diff(selected_path) {
                Ok(diff) => {
                    prompt(&diff);
                }
                Err(err) => {
                    let msg = format!("Unable to diff info: {}", err);
                    prompt_error(&msg);
                }
            },
            Some(TableOption::DIPSwitches) => {
                if info.requires_pinmame {
                    let nvram = nvram_for_rom(info);
                    if let Some(nvram) = nvram {
                        // open file in read/write mode
                        match edit_dip_switches(nvram) {
                            Ok(_) => {
                                // ok
                            }
                            Err(err) => {
                                let msg = format!("Unable to edit DIP switches: {}", err);
                                prompt_error(&msg);
                            }
                        }
                    } else {
                        prompt("This table does not have an NVRAM file, try launching it once.");
                    }
                } else {
                    prompt("This table is not using used PinMAME");
                }
            }
            Some(TableOption::NVRAMClear) => {
                clear_nvram(info);
            }
            Some(TableOption::B2SAutoPositionDMD) => match auto_position_dmd(config, &info) {
                Ok(msg) => {
                    prompt(&msg);
                }
                Err(err) => {
                    let msg = format!("Unable to auto-position DMD: {}", err);
                    prompt_error(&msg);
                }
            },
            Some(TableOption::EditINI) => {
                let path = ini_path_for(selected_path);
                let result = if path.exists() {
                    open_editor(&path, Some(config))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("File {} does not exist.", path.display()),
                    ))
                };
                match result {
                    Ok(_) => {
                        println!("Launched editor for {}", path.display());
                    }
                    Err(err) => {
                        let msg = format!("Unable to edit INI: {}", err);
                        prompt(&msg.truecolor(255, 125, 0).to_string());
                    }
                }
            }
            None => exit = true,
        }
    }
}

fn capture_windows(table_info: &IndexedTable) -> Result<i8, XCapError> {
    // Launch vpinball
    // Wait 12 seconds (make configurable)
    // Use xcap to find all windows that match the vpinball window titles
    // Make captures with xcap
    // Save captures to [tablefolder]/captures/[capturename].png
    // Exit vpinball

    // TODO launch vpinball

    // TODO we probably want a map that stores to what file the capture belongs
    let titles: HashSet<&str> = vec![
        "Visual Pinball Player",
        "Visual Pinball - Score",
        "PUPPlayfield",
        "PUPDMD",
        "PUPBackglass",
        "PUPFullDMD",
        "PUPTopper",
        "B2SBackglass",
        "B2SDMD",
        "PinMAME",
        "FlexDMD",
    ]
    .into_iter()
    .collect();

    let windows = xcap::Window::all()?;
    let mut captured = 0;
    for window in windows {
        if window.is_minimized()? {
            continue;
        }
        let title = window.title()?;
        if titles.contains(title.as_str()) {
            println!(
                "Window: {:?} {:?} {:?}",
                window.title()?,
                (window.x()?, window.y()?, window.width()?, window.height()?),
                (window.is_minimized()?, window.is_maximized()?)
            );
            let image = window.capture_image()?;
            let image_path = table_info
                .path
                .parent()
                .unwrap()
                .join("captures")
                .join(format!("test{}.png", captured));
            println!("Saving to {:?}", image_path);
            // drop the alpha channel to reduce file size
            let rgb_image: RgbImage = DynamicImage::from(image).to_rgb8();
            rgb_image
                .save(image_path)
                .map_err(|e| XCapError::new(format!("Unable to save image: {}", e)))?;
            captured += 1;
        }
    }
    Ok(captured)
}

fn auto_position_dmd(config: &ResolvedConfig, info: &&IndexedTable) -> Result<String, String> {
    match &info.b2s_path {
        Some(b2s_path) => {
            // TODO move image reading parsing code to vpin
            let reader =
                BufReader::new(File::open(b2s_path).map_err(|e| {
                    format!("Unable to open B2S file {}: {}", b2s_path.display(), e)
                })?);
            let b2s = vpin::directb2s::read(reader)
                .map_err(|e| format!("Unable to read B2S file: {}", e))?;

            if let Some(dmd_image) = b2s.images.dmd_image {
                // load vpinball config

                let ini_file = config.vpinball_ini_file();
                if ini_file.exists() {
                    let base64data_with_cr_lf = dmd_image.value;
                    let base64data = strip_cr_lf(&base64data_with_cr_lf);
                    let decoded_data = base64::engine::general_purpose::STANDARD
                        .decode(base64data)
                        .map_err(|e| format!("Unable to decode base64 data: {}", e))?;
                    // read the image with image crate
                    let image = image::load_from_memory(&decoded_data)
                        .map_err(|e| format!("Unable to read DMD image: {}", e))?;
                    let hole_opt = find_hole(&image, 6, &image.width() / 2, 5)
                        .map_err(|e| format!("Unable to find hole in DMD image: {}", e))?;
                    if let Some(hole) = hole_opt {
                        let table_ini_path = info.path.with_extension("ini");
                        let vpinball_config = VPinballConfig::read(&ini_file)
                            .map_err(|e| format!("Unable to read vpinball ini file: {}", e))?;
                        let mut table_config = if table_ini_path.exists() {
                            VPinballConfig::read(&table_ini_path)
                                .map_err(|e| format!("Unable to read table ini file: {}", e))
                        } else {
                            Ok(VPinballConfig::default())
                        }?;

                        let window_info = table_config
                            .get_window_info(WindowType::B2SDMD)
                            .or(vpinball_config.get_window_info(WindowType::B2SDMD));

                        if let Some(WindowInfo {
                            x: Some(x),
                            y: Some(y),
                            width: Some(width),
                            height: Some(height),
                            ..
                        }) = window_info
                        {
                            // Scale and position the hole to the vpinball FullDMD size.
                            // We might want to preserve the aspect ratio.
                            let hole = hole.scale_to_parent(width, height);

                            let dmd_x = x + hole.x();
                            let dmd_y = y + hole.y();
                            if hole.width() < 10 || hole.height() < 10 {
                                return Err(
                                    "Detected hole is too small, unable to update".to_string()
                                );
                            }
                            table_config.set_window_position(WindowType::PinMAME, dmd_x, dmd_y);
                            table_config.set_window_size(
                                WindowType::PinMAME,
                                hole.width(),
                                hole.height(),
                            );
                            table_config.set_window_position(WindowType::FlexDMD, dmd_x, dmd_y);
                            table_config.set_window_size(
                                WindowType::FlexDMD,
                                hole.width(),
                                hole.height(),
                            );

                            table_config.set_window_position(WindowType::DMD, dmd_x, dmd_y);
                            table_config.set_window_size(
                                WindowType::DMD,
                                hole.width(),
                                hole.height(),
                            );
                            table_config.write(&table_ini_path).unwrap();
                            Ok(format!(
                                "DMD window dimensions an position in {} updated to {}x{} at {},{}",
                                table_ini_path.file_name().unwrap().to_string_lossy(),
                                hole.width(),
                                hole.height(),
                                dmd_x,
                                dmd_y
                            ))
                        } else {
                            Err("Unable to find B2SDMD window or dimensions not specified in vpinball ini file".to_string())
                        }
                    } else {
                        Err("Unable to find hole in DMD image".to_string())
                    }
                } else {
                    Err("Unable to read vpinball ini file".to_string())
                }
            } else {
                Err("This table does not have a DMD image".to_string())
            }
        }
        None => Err("This table does not have a B2S file".to_string()),
    }
}

fn edit_dip_switches(nvram: PathBuf) -> io::Result<()> {
    let mut nvram_file = OpenOptions::new().read(true).write(true).open(nvram)?;
    let mut switches = get_all_dip_switches(&mut nvram_file)?;

    let items = switches
        .iter()
        .map(|s| format!("DIP #{}", s.nr))
        .collect::<Vec<String>>();

    let defaults = switches.iter().map(|s| s.on).collect::<Vec<bool>>();

    let help = "(<␣> selects, <⏎> saves, <esc/q> exits)"
        .dimmed()
        .to_string();
    let prompt_string = format!("Toggle switches {}", help);
    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt_string)
        .items(&items)
        .defaults(&defaults)
        .interact_opt()
        .unwrap();

    if let Some(selection) = selection {
        // update the switches
        switches.iter_mut().enumerate().for_each(|(i, s)| {
            s.on = selection.contains(&i);
        });

        set_dip_switches(&mut nvram_file, &switches)?;
        prompt("DIP switches updated");
    }
    Ok(())
}

fn clear_nvram(info: &IndexedTable) {
    if info.requires_pinmame {
        let nvram_file = nvram_for_rom(info);
        if let Some(nvram_file) = nvram_file {
            if nvram_file.exists() {
                match confirm(
                    "This will remove the table NVRAM file and you will lose all settings / high scores!".to_string(),
                    "Are you sure?".to_string(),
                ) {
                    Ok(true) => {
                        match std::fs::remove_file(&nvram_file) {
                            Ok(_) => {
                                prompt(&format!("NVRAM file {} removed", nvram_file.display()));
                            }
                            Err(err) => {
                                let msg = format!("Unable to remove NVRAM file: {}", err);
                                prompt(&msg.truecolor(255, 125, 0).to_string());
                            }
                        }
                    }
                    Ok(false) => {
                        prompt("NVRAM file removal canceled.");
                    }
                    Err(err) => {
                        let msg = format!("Error during confirmation: {}", err);
                        prompt(&msg.truecolor(255, 125, 0).to_string());
                    }
                }
            } else {
                prompt(&format!(
                    "NVRAM file {} does not exist",
                    nvram_file.display()
                ));
            }
        } else {
            prompt("This table does not have an NVRAM file");
        }
    } else {
        prompt("This table is not using used PinMAME");
    }
}

/// Find the NVRAM file for a ROM, not checking if it exists
fn nvram_for_rom(info: &IndexedTable) -> Option<PathBuf> {
    info.rom_path().as_ref().and_then(|rom_path| {
        // ../nvram/[romname].nv
        rom_path.parent().and_then(|p| p.parent()).and_then(|p| {
            rom_path
                .file_name()
                .map(|file_name| p.join("nvram").join(file_name).with_extension("nv"))
        })
    })
}

fn prompt(msg: &str) {
    Input::<String>::new()
        .with_prompt(format!("{} - Press enter to continue.", msg))
        .default("".to_string())
        .show_default(false)
        .interact()
        .unwrap();
}

fn prompt_error(msg: &str) {
    prompt(&msg.truecolor(255, 125, 0).to_string());
}

fn choose_table_option(table_name: &str, selected: Option<TableOption>) -> Option<TableOption> {
    let mut default = 0;
    // iterate over table options
    let selections = TableOption::ALL
        .iter()
        .enumerate()
        .map(|(index, option)| {
            if Some(option) == selected.as_ref() {
                default = index;
            }
            option.display()
        })
        .collect::<Vec<String>>();
    let selection_opt = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(table_name)
        .default(default)
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
                prompt(&format!(
                    "{} Visual Pinball exited with segfault, you might want to report this to the vpinball team.",
                    CRASH
                ));
            }
            Some(139) => {
                prompt(&format!(
                    "{} Visual Pinball exited with segfault, you might want to report this to the vpinball team.",
                    CRASH
                ));
            }
            Some(code) => {
                prompt(&format!(
                    "{} Visual Pinball exited with code {}",
                    CRASH, code
                ));
            }
            None => {
                prompt("Visual Pinball exited with unknown code");
            }
        },
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
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
                format!("({})", file_name).dimmed()
            )
        })
        .unwrap_or(file_name)
}

fn display_table_line_full(table: &IndexedTable) -> String {
    let base = display_table_line(table);
    let gamename_suffix = match &table.game_name {
        Some(name) => {
            let rom_found = table.rom_path().is_some();
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
