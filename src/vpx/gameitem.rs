mod color;
mod font;
mod light;
mod point;
mod primitive;
mod vertex2d;
mod vertex3d;

use crate::vpx::biff::BiffRead;

use font::Font;
use point::Point;
use primitive::Primitive;

use super::biff::{self, BiffReader};

// TODO we might come up with a macro that generates the biff reading from the struct annotations
//   like VPE

#[derive(PartialEq, Debug)]
pub enum GameItem {
    Wall { name: String, points: Vec<Point> },
    Flipper { name: String },
    Timer { name: String },
    Plunger { name: String },
    TextBox { name: String },
    Bumper { name: String },
    Trigger { name: String, points: Vec<Point> },
    Light(light::Light),
    Kicker { name: String },
    Decal { name: String, font: Font },
    Gate { name: String },
    Spinner { name: String },
    Ramp { name: String, points: Vec<Point> },
    Table { name: String },
    LightCenter { name: String },
    DragPoint { name: String },
    Collection { name: String },
    Reel { name: String },
    LightSequencer { name: String },
    Primitive(primitive::Primitive),
    Flasher { name: String, points: Vec<Point> },
    Rubber { name: String, points: Vec<Point> },
    HitTarget { name: String },
    Other { item_type: u32, name: String },
}

impl GameItem {
    fn name(&self) -> &str {
        match self {
            GameItem::Wall { name, .. } => name,
            GameItem::Flipper { name, .. } => name,
            GameItem::Timer { name, .. } => name,
            GameItem::Plunger { name, .. } => name,
            GameItem::TextBox { name, .. } => name,
            GameItem::Bumper { name, .. } => name,
            GameItem::Trigger { name, .. } => name,
            GameItem::Light(light) => &light.name,
            GameItem::Kicker { name, .. } => name,
            GameItem::Decal { name, .. } => name,
            GameItem::Gate { name, .. } => name,
            GameItem::Spinner { name, .. } => name,
            GameItem::Ramp { name, .. } => name,
            GameItem::Table { name, .. } => name,
            GameItem::LightCenter { name, .. } => name,
            GameItem::DragPoint { name, .. } => name,
            GameItem::Collection { name, .. } => name,
            GameItem::Reel { name, .. } => name,
            GameItem::LightSequencer { name, .. } => name,
            GameItem::Primitive(primitive) => &primitive.name,
            GameItem::Flasher { name, .. } => name,
            GameItem::Rubber { name, .. } => name,
            GameItem::HitTarget { name, .. } => name,
            GameItem::Other { name, .. } => name,
        }
    }
}

// Item types:
// 0: Wall
// 1: Flipper
// 2: Timer
// 3: Plunger
// 4: Text box
// 5: Bumper
// 6: Trigger
// 7: Light
// 8: Kicker
// 9: Decal
// 10: Gate
// 11: Spinner
// 12: Ramp
// 13: Table
// 14: Light Center
// 15: Drag Point
// 16: Collection
// 17: Reel
// 18: Light sequencer
// 19: Primitive
// 20: Flasher
// 21: Rubber
// 22: Hit Target

const ITEM_TYPE_WALL: u32 = 0;
const ITEM_TYPE_FLIPPER: u32 = 1;
const ITEM_TYPE_TIMER: u32 = 2;
const ITEM_TYPE_PLUNGER: u32 = 3;
const ITEM_TYPE_TEXT_BOX: u32 = 4;
const ITEM_TYPE_BUMPER: u32 = 5;
const ITEM_TYPE_TRIGGER: u32 = 6;
const ITEM_TYPE_LIGHT: u32 = 7;
const ITEM_TYPE_KICKER: u32 = 8;
const ITEM_TYPE_DECAL: u32 = 9;
const ITEM_TYPE_GATE: u32 = 10;
const ITEM_TYPE_SPINNER: u32 = 11;
const ITEM_TYPE_RAMP: u32 = 12;
const ITEM_TYPE_TABLE: u32 = 13;
const ITEM_TYPE_LIGHT_CENTER: u32 = 14;
const ITEM_TYPE_DRAG_POINT: u32 = 15;
const ITEM_TYPE_COLLECTION: u32 = 16;
const ITEM_TYPE_REEL: u32 = 17;
const ITEM_TYPE_LIGHT_SEQUENCER: u32 = 18;
const ITEM_TYPE_PRIMITIVE: u32 = 19;
const ITEM_TYPE_FLASHER: u32 = 20;
const ITEM_TYPE_RUBBER: u32 = 21;
const ITEM_TYPE_HIT_TARGET: u32 = 22;

