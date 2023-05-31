pub mod biff;
pub mod font;
pub mod gamedata;
pub mod image;
pub mod sound;
pub mod tableinfo;

use cfb::CompoundFile;
use clap::{arg, Arg, Command};
use colored::Colorize;
use gamedata::Record;
use serde_json::json;
use std::fmt::Debug;
use std::fs::{metadata, File};
use std::io::{self, Read};
use std::path::Path;
use std::process::exit;

use nom::{number::complete::le_u32, IResult};

use std::io::Write; // bring trait into scope

use git_version::git_version;

use crate::sound::write_sound;

// see https://github.com/fusion-engineering/rust-git-version/issues/21 for why the ""
const GIT_VERSION: &str = git_version!(args = ["--tags", "--always", "--dirty=-modified"]);

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
            Command::new("extract")
                .about("Extracts a vpx file")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true))
                .arg(
                    Arg::new("FORCE")
                        .short('f')
                        .long("force")
                        .num_args(0)
                        .help("Do not ask for confirmation before overwriting existing files"),
                ),
        )
        .subcommand(
            Command::new("extractvbs")
                .about("Extracts the vbs from a vpx file")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true))
                .arg(
                    Arg::new("FORCE")
                        .short('f')
                        .long("force")
                        .num_args(0)
                        .help("Do not ask for confirmation before overwriting existing files"),
                ),
        )
        .subcommand(
            Command::new("assemble")
                .about("Assembles a vpx file")
                .arg(arg!(<DIRPATH> "The path to the vpx structure").required(true)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("info", sub_matches)) => {
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = expand_path(path);
            println!("showing info for {}", expanded_path);
            let json = sub_matches.get_flag("JSON");
            info(expanded_path.as_ref(), json);
        }
        Some(("extract", sub_matches)) => {
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            // TODO expand all instead of only tilde?
            let expanded_path = expand_path(path);
            println!("extracting from {}", expanded_path);
            let yes = sub_matches.get_flag("FORCE");
            extract(expanded_path.as_ref(), yes);
        }
        Some(("extractvbs", sub_matches)) => {
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = expand_path(path);
            println!("extracting from {}", expanded_path);
            let yes = sub_matches.get_flag("FORCE");
            extractvbs(expanded_path.as_ref(), yes);
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn expand_path(path: &str) -> String {
    // TODO expand all instead of only tilde?
    let expanded_path = shellexpand::tilde(path);
    match metadata(path) {
        Ok(md) => {
            if !md.is_file() {
                println!("{} is not a file", expanded_path);
                exit(1);
            }
        }
        Err(msg) => {
            println!("Failed to open {}: {}", expanded_path, msg);
            exit(1);
        }
    }
    expanded_path.to_string()
}

fn info(vpx_file_path: &str, json: bool) {
    let mut comp = cfb::open(&vpx_file_path).unwrap();
    let version = read_version(&mut comp);
    // GameData also has a name field that we might want to display here
    // where is this shown in the UI?
    let table_info = tableinfo::read_tableinfo(&mut comp);
    // TODO come up with a proper format with colors and handle newlines?
    // TODO check the json flag
    dbg!(version);
    dbg!(table_info);
}

fn extract(vpx_file_path: &str, yes: bool) {
    let root_dir_path_str = vpx_file_path.replace(".vpx", "");
    let root_dir_path = Path::new(&root_dir_path_str);
    let mut root_dir = std::fs::DirBuilder::new();
    let vbs_path = root_dir_path.join("script.vbs").to_owned();

    let mut root_dir = std::fs::DirBuilder::new();
    root_dir.recursive(true);
    // ask for confirmation if the directory exists
    if root_dir_path.exists() && !yes {
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

    let mut comp = cfb::open(&vpx_file_path).unwrap();
    let version = read_version(&mut comp);
    let records = read_gamedata(&mut comp);

    extract_info(&mut comp, root_dir_path);
    extract_script(&records, &vbs_path);
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

fn extractvbs(vpx_file_path: &str, yes: bool) {
    let vbs_path_str = vpx_file_path.replace(".vpx", ".vbs");
    let vbs_path = Path::new(&vbs_path_str);

    if vbs_path.exists() && !yes {
        let warning = format!("File {} already exists", vbs_path.display()).truecolor(255, 125, 0);
        println!("{}", warning);
        println!("Do you want to continue exporting? (y/n)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim() != "y" {
            println!("Aborting");
            exit(1);
        }
    }

    let mut comp = cfb::open(&vpx_file_path).unwrap();
    let _version = read_version(&mut comp);
    let records = read_gamedata(&mut comp);
    extract_script(&records, &vbs_path);
}

fn extract_script<P: AsRef<Path>>(records: &Vec<Record>, vbs_path: &P) {
    let script = read_script(records);
    std::fs::write(vbs_path, script).unwrap();
    println!(
        "VBScript file written to\n  {}",
        vbs_path.as_ref().display()
    );
}

fn dump<T: Debug>(res: IResult<&[u8], T>) {
    match res {
        IResult::Ok((rest, value)) => {
            println!("Done {:?} {:?}...", value, rest)
        }
        IResult::Err(err) => {
            println!("Err {:?}", err)
        } // IResult::Incomplete(needed) => {println!("Needed {:?}",needed)}
    }
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
    println!("VPX file version: {}", version);
    version
}

fn read_script(records: &Vec<Record>) -> String {
    //dump(result);

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
    if table_info.screenshot.len() > 0 {
        let screenshot_path = root_dir_path.join("screenshot.bin");
        let mut screenshot_file = std::fs::File::create(screenshot_path).unwrap();
        screenshot_file.write_all(&table_info.screenshot).unwrap();
    }

    // TODO convert to a serde
    // TODO add free properties
    // TODO add missing data
    let info = json!({
        "name": table_info.table_name,
        "author": table_info.author_name,
    });

    serde_json::to_writer_pretty(&mut json_file, &info).unwrap();
    println!("Info file written to\n  {}", &json_path.display());
}

fn extract_images(comp: &mut CompoundFile<File>, records: &Vec<Record>, root_dir_path: &Path) {
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
    records: &Vec<Record>,
    root_dir_path: &Path,
    fileVersion: u32,
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
        let (_, sound) = sound::read(path.to_owned(), fileVersion, &input).unwrap();

        let ext = sound.ext();
        let mut sound_path = sounds_path.clone();
        sound_path.push(format!("Sound{}.{}.{}", index, sound.name, ext));
        //dbg!(&jpeg_path);
        let mut file = std::fs::File::create(sound_path).unwrap();
        file.write_all(&write_sound(&sound)).unwrap();
    }
}

fn extract_fonts(comp: &mut CompoundFile<File>, records: &Vec<Record>, root_dir_path: &Path) {
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
        let (_, font) = font::read(path.to_owned(), &input).unwrap();

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
            return path.to_owned();
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
