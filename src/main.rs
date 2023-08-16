pub mod directb2s;
mod frontend;
mod indexer;
pub mod jsonmodel;
pub mod vpx;

use clap::{arg, Arg, Command};
use colored::Colorize;
use console::Emoji;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{metadata, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::exit;

use std::io::Write;

use git_version::git_version;

use base64::{engine::general_purpose, Engine as _};

use directb2s::load;
use vpx::tableinfo::{self};
use vpx::{expanded, importvbs, verify, VerifyResult};
use vpx::{extractvbs, ExtractResult};

use crate::vpx::version;

// see https://github.com/fusion-engineering/rust-git-version/issues/21
const GIT_VERSION: &str = git_version!(args = ["--tags", "--always", "--dirty=-modified"]);

// TODO switch to figment for config
//   write the config if it doesn't exist
//   with empty values or defults?

const OK: Emoji = Emoji("✅", "[launch]");
const NOK: Emoji = Emoji("❌", "[crash]");

fn default_vpinball_executable() -> PathBuf {
    if cfg!(target_os = "windows") {
        // baller installer default
        let dir = PathBuf::from("c:\\vPinball\\VisualPinball");
        let exe = dir.join("VPinballX64.exe");
        if exe.exists() {
            exe
        } else {
            dir.join("VPinballX.exe")
        }
    } else {
        let home = dirs::home_dir().unwrap();
        home.join("vpinball").join("vpinball").join("VPinballX_GL")
    }
}

fn default_tables_root() -> PathBuf {
    if cfg!(target_os = "windows") {
        // baller installer default
        PathBuf::from("c:\\vPinball\\VisualPinball\\Tables")
    } else {
        let home = dirs::home_dir().unwrap();
        home.join("vpinball").join("tables")
    }
}

fn main() {
    // to allow for non static strings in clap
    // I had to enable the "string" module
    // is this considered a bad idea?
    let default_tables_root = default_tables_root().into_os_string();
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
                        .default_value(&default_tables_root)
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
                        .default_value(&default_tables_root)
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
                .about("Extracts the vbs from a vpx file next to it")
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
            Command::new("importvbs")
                .about("Imports the vbs next to it into a vpx file")
                .arg(
                    arg!(<VPXPATH> "The path(s) to the vpx file(s)")
                        .required(true)
                        .num_args(1..),
                ),
        )
        .subcommand(
            Command::new("verify")
                .about("Verify the structure of a vpx file")
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
        .subcommand(
            Command::new("new")
                .about("Creates a minimal empty new vpx file")
                .arg(arg!(<VPXPATH> "The path(s) to the vpx file").required(true)),
        )
        .get_matches_from(wild::args());

    match matches.subcommand() {
        Some(("info", sub_matches)) => {
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = expand_path(path);
            println!("showing info for {}", expanded_path);
            let json = sub_matches.get_flag("JSON");
            info(expanded_path.as_ref(), json).unwrap();
        }
        Some(("diff", sub_matches)) => {
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            let expanded_path = expand_path(path);
            match vpx::diff_script(PathBuf::from(expanded_path)) {
                Ok(output) => {
                    println!("{}", output);
                }
                Err(e) => {
                    let warning = format!("Error running diff: {}", e).red();
                    println!("{}", warning);
                    exit(1);
                }
            }
        }
        Some(("frontend", sub_matches)) => {
            let recursive = sub_matches.get_flag("RECURSIVE");
            let path = sub_matches
                .get_one::<String>("VPXROOTPATH")
                .map(|s| s.as_str());
            let tables_path = path.unwrap_or("");
            let expanded_tables_path = expand_path(tables_path);
            let vpx_files_with_tableinfo =
                frontend::frontend_index(expanded_tables_path, recursive);
            let vpinball_executable = default_vpinball_executable();
            frontend::frontend(vpx_files_with_tableinfo, &vpinball_executable);
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
                let expanded_path = PathBuf::from(expand_path(path));
                match extractvbs(&expanded_path, overwrite, None) {
                    ExtractResult::Existed(vbs_path) => {
                        let warning =
                            format!("EXISTED {}", vbs_path.display()).truecolor(255, 125, 0);
                        println!("{}", warning);
                    }
                    ExtractResult::Extracted(vbs_path) => {
                        println!("CREATED {}", vbs_path.display());
                    }
                }
            }
        }
        Some(("importvbs", sub_matches)) => {
            let path: &str = sub_matches.get_one::<String>("VPXPATH").unwrap().as_str();
            let expanded_path = PathBuf::from(expand_path(path));
            match importvbs(&expanded_path, None) {
                Ok(vbs_path) => {
                    println!("IMPORTED {}", vbs_path.display());
                }
                Err(e) => {
                    let warning = format!("Error importing vbs: {}", e).red();
                    println!("{}", warning);
                    exit(1);
                }
            }
        }

        Some(("verify", sub_matches)) => {
            let paths: Vec<&str> = sub_matches
                .get_many::<String>("VPXPATH")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();
            for path in paths {
                let expanded_path = PathBuf::from(expand_path(path));
                match verify(&expanded_path) {
                    VerifyResult::Ok(vbs_path) => {
                        println!("{OK} {}", vbs_path.display());
                    }
                    VerifyResult::Failed(vbs_path, msg) => {
                        let warning =
                            format!("{NOK} {} {}", vbs_path.display(), msg).truecolor(255, 125, 0);
                        println!("{}", warning);
                    }
                }
            }
        }
        Some(("new", sub_matches)) => {
            let path = {
                let this = sub_matches.get_one::<String>("VPXPATH").map(|v| v.as_str());
                match this {
                    Some(x) => x,
                    None => unreachable!("VPXPATH is required"),
                }
            };

            let expanded_path = shellexpand::tilde(path);
            println!("creating new vpx file at {}", expanded_path);
            new(expanded_path.as_ref()).unwrap();
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn new(vpx_file_path: &str) -> std::io::Result<()> {
    // TODO check if file exists and prompt to overwrite / add option to force
    vpx::new_minimal_vpx(vpx_file_path)
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

fn info(vpx_file_path: &str, json: bool) -> io::Result<()> {
    let mut comp = cfb::open(vpx_file_path)?;
    let version = version::read_version(&mut comp)?;
    // GameData also has a name field that we might want to display here
    // where is this shown in the UI?
    let table_info = tableinfo::read_tableinfo(&mut comp)?;
    // TODO check the json flag

    println!("{:>18} {}", "VPX Version:".green(), version);
    println!("{:>18} {}", "Table Name:".green(), table_info.table_name);
    println!("{:>18} {}", "Version:".green(), table_info.table_version);
    println!(
        "{:>18} {}{}{}",
        "Author:".green(),
        Some(table_info.author_name)
            .map(|s| s.unwrap_or("[not set]".to_string()))
            .filter(|s| !s.is_empty())
            .map(|s| format!("{} ", s))
            .unwrap_or_default(),
        Some(table_info.author_email)
            .map(|s| s.unwrap_or("[not set]".to_string()))
            .filter(|s| !s.is_empty())
            .map(|s| format!("{} ", s))
            .unwrap_or_default(),
        Some(table_info.author_website)
            .map(|s| s.unwrap_or("[not set]".to_string()))
            .filter(|s| !s.is_empty())
            .map(|s| format!("{} ", s))
            .unwrap_or_default(),
    );
    println!(
        "{:>18} {}",
        "Save revision:".green(),
        table_info.table_save_rev.unwrap_or("[not set]".to_string())
    );
    println!(
        "{:>18} {}",
        "Save date:".green(),
        table_info
            .table_save_date
            .unwrap_or("[not set]".to_string())
    );
    println!(
        "{:>18} {}",
        "Release Date:".green(),
        table_info.release_date.unwrap_or("[not set]".to_string())
    );
    println!(
        "{:>18} {}",
        "Description:".green(),
        table_info.table_description.unwrap_or("[not set]".to_string())
    );
    println!(
        "{:>18} {}",
        "Blurb:".green(),
        table_info.table_blurb.unwrap_or("[not set]".to_string())
    );
    println!(
        "{:>18} {}",
        "Rules:".green(),
        table_info.table_rules.unwrap_or("[not set]".to_string())
    );
    // other properties
    table_info.properties.iter().for_each(|(prop, value)| {
        println!("{:>18}: {}", prop.green(), value);
    });

    Ok(())
}

pub fn extract(vpx_file_path: &Path, yes: bool) {
    let root_dir_path_str = vpx_file_path.with_extension("");
    let root_dir_path = Path::new(&root_dir_path_str);

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
            exit(0);
        }
    }
    match expanded::extract(vpx_file_path, root_dir_path) {
        Ok(_) => {
            println!("Successfully extracted to {}", root_dir_path.display());
        }
        Err(e) => {
            println!("Failed to extract: {}", e);
            exit(1);
        }
    }
}
