pub mod biff;
pub mod gamedata;

use clap::{arg, Arg, Command};
use colored::Colorize;
use nom::bytes::streaming::take;
use nom::multi::many0;
use nom::number::complete::le_f32;
use serde_json::json;
use std::fmt::Debug;
use std::io::{self, Read};
use std::path::Path;
use std::process::exit;
use std::str::{self, from_utf8};

use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    number::complete::le_u32,
    sequence::tuple,
    IResult,
};

use biff::biff_to_utf8;

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
            println!("extracting {}", expanded_path);
            let yes = sub_matches.get_flag("FORCE");
            extract(expanded_path.as_ref(), yes);
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn extract(vpx_file_path: &str, yes: bool) {
    // let (comp_path, inner_path) = expanded_path.split_once('/').unwrap();
    let mut comp = cfb::open(&vpx_file_path).unwrap();

    // println!("{}", serde_json::to_string_pretty(&root).unwrap());

    // TODO write the json to a file with the same name as the vpx file
    // make root dir if missing
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

    println!("Info file written to\n  {}", json_path.display());

    extract_script(&mut comp);

    println!("VBScript file written to\n  {}", vbs_path.display());

    extract_binaries(comp, root_dir_path);

    println!("Binaries written to\n  {}", root_dir_path.display());

    // let mut file_version = String::new();
    // comp.open_stream("/GameStg/Version")
    //     .unwrap()
    //     .read_to_string(&mut file_version)
    //     .unwrap();
    // println!("{}", file_version);

    // let mut stream = comp.open_stream(inner_path).unwrap();
    // io::copy(&mut stream, &mut io::stdout()).unwrap();
}

fn dump<T: Debug>(res: IResult<&[u8], T>) {
    match res {
        IResult::Ok((rest, value)) => {
            println!("Done {:?} {:?}...", value, &rest[..8])
        }
        IResult::Err(err) => {
            println!("Err {:?}", err)
        } // IResult::Incomplete(needed) => {println!("Needed {:?}",needed)}
    }
}

fn extract_script(comp: &mut cfb::CompoundFile<std::fs::File>) {
    let mut game_data_vec = Vec::new();
    comp.open_stream("/GameStg/GameData")
        .unwrap()
        .read_to_end(&mut game_data_vec)
        .unwrap();

    // let result = parseGameData(&game_data_vec[..]);
    // dump(result);

    let result = gamedata::parse_all(&game_data_vec[..]);
    dump(result);

    let mut buffer = [0; 32];

    // read at most five bytes
    let mut handle = game_data_vec[12..].take(32);

    handle.read(&mut buffer);
    println!("{:?}", buffer);
    println!("{:?}", buffer.map(|b| b as char));

    buffer.iter().for_each(|b| print!("{:02X} ", b));
    println!();
}

fn extract_info<P: AsRef<Path>>(comp: &mut cfb::CompoundFile<std::fs::File>, json_path: &P) {
    let mut json_file = std::fs::File::create(json_path).unwrap();
    let json_root = read_tableinfo(comp);
    serde_json::to_writer_pretty(&mut json_file, &json_root).unwrap();
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
    })
}

fn read_tableinfo(comp: &mut cfb::CompoundFile<std::fs::File>) -> serde_json::Value {
    let table_info_path = "/TableInfo";
    // println!("Reading table info at {}", table_info_path);
    let stream_paths: Vec<String> = {
        let walk = comp.walk_storage(table_info_path).unwrap();
        let stream_paths = walk.flat_map(|entry| {
            let path = entry.path();
            let path = path.to_str().unwrap().to_string();
            if entry.is_stream() {
                return Some(path);
            } else {
                return None;
            }
        });
        stream_paths.collect()
    };

    let keys_vals = stream_paths
        .iter()
        .map(|path| {
            let mut stream = comp.open_stream(path).unwrap();
            // read the stream to a string
            let mut buffer = Vec::new();
            stream.read_to_end(&mut buffer).unwrap();

            let buffer = biff_to_utf8(buffer);

            let s = match str::from_utf8(&buffer) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let key = path.replace(table_info_path, "").replacen("/", "", 1);

            return (key, s.to_string());
        })
        .collect::<Vec<_>>();

    // keys_vals.iter().for_each(|(path, value)| {
    //     println!("{} -> {}", path, value);
    // });

    use serde_json::{Map, Value};

    let mut table_info_map = Map::new();

    // assuming keys_vals is a Vec<(String, String)>
    for (key, val) in keys_vals {
        table_info_map.insert(key.to_string(), Value::String(val));
    }

    let json_tableinfo = Value::Object(table_info_map);

    let json_root = json!({ "TableInfo": json_tableinfo });
    json_root
}
