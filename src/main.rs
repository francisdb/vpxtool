use clap::{arg, Command};
use colored::Colorize;
use serde_json::json;
use std::io::{self, Read};
use std::path::Path;
use std::process::exit;
use std::str;

fn main() {
    let matches = Command::new("vpxtool")
        .version("0.1")
        .author("Francis DB")
        .about("Extracts and assembles vpx files")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("extract")
                .about("Extracts a vpx file")
                .arg(arg!(<VPXPATH> "The path to the vpx file").required(true)),
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
            extract(expanded_path.as_ref());
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn extract(vpx_file_path: &str) {
    // let (comp_path, inner_path) = expanded_path.split_once('/').unwrap();
    let mut comp = cfb::open(&vpx_file_path).unwrap();

    let json_root = extract_tableinfo(&mut comp);

    // println!("{}", serde_json::to_string_pretty(&root).unwrap());

    // TODO write the json to a file with the same name as the vpx file
    // make root dir if missing
    let root_dir_path_str = vpx_file_path.replace(".vpx", "");
    let root_dir_path = Path::new(&root_dir_path_str);
    let mut root_dir = std::fs::DirBuilder::new();
    root_dir.recursive(true);
    // ask for confirmation if the directory exists
    if root_dir_path.exists() {
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

    let json_path = root_dir_path.join("TableInfo.json");

    let mut file = std::fs::File::create(&json_path).unwrap();
    serde_json::to_writer_pretty(&mut file, &json_root).unwrap();

    println!("Info file written to {}", json_path.display());

    // read all entries
    let entries: Vec<String> = comp
        .walk()
        .filter(|entry| entry.is_stream() && !entry.path().starts_with("/TableInfo"))
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
        println!("Writing to {}", file_path.display());
        // make sure the parent directory exists
        let parent = file_path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        let mut file = std::fs::File::create(file_path).unwrap();
        io::copy(&mut stream, &mut file).unwrap();
    })

    // let mut stream = comp.open_stream(inner_path).unwrap();
    // io::copy(&mut stream, &mut io::stdout()).unwrap();
}

fn extract_tableinfo(comp: &mut cfb::CompoundFile<std::fs::File>) -> serde_json::Value {
    let table_info_path = "/TableInfo";
    println!("Reading table info at {}", table_info_path);
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

fn biff_to_utf8(buffer: Vec<u8>) -> Vec<u8> {
    // the string has each char suffixed with a zero
    // not sure what format this is (biff?)
    // https://github.com/vpinball/vpinball/blob/c3c59e09ed56a69759280867affa1f0abf537451/pintable.cpp#L3117
    // https://github.com/freezy/VisualPinball.Engine/blob/ec1e9765cd4832c134e889d6e6d03320bc404bd5/VisualPinball.Engine/IO/BiffUtil.cs#L57

    // remove each second byte from the stream
    // this is probably not the best way to do this
    // but it works for now
    let uneven_chars = buffer
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, b)| *b)
        .collect::<Vec<_>>();
    uneven_chars
}