const TYPE_NAMES: [&str; 23] = [
    "Wall",
    "Flipper",
    "Timer",
    "Plunger",
    "Text",
    "Bumper",
    "Trigger",
    "Light",
    "Kicker",
    "Decal",
    "Gate",
    "Spinner",
    "Ramp",
    "Table",
    "LightCenter",
    "DragPoint",
    "Collection",
    "DispReel",
    "LightSeq",
    "Prim",
    "Flasher",
    "Rubber",
    "Target",
];

pub fn read(input: &[u8]) -> GameItem {
    let mut reader = BiffReader::new(input);
    let item_type = reader.get_u32_no_remaining_update();
    println!(
        "  Item type: {} {}",
        item_type, TYPE_NAMES[item_type as usize]
    );
    let item = match item_type {
        ITEM_TYPE_WALL => load_wall(&mut reader),
        ITEM_TYPE_TRIGGER => load_trigger(&mut reader),
        ITEM_TYPE_LIGHT => GameItem::Light(light::Light::biff_read(&mut reader)),
        ITEM_TYPE_RAMP => load_ramp(&mut reader),
        ITEM_TYPE_RUBBER => load_rubber(&mut reader),
        ITEM_TYPE_DECAL => load_decal(&mut reader),
        ITEM_TYPE_PRIMITIVE => GameItem::Primitive(primitive::Primitive::biff_read(&mut reader)),
        ITEM_TYPE_FLASHER => load_flasher(&mut reader),
        other_item_type => load_other_item(&mut reader, other_item_type),
    };
    println!("  Name: {}", item.name());
    dbg!(&item);
    item
}

fn load_flasher(reader: &mut BiffReader<'_>) -> GameItem {
    let mut name = Default::default();
    let mut points: Vec<Point> = Default::default();

    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();
        match tag_str {
            "NAME" => {
                name = reader.get_wide_string();
            }
            "DPNT" => {
                let point = Point::biff_read(reader);
                points.push(point);
            }
            _ => {
                println!("Unknown tag: {}", tag_str);
            }
        }
    }
    GameItem::Flasher { name, points }
}

fn load_trigger(reader: &mut BiffReader<'_>) -> GameItem {
    let mut name = Default::default();
    let mut points: Vec<Point> = Default::default();

    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();
        match tag_str {
            "NAME" => {
                name = reader.get_wide_string();
            }
            "DPNT" => {
                let point = Point::biff_read(reader);
                points.push(point);
            }
            _ => {
                println!("Unknown tag: {}", tag_str);
            }
        }
    }
    GameItem::Trigger { name, points }
}

fn load_decal(reader: &mut BiffReader<'_>) -> GameItem {
    let mut name = Default::default();
    let mut font = Default::default();

    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();
        match tag_str {
            "NAME" => {
                name = reader.get_wide_string();
            }
            "FONT" => {
                font = Font::biff_read(reader);
            }
            _ => {
                println!("Unknown tag: {}", tag_str);
            }
        }
    }
    GameItem::Decal { name, font }
}

fn load_rubber(reader: &mut BiffReader<'_>) -> GameItem {
    let mut name = Default::default();
    let mut points: Vec<Point> = Default::default();

    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();
        match tag_str {
            "NAME" => {
                name = reader.get_wide_string();
            }
            "DPNT" => {
                let point = Point::biff_read(reader);
                points.push(point);
            }
            _ => {
                println!("Unknown tag: {}", tag_str);
            }
        }
    }
    GameItem::Rubber { name, points }
}

fn load_ramp(reader: &mut BiffReader<'_>) -> GameItem {
    let mut name = Default::default();
    let mut points: Vec<Point> = Default::default();

    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();
        match tag_str {
            "NAME" => {
                name = reader.get_wide_string();
            }
            "DPNT" => {
                let point = Point::biff_read(reader);
                points.push(point);
            }
            _ => {
                println!("Unknown tag: {}", tag_str);
            }
        }
    }
    GameItem::Ramp { name, points }
}

fn load_wall(reader: &mut BiffReader<'_>) -> GameItem {
    let mut name = Default::default();
    let mut points: Vec<Point> = Default::default();

    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();
        match tag_str {
            "NAME" => {
                name = reader.get_wide_string();
            }
            "DPNT" => {
                let point = Point::biff_read(reader);
                points.push(point);
            }
            _ => {
                println!("Unknown tag: {}", tag_str);
            }
        }
    }
    GameItem::Wall { name, points }
}

fn load_other_item(reader: &mut BiffReader, other_item_type: u32) -> GameItem {
    let mut name = "".to_string();
    loop {
        reader.next(biff::WARN);
        if reader.is_eof() {
            break;
        }
        let tag = reader.tag();
        let tag_str = tag.as_str();
        match tag_str {
            "NAME" => {
                name = reader.get_wide_string();
            }
            other => {
                println!("Unknown tag: {}", other);
                reader.skip_tag();
            }
        }
    }
    GameItem::Other {
        item_type: other_item_type,
        name,
    }
}
