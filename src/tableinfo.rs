use cfb::CompoundFile;
use serde_json::json;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::Read;
use utf16string::WStr;

pub fn read_tableinfo(comp: &mut CompoundFile<File>) -> Value {
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

            let s = WStr::from_utf16le(&buffer).unwrap().to_utf8();
            let key = path.replace(table_info_path, "").replacen("/", "", 1);

            return (key, s.to_string());
        })
        .collect::<Vec<_>>();

    // keys_vals.iter().for_each(|(path, value)| {
    //     println!("{} -> {}", path, value);
    // });

    let mut table_info_map = Map::new();

    // assuming keys_vals is a Vec<(String, String)>
    for (key, val) in keys_vals {
        table_info_map.insert(key.to_string(), Value::String(val));
    }

    let json_tableinfo = Value::Object(table_info_map);

    let json_root = json!({ "TableInfo": json_tableinfo });
    json_root
}
