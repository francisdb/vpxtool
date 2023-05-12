pub mod gamedata;
pub mod tableinfo;

use cfb::CompoundFile;
use clap::{arg, Arg, Command};
use colored::Colorize;
use gamedata::Record;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::process::exit;
use std::str::{self, from_utf8};

use nom::{number::complete::le_u32, IResult};

fn main() {
    let matches = Command::new("vpxtool")
        .version("0.1")
        .author("Francis DB")
        .about("Extracts and assembles vpx files")
        .arg_required_else_help(true)
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
        Some(("extract", sub_matches)) => {
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            // TODO expand all instead of only tilde?
            let expanded_path = shellexpand::tilde(path);
            println!("extracting from {}", expanded_path);
            let yes = sub_matches.get_flag("FORCE");
            extract(expanded_path.as_ref(), yes);
        }
        Some(("extractvbs", sub_matches)) => {
            let path = sub_matches.get_one::<String>("VPXPATH").map(|s| s.as_str());
            let path = path.unwrap_or("");
            // TODO expand all instead of only tilde?
            let expanded_path = shellexpand::tilde(path);
            println!("extracting from {}", expanded_path);
            let yes = sub_matches.get_flag("FORCE");
            extractvbs(expanded_path.as_ref(), yes);
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn extract(vpx_file_path: &str, yes: bool) {
    let mut comp = cfb::open(&vpx_file_path).unwrap();
    let root_dir_path_str = vpx_file_path.replace(".vpx", "");
    let root_dir_path = Path::new(&root_dir_path_str);

    let json_path = root_dir_path.join("TableInfo.json");
    let vbs_path_str = vpx_file_path.replace(".vpx", ".vbs");
    let vbs_path = Path::new(&vbs_path_str);

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

    extract_info(&mut comp, &json_path);
    extract_script(&mut comp, vbs_path);
    extract_binaries(comp, root_dir_path);

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
    extract_script(&mut comp, vbs_path);
}

fn extract_script(comp: &mut cfb::CompoundFile<std::fs::File>, vbs_path: &Path) {
    let version = read_version(comp);
    let script = read_script(comp);
    std::fs::write(vbs_path, script).unwrap();
    println!("VBScript file written to\n  {}", vbs_path.display());
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

    fn read_version (input: &[u8]) -> IResult<&[u8], u32> {
        le_u32(input)
    }

    // use lut to read as u32
    let (_, version) = read_version(&file_version[..]).unwrap();

    // let version_float = (version as f32)/100f32;
    println!("VPX file version: {}", version);
    version
}



fn read_script(comp: &mut cfb::CompoundFile<std::fs::File>) -> String {
    let mut game_data_vec = Vec::new();
    comp.open_stream("/GameStg/GameData")
        .unwrap()
        .read_to_end(&mut game_data_vec)
        .unwrap();

    // let result = parseGameData(&game_data_vec[..]);
    // dump(result);

    let (_, records) = gamedata::read_all_records(&game_data_vec[..]).unwrap();

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

fn extract_info<P: AsRef<Path>>(comp: &mut CompoundFile<File>, json_path: &P) {
    let mut json_file = std::fs::File::create(json_path).unwrap();
    let json_root = tableinfo::read_tableinfo(comp);
    serde_json::to_writer_pretty(&mut json_file, &json_root).unwrap();
    println!("Info file written to\n  {}", json_path.as_ref().display());
}

fn extract_binaries(mut comp: cfb::CompoundFile<std::fs::File>, root_dir_path: &Path) {
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
