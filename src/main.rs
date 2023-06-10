pub mod biff;
pub mod directb2s;
pub mod font;
pub mod gamedata;
pub mod image;
pub mod indexer;
pub mod jsonmodel;
pub mod sound;
pub mod tableinfo;

use cfb::CompoundFile;
use clap::{arg, Arg, Command};
use colored::Colorize;
use console::Emoji;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use gamedata::Record;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{metadata, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{exit, ExitStatus};

use nom::{number::complete::le_u32, IResult};

use std::io::Write; // bring trait into scope

use git_version::git_version;

use base64::{engine::general_purpose, Engine as _};

use crate::directb2s::load;
use crate::jsonmodel::table_json;
use crate::sound::write_sound;

// see https://github.com/fusion-engineering/rust-git-version/issues/21
const GIT_VERSION: &str = git_version!(args = ["--tags", "--always", "--dirty=-modified"]);

const DEFAULT_VPINBALL_ROOT: &str = "~/vpinball";
const DEFAULT_TABLES_ROOT: &str = "~/vpinball/tables";

fn main() {
    let matches = Command::new("vpxtool")
        .version(GIT_VERSION)
        .author("Francis DB")
        .about("Extracts and assembles vpx files")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("info")
                .about("Show information about a vpx file")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true))
                .arg(
                    Arg::new("JSON")
                        .short('j')
                        .long("json")
                        .num_args(0)
                        .help("Output as JSON"),
                ),
        )
        .subcommand(
            Command::new("diff")
                .about("Prints out a diff between the vbs in the vpx and the sidecar vbs")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true))
        )
        .subcommand(
            Command::new("frontend")
                .about("Acts as a frontend for launching vpx files")
                .arg(
                    Arg::new("RECURSIVE")
                        .short('r')
                        .long("recursive")
                        .num_args(0)
                        .help("Recursively index subdirectories")
                        .default_value("true"),
                )
                .arg(
                    arg!(<VPXROOTPATH> "The path to the root directory of vpx files")
                        .required(false)
                        .default_value(DEFAULT_TABLES_ROOT)
                ),
        )
        .subcommand(
            Command::new("index")
                .about("Indexes a directory of vpx files")
                .arg(
                    Arg::new("RECURSIVE")
                        .short('r')
                        .long("recursive")
                        .num_args(0)
                        .help("Recursively index subdirectories")
                        .default_value("true"),
                )
                .arg(
                    arg!(<VPXROOTPATH> "The path to the root directory of vpx files")
                        .required(false)
                        .default_value(DEFAULT_TABLES_ROOT)
                ),
        )
        .subcommand(
            Command::new("extract")
                .about("Extracts a vpx file")
                .arg(
                    Arg::new("FORCE")
                        .short('f')
                        .long("force")
                        .num_args(0)
                        .help("Do not ask for confirmation before overwriting existing files"),
                )
                .arg(
                    arg!(<VPXPATH> "The path(s) to the vpx file(s)")
                        .required(true)
                        .num_args(1..),
                ),
        )
        .subcommand(
            Command::new("extractvbs")
                .about("Extracts the vbs from a vpx file")
                .arg(
                    Arg::new("OVERWRITE")
                        .short('o')
                        .long("overwrite")
                        .num_args(0)
                        .default_value("false")
                        .help("(Default: false) Will overwrite existing .vbs files if true, will skip the table file if false."),
                )
                .arg(
                    arg!(<VPXPATH> "The path(s) to the vpx file(s)")
                        .required(true)
                        .num_args(1..),
                ),
        )
        .subcommand(
            Command::new("assemble")
                .about("Assembles a vpx file")
                .arg(arg!(<DIRPATH> "The path to the vpx structure").required(true)),
        )
        .get_matches_from(wild::args());

    match matches.subcommand() {
        Some(("info", sub_matches)) => {
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = expand_path(path);
            println!("showing info for {}", expanded_path);
            let json = sub_matches.get_flag("JSON");
            info(expanded_path.as_ref(), json);
        }
        Some(("diff", sub_matches)) => {
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = expand_path(path);
            diff(expanded_path.as_ref());
        }
        Some(("frontend", sub_matches)) => {
            let recursive = sub_matches.get_flag("RECURSIVE");
            let path = sub_matches
                .get_one::<String>("VPXROOTPATH")
                .map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = expand_path(path);
            println!("Indexing {}", expanded_path);
            let vpx_files = indexer::find_vpx_files(recursive, expanded_path.as_ref());
            let pb = ProgressBar::new(vpx_files.len() as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "{spinner:.green} [{bar:.cyan/blue}] {pos}/{human_len} ({eta})",
                )
                .unwrap(),
            );
            let mut vpx_files_with_tableinfo = indexer::index_vpx_files(&vpx_files, |pos: u64| {
                pb.set_position(pos);
            });
            pb.finish_and_clear();

            // TODO this is a second sort, does not make a lot of sense to do the first one
            vpx_files_with_tableinfo.sort_by_key(|(path1, info1)| display_table_line(path1, info1));

            loop {
                let selections = vpx_files_with_tableinfo
                    .iter()
                    // TODO can we expand the tuple to args?
                    .map(|(path, info)| display_table_line(path, info))
                    .collect::<Vec<String>>();

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a table to launch")
                    .default(0)
                    .items(&selections[..])
                    .interact()
                    .unwrap();

                let (selected_path, _selected_info) =
                    vpx_files_with_tableinfo.get(selection).unwrap();

                let launch = Emoji("🚚 ", "[launch]");
                let crash = Emoji("💥 ", "[crash]");

                println!("{} {}", launch, selected_path.display());
                match launch_table(selected_path) {
                    Ok(status) => match status.code() {
                        Some(0) => {
                            //println!("Table exited normally");
                        }
                        Some(11) => {
                            println!("{} Table exited with segfault, you might want to report this to the vpinball team.", crash);
                        }
                        Some(139) => {
                            println!("{} Table exited with segfault, you might want to report this to the vpinball team.", crash);
                        }
                        Some(code) => {
                            println!("Table exited with code {}", code);
                        }
                        None => {
                            println!("Table exited with unknown code");
                        }
                    },
                    Err(e) => {
                        println!("Error launching table: {:?}", e);
                    }
                }
            }
        }
        Some(("index", sub_matches)) => {
            let recursive = sub_matches.get_flag("RECURSIVE");
            let path = sub_matches
                .get_one::<String>("VPXROOTPATH")
                .map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = expand_path(path);
            println!("Indexing {}", expanded_path);
            let vpx_files = indexer::find_vpx_files(recursive, expanded_path.as_ref());
            let pb = ProgressBar::new(vpx_files.len() as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "{spinner:.green} [{bar:.cyan/blue}] {pos}/{human_len} ({eta})",
                )
                .unwrap(),
            );
            let vpx_files_with_tableinfo = indexer::index_vpx_files(&vpx_files, |pos: u64| {
                pb.set_position(pos);
            });
            pb.finish_and_clear();
            let json_path = Path::new(&expanded_path).join("index.json");
            indexer::write_index_json(vpx_files_with_tableinfo, json_path.clone());
            println!(
                "Indexed {} vpx files into {}",
                vpx_files.len(),
                &json_path.display()
            );
        }
        Some(("extract", sub_matches)) => {
            let yes = sub_matches.get_flag("FORCE");
            let paths: Vec<&str> = sub_matches
                .get_many::<String>("VPXPATH")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect();
            for path in paths {
                let expanded_path = expand_path(path);
                println!("extracting from {}", expanded_path);
                if expanded_path.ends_with(".directb2s") {
                    extract_directb2s(&expanded_path);
                } else {
                    extract(expanded_path.as_ref(), yes);
                }
            }
        }
        Some(("extractvbs", sub_matches)) => {
            let overwrite = sub_matches.get_flag("OVERWRITE");
            let paths: Vec<&str> = sub_matches
                .get_many::<String>("VPXPATH")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();
            for path in paths {
                let expanded_path = expand_path(path);
                extractvbs(expanded_path.as_ref(), overwrite);
            }
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn launch_table(selected_path: &PathBuf) -> io::Result<ExitStatus> {
    let executable = Path::new(&expand_path(DEFAULT_VPINBALL_ROOT))
        .join("vpinball")
        .join("VPinballX_GL");

    // start process ./VPinballX_GL -play [table path]
    let mut cmd = std::process::Command::new(executable);
    cmd.arg("-play");
    cmd.arg(selected_path);
    let mut child = cmd.spawn()?;
    let result = child.wait()?;
    Ok(result)
}

fn display_table_line(path: &Path, info: &tableinfo::TableInfo) -> String {
    let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
    Some(info.table_name.to_owned())
        .filter(|s| !s.is_empty())
        .map(|s| format!("{} {}", s, (format!("({})", file_name)).dimmed()))
        .unwrap_or(file_name)
}

fn extract_directb2s(expanded_path: &String) {
    let mut file = File::open(expanded_path).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    match load(&text) {
        Ok(b2s) => {
            println!("DirectB2S file version {}", b2s.version);
            let root_dir_path_str = expanded_path.replace(".directb2s", ".directb2s.extracted");

            let root_dir_path = Path::new(&root_dir_path_str);
            let mut root_dir = std::fs::DirBuilder::new();
            root_dir.recursive(true);
            root_dir.create(root_dir_path).unwrap();

            println!("Writing to {}", root_dir_path_str);
            wite_images(b2s, root_dir_path);
        }
        Err(msg) => {
            println!("Failed to load {}: {}", expanded_path, msg);
            exit(1);
        }
    }
}

fn wite_images(b2s: directb2s::DirectB2SData, root_dir_path: &Path) {
    if let Some(backglass_off_image) = b2s.images.backglass_off_image {
        write_base64_to_file(
            root_dir_path,
            None,
            "backglassimage.img".to_string(),
            &backglass_off_image.value,
        );
    }
    if let Some(backglass_on_image) = b2s.images.backglass_on_image {
        write_base64_to_file(
            root_dir_path,
            Some(backglass_on_image.file_name),
            "backglassimage.img".to_string(),
            &backglass_on_image.value,
        );
    }
    if let Some(backglass_image) = b2s.images.backglass_image {
        write_base64_to_file(
            root_dir_path,
            Some(backglass_image.file_name),
            "backglassimage.img".to_string(),
            &backglass_image.value,
        );
    }

    if let Some(dmd_image) = b2s.images.dmd_image {
        write_base64_to_file(
            root_dir_path,
            Some(dmd_image.file_name),
            "dmdimage.img".to_string(),
            &dmd_image.value,
        );
    }
    if let Some(illumination_image) = b2s.images.illumination_image {
        write_base64_to_file(
            root_dir_path,
            None,
            "dmdimage.img".to_string(),
            &illumination_image.value,
        );
    }

    let thumbnail_image = b2s.images.thumbnail_image;
    write_base64_to_file(
        root_dir_path,
        None,
        "thumbnailimage.png".to_string(),
        &thumbnail_image.value,
    );

    for bulb in b2s.illumination.bulb.unwrap_or_default() {
        write_base64_to_file(
            root_dir_path,
            None,
            format!("{}.png", bulb.name).to_string(),
            &bulb.image,
        );
        if let Some(off_image) = bulb.off_image {
            write_base64_to_file(
                root_dir_path,
                None,
                format!("{}_off.png", bulb.name).to_string(),
                &off_image,
            );
        }
    }

    if let Some(reel) = b2s.reels {
        for reels_image in reel.images.image.iter().flatten() {
            write_base64_to_file(
                root_dir_path,
                None,
                format!("{}.png", reels_image.name).to_string(),
                &reels_image.image,
            );
        }
        for illuminated_set in reel.illuminated_images.set.iter().flatten() {
            for reels_image in &illuminated_set.illuminated_image {
                write_base64_to_file(
                    root_dir_path,
                    None,
                    format!("{}.png", reels_image.name).to_string(),
                    &reels_image.image,
                );
            }
        }
    }
}

fn write_base64_to_file(
    root_dir_path: &Path,
    original_file_path: Option<String>,
    default: String,
    base64data_with_cr_lf: &str,
) {
    // TODO bring in the other default here
    let file_name: String =
        os_independent_file_name(original_file_path.unwrap_or(default.clone())).unwrap_or(default);

    let file_path = root_dir_path.join(file_name);

    let mut file = File::create(file_path).unwrap();
    let base64data = strip_cr_lf(base64data_with_cr_lf);

    let decoded_data = general_purpose::STANDARD.decode(base64data).unwrap();
    file.write_all(&decoded_data).unwrap();
}

fn strip_cr_lf(s: &str) -> String {
    s.chars().filter(|c| !c.is_ascii_whitespace()).collect()
}

fn os_independent_file_name(file_path: String) -> Option<String> {
    // we can't use path here as this uses the system path encoding
    // we might have to parse windows paths on mac/linux
    file_path
        .rsplit(|c| c == '/' || c == '\\')
        .next()
        .map(|f| f.to_string())
}

fn expand_path(path: &str) -> String {
    // TODO expand all instead of only tilde?
    let expanded_path = shellexpand::tilde(path);
    match metadata(expanded_path.as_ref()) {
        Ok(md) => {
            if !md.is_file() && !md.is_dir() && md.is_symlink() {
                println!("{} is not a file", expanded_path);
                exit(1);
            }
        }
        Err(msg) => {
            let warning = format!("Failed to read metadata for {}: {}", expanded_path, msg).red();
            println!("{}", warning);
            exit(1);
        }
    }
    expanded_path.to_string()
}

fn info(vpx_file_path: &str, json: bool) {
    let mut comp = cfb::open(vpx_file_path).unwrap();
    let version = read_version(&mut comp);
    // GameData also has a name field that we might want to display here
    // where is this shown in the UI?
    let table_info = tableinfo::read_tableinfo(&mut comp);
    // TODO come up with a proper format with colors and handle newlines?
    // TODO check the json flag
    dbg!(version);
    dbg!(table_info);
}

fn diff(vpx_file_path: &str) {
    let vbs_path_str = vpx_file_path.replace(".vpx", ".vbs");
    let vbs_path = Path::new(&vbs_path_str);
    let original_vbs_path_str = vpx_file_path.replace(".vpx", ".vbs.original.tmp");
    let original_vbs_path = Path::new(&original_vbs_path_str);

    if vbs_path.exists() {
        match cfb::open(vpx_file_path) {
            Ok(mut comp) => {
                let records = read_gamedata(&mut comp);
                let script = find_script(&records);
                std::fs::write(original_vbs_path, script).unwrap();

                match run_diff(original_vbs_path, vbs_path) {
                    Ok(output) => {
                        println!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                    Err(e) => {
                        println!("Error running diff: {}", e);
                    }
                }

                if original_vbs_path.exists() {
                    std::fs::remove_file(original_vbs_path).unwrap();
                }
            }
            Err(e) => {
                let warning = format!("Not a valid vpx file {}: {}", vpx_file_path, e).red();
                println!("{}", warning);
                exit(1)
            }
        }
    } else {
        let warning = format!("No sidecar vbs file found: {}", vbs_path.display()).red();
        println!("{}", warning);
        exit(1)
    }
}

fn run_diff(original_vbs_path: &Path, vbs_path: &Path) -> Result<std::process::Output, io::Error> {
    std::process::Command::new("diff")
        .arg("-u")
        .arg("-w")
        .arg("--color=always")
        .arg(original_vbs_path)
        .arg(vbs_path)
        .output()
}

fn extract(vpx_file_path: &str, yes: bool) {
    let root_dir_path_str = vpx_file_path.replace(".vpx", "");
    let root_dir_path = Path::new(&root_dir_path_str);
    let vbs_path = root_dir_path.join("script.vbs");

    let mut root_dir = std::fs::DirBuilder::new();
    root_dir.recursive(true);
    // ask for confirmation if the directory exists
    if root_dir_path.exists() && !yes {
        // TODO do we need to check for terminal here?
        //   let use_color = stdout().is_terminal();
        let warning =
            format!("Directory {} already exists", root_dir_path.display()).truecolor(255, 125, 0);
        println!("{}", warning);
        println!("Do you want to continue exporting? (y/n)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim() != "y" {
            println!("Aborting");
            exit(1);
        }
    }
    root_dir.create(root_dir_path).unwrap();

    let mut comp = cfb::open(vpx_file_path).unwrap();
    let version = read_version(&mut comp);
    let records = read_gamedata(&mut comp);

    extract_info(&mut comp, root_dir_path);
    extract_script(&records, &vbs_path);
    println!("VBScript file written to\n  {}", &vbs_path.display());
    extract_binaries(&mut comp, root_dir_path);
    extract_images(&mut comp, &records, root_dir_path);
    extract_sounds(&mut comp, &records, root_dir_path, version);
    extract_fonts(&mut comp, &records, root_dir_path);

    // let mut file_version = String::new();
    // comp.open_stream("/GameStg/Version")
    //     .unwrap()
    //     .read_to_string(&mut file_version)
    //     .unwrap();
    // println!("File version: {}", file_version);

    // let mut stream = comp.open_stream(inner_path).unwrap();
    // io::copy(&mut stream, &mut io::stdout()).unwrap();
}

fn extractvbs(vpx_file_path: &str, overwrite: bool) {
    let vbs_path_str = vpx_file_path.replace(".vpx", ".vbs");
    let vbs_path = Path::new(&vbs_path_str);

    if !vbs_path.exists() || (vbs_path.exists() && overwrite) {
        let mut comp = cfb::open(vpx_file_path).unwrap();
        let _version = read_version(&mut comp);
        let records = read_gamedata(&mut comp);
        extract_script(&records, &vbs_path);
        println!("CREATED {}", vbs_path.display());
    } else {
        let warning = format!("EXISTED {}", vbs_path.display()).truecolor(255, 125, 0);
        println!("{}", warning);
    }
}

fn extract_script<P: AsRef<Path>>(records: &[Record], vbs_path: &P) {
    let script = find_script(records);
    std::fs::write(vbs_path, script).unwrap();
}

// TODO: read version
//  https://github.com/vbousquet/vpx_lightmapper/blob/331a8576bb7b86668a023b304e7dd04261487106/addons/vpx_lightmapper/vlm_import.py#L328
fn read_version(comp: &mut cfb::CompoundFile<std::fs::File>) -> u32 {
    let mut file_version = Vec::new();
    comp.open_stream("/GameStg/Version")
        .unwrap()
        .read_to_end(&mut file_version)
        .unwrap();

    fn read_version(input: &[u8]) -> IResult<&[u8], u32> {
        le_u32(input)
    }

    // use lut to read as u32
    let (_, version) = read_version(&file_version[..]).unwrap();

    // let version_float = (version as f32)/100f32;
    // println!("VPX file version: {}", version);
    version
}

fn find_script(records: &[Record]) -> String {
    let code = records
        .iter()
        .find_map(|r| match r {
            Record::Code { script } => Some(script),
            _ => None,
        })
        .unwrap();

    code.to_owned()
}

fn read_gamedata(comp: &mut CompoundFile<File>) -> Vec<Record> {
    let mut game_data_vec = Vec::new();
    comp.open_stream("/GameStg/GameData")
        .unwrap()
        .read_to_end(&mut game_data_vec)
        .unwrap();

    // let result = parseGameData(&game_data_vec[..]);
    // dump(result);

    let (_, records) = gamedata::read_all_gamedata_records(&game_data_vec[..]).unwrap();
    records
}

fn extract_info(comp: &mut CompoundFile<File>, root_dir_path: &Path) {
    let json_path = root_dir_path.join("TableInfo.json");
    let mut json_file = std::fs::File::create(&json_path).unwrap();
    let table_info = tableinfo::read_tableinfo(comp);
    if !table_info.screenshot.is_empty() {
        let screenshot_path = root_dir_path.join("screenshot.bin");
        let mut screenshot_file = std::fs::File::create(screenshot_path).unwrap();
        screenshot_file.write_all(&table_info.screenshot).unwrap();
    }

    let info = table_json(&table_info);

    serde_json::to_writer_pretty(&mut json_file, &info).unwrap();
    println!("Info file written to\n  {}", &json_path.display());
}

fn extract_images(comp: &mut CompoundFile<File>, records: &[Record], root_dir_path: &Path) {
    // let result = parseGameData(&game_data_vec[..]);
    // dump(result);

    let images_size = records
        .iter()
        .find_map(|r| match r {
            Record::ImagesSize(size) => Some(size),
            _ => None,
        })
        .unwrap_or(&0)
        .to_owned();

    let images_path = root_dir_path.join("images");
    std::fs::create_dir_all(&images_path).unwrap();

    println!(
        "Writing {} images to\n  {}",
        images_size,
        images_path.display()
    );

    for index in 0..images_size {
        let path = format!("GameStg/Image{}", index);
        let mut input = Vec::new();
        comp.open_stream(&path)
            .unwrap()
            .read_to_end(&mut input)
            .unwrap();
        let (_, img) = image::read(path.to_owned(), &input).unwrap();
        match &img.jpeg {
            Some(jpeg) => {
                let ext = img.ext();
                let mut jpeg_path = images_path.clone();
                jpeg_path.push(format!("Image{}.{}.{}", index, img.name, ext));
                //dbg!(&jpeg_path);
                let mut file = std::fs::File::create(jpeg_path).unwrap();
                file.write_all(&jpeg.data).unwrap();
            }
            None => {
                println!("Image {} has no jpeg data", index)
                // nothing to do here
            }
        }
    }
}

fn extract_sounds(
    comp: &mut CompoundFile<File>,
    records: &[Record],
    root_dir_path: &Path,
    file_version: u32,
) {
    // let result = parseGameData(&game_data_vec[..]);
    // dump(result);

    let sounds_size = records
        .iter()
        .find_map(|r| match r {
            Record::SoundsSize(size) => Some(size),
            _ => None,
        })
        .unwrap_or(&0)
        .to_owned();

    let sounds_path = root_dir_path.join("sounds");
    std::fs::create_dir_all(&sounds_path).unwrap();

    println!(
        "Writing {} sounds to\n  {}",
        sounds_size,
        sounds_path.display()
    );

    for index in 0..sounds_size {
        let path = format!("GameStg/Sound{}", index);
        let mut input = Vec::new();
        comp.open_stream(&path)
            .unwrap()
            .read_to_end(&mut input)
            .unwrap();
        let (_, sound) = sound::read(path.to_owned(), file_version, &input).unwrap();

        let ext = sound.ext();
        let mut sound_path = sounds_path.clone();
        sound_path.push(format!("Sound{}.{}.{}", index, sound.name, ext));
        //dbg!(&jpeg_path);
        let mut file = std::fs::File::create(sound_path).unwrap();
        file.write_all(&write_sound(&sound)).unwrap();
    }
}

fn extract_fonts(comp: &mut CompoundFile<File>, records: &[Record], root_dir_path: &Path) {
    // let result = parseGameData(&game_data_vec[..]);
    // dump(result);

    let fonts_size = records
        .iter()
        .find_map(|r| match r {
            Record::FontsSize(size) => Some(size),
            _ => None,
        })
        .unwrap_or(&0)
        .to_owned();

    let fonts_path = root_dir_path.join("fonts");
    std::fs::create_dir_all(&fonts_path).unwrap();

    println!(
        "Writing {} fonts to\n  {}",
        fonts_size,
        fonts_path.display()
    );

    for index in 0..fonts_size {
        let path = format!("GameStg/Font{}", index);
        let mut input = Vec::new();
        comp.open_stream(&path)
            .unwrap()
            .read_to_end(&mut input)
            .unwrap();
        let (_, font) = font::read(&input).unwrap();

        let ext = font.ext();
        let mut font_path = fonts_path.clone();
        font_path.push(format!("Font{}.{}.{}", index, font.name, ext));
        //dbg!(&jpeg_path);
        let mut file = std::fs::File::create(font_path).unwrap();
        file.write_all(&font.data).unwrap();
    }
}

fn extract_binaries(comp: &mut CompoundFile<std::fs::File>, root_dir_path: &Path) {
    // write all remaining entries
    let entries: Vec<String> = comp
        .walk()
        .filter(|entry| {
            entry.is_stream()
                && !entry.path().starts_with("/TableInfo")
                && !entry.path().starts_with("/GameStg/GameData")
        })
        .map(|entry| {
            let path = entry.path();
            let path = path.to_str().unwrap();
            //println!("{} {} {}", path, entry.is_stream(), entry.len());
            path.to_owned()
        })
        .collect();

    entries.iter().for_each(|path| {
        let mut stream = comp.open_stream(path).unwrap();
        // write the steam directly to a file
        let file_path = root_dir_path.join(&path[1..]);
        // println!("Writing to {}", file_path.display());
        // make sure the parent directory exists
        let parent = file_path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        let mut file = std::fs::File::create(file_path).unwrap();
        io::copy(&mut stream, &mut file).unwrap();
    });

    println!("Binaries written to\n  {}", root_dir_path.display());
}
