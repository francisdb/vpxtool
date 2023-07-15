mod bumper;
mod collection;
mod color;
mod decal;
mod dragpoint;
mod flasher;
mod flipper;
mod font;
mod gate;
mod hittarget;
mod kicker;
mod light;
mod lightcenter;
mod lightsequencer;
mod plunger;
mod primitive;
mod ramp;
mod reel;
mod rubber;
mod spinner;
mod table;
mod textbox;
mod timer;
mod trigger;
mod vertex2d;
mod vertex3d;
mod wall;

use crate::vpx::biff::BiffRead;

use dragpoint::DragPoint;
use font::Font;

use super::biff::{self, BiffReader};

// TODO we might come up with a macro that generates the biff reading from the struct annotations
//   like VPE

#[derive(PartialEq, Debug)]
pub enum GameItem {
    Wall(wall::Wall),
    Flipper(flipper::Flipper),
    Timer(timer::Timer),
    Plunger(plunger::Plunger),
    TextBox(textbox::TextBox),
    Bumper(bumper::Bumper),
    Trigger(trigger::Trigger),
    Light(light::Light),
    Kicker(kicker::Kicker),
    Decal(decal::Decal),
    Gate(gate::Gate),
    Spinner(spinner::Spinner),
    Ramp(ramp::Ramp),
    Table(table::Table),
    LightCenter(lightcenter::LightCenter),
    DragPoint(dragpoint::DragPoint),
    Collection(collection::Collection),
    Reel(reel::Reel),
    LightSequencer(lightsequencer::LightSequencer),
    Primitive(primitive::Primitive),
    Flasher(flasher::Flasher),
    Rubber(rubber::Rubber),
    HitTarget(hittarget::HitTarget),
    Other { item_type: u32, name: String },
}

impl GameItem {
    fn name(&self) -> &str {
        match self {
            GameItem::Wall(wall) => &wall.name,
            GameItem::Flipper(flipper) => &flipper.name,
            GameItem::Timer(timer) => &timer.name,
            GameItem::Plunger(plunger) => &plunger.name,
            GameItem::TextBox(textbox) => &textbox.name,
            GameItem::Bumper(bumper) => &bumper.name,
            GameItem::Trigger(trigger) => &trigger.name,
            GameItem::Light(light) => &light.name,
            GameItem::Kicker(kicker) => &kicker.name,
            GameItem::Decal(decal) => &decal.name,
            GameItem::Gate(gate) => &gate.name,
            GameItem::Spinner(spinner) => &spinner.name,
            GameItem::Ramp(ramp) => &ramp.name,
            GameItem::Table(table) => &table.name,
            GameItem::LightCenter(lightcenter) => &lightcenter.name,
            GameItem::DragPoint(dragpoint) => "unnamed dragpoint",
            GameItem::Collection(collection) => &collection.name,
            GameItem::Reel(reel) => &reel.name,
            GameItem::LightSequencer(lightsequencer) => &lightsequencer.name,
            GameItem::Primitive(primitive) => &primitive.name,
            GameItem::Flasher(flasher) => &flasher.name,
            GameItem::Rubber(rubber) => &rubber.name,
            GameItem::HitTarget(hittarget) => &hittarget.name,
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

pub const FILTER_NONE: u32 = 0;
pub const FILTER_ADDITIVE: u32 = 1;
pub const FILTER_OVERLAY: u32 = 2;
pub const FILTER_MULTIPLY: u32 = 3;
pub const FILTER_SCREEN: u32 = 4;

pub const IMAGE_ALIGN_WORLD: u32 = 0;
pub const IMAGE_ALIGN_TOP_LEFT: u32 = 1;
pub const IMAGE_ALIGN_CENTER: u32 = 2;

// TODO move this to the component that it relates to?
pub const TRIGGER_SHAPE_NONE: u32 = 0;
pub const TRIGGER_SHAPE_WIRE_A: u32 = 1;
pub const TRIGGER_SHAPE_STAR: u32 = 2;
pub const TRIGGER_SHAPE_WIRE_B: u32 = 3;
pub const TRIGGER_SHAPE_BUTTON: u32 = 4;
pub const TRIGGER_SHAPE_WIRE_C: u32 = 5;
pub const TRIGGER_SHAPE_WIRE_D: u32 = 6;

pub fn read(input: &[u8]) -> GameItem {
    let mut reader = BiffReader::new(input);
    let item_type = reader.get_u32_no_remaining_update();
    // if item_type != ITEM_TYPE_TRIGGER {
    //     return GameItem::Other {
    //         item_type,
    //         name: "skipped".to_owned(),
    //     };
    // }
    println!(
        "  Item type: {} {}",
        item_type, TYPE_NAMES[item_type as usize]
    );
    let item = match item_type {
        ITEM_TYPE_WALL => GameItem::Wall(wall::Wall::biff_read(&mut reader)),
        ITEM_TYPE_FLIPPER => GameItem::Flipper(flipper::Flipper::biff_read(&mut reader)),
        ITEM_TYPE_TIMER => GameItem::Timer(timer::Timer::biff_read(&mut reader)),
        ITEM_TYPE_PLUNGER => GameItem::Plunger(plunger::Plunger::biff_read(&mut reader)),
        ITEM_TYPE_TEXT_BOX => GameItem::TextBox(textbox::TextBox::biff_read(&mut reader)),
        ITEM_TYPE_BUMPER => GameItem::Bumper(bumper::Bumper::biff_read(&mut reader)),
        ITEM_TYPE_TRIGGER => GameItem::Trigger(trigger::Trigger::biff_read(&mut reader)),
        ITEM_TYPE_LIGHT => GameItem::Light(light::Light::biff_read(&mut reader)),
        ITEM_TYPE_KICKER => GameItem::Kicker(kicker::Kicker::biff_read(&mut reader)),
        ITEM_TYPE_DECAL => GameItem::Decal(decal::Decal::biff_read(&mut reader)),
        ITEM_TYPE_GATE => GameItem::Gate(gate::Gate::biff_read(&mut reader)),
        ITEM_TYPE_SPINNER => GameItem::Spinner(spinner::Spinner::biff_read(&mut reader)),
        ITEM_TYPE_RAMP => GameItem::Ramp(ramp::Ramp::biff_read(&mut reader)),
        ITEM_TYPE_TABLE => GameItem::Table(table::Table::biff_read(&mut reader)),
        ITEM_TYPE_LIGHT_CENTER => {
            GameItem::LightCenter(lightcenter::LightCenter::biff_read(&mut reader))
        }
        ITEM_TYPE_DRAG_POINT => GameItem::DragPoint(dragpoint::DragPoint::biff_read(&mut reader)),
        ITEM_TYPE_COLLECTION => {
            GameItem::Collection(collection::Collection::biff_read(&mut reader))
        }
        ITEM_TYPE_REEL => GameItem::Reel(reel::Reel::biff_read(&mut reader)),
        ITEM_TYPE_LIGHT_SEQUENCER => {
            GameItem::LightSequencer(lightsequencer::LightSequencer::biff_read(&mut reader))
        }
        ITEM_TYPE_PRIMITIVE => GameItem::Primitive(primitive::Primitive::biff_read(&mut reader)),
        ITEM_TYPE_FLASHER => GameItem::Flasher(flasher::Flasher::biff_read(&mut reader)),
        ITEM_TYPE_RUBBER => GameItem::Rubber(rubber::Rubber::biff_read(&mut reader)),
        ITEM_TYPE_HIT_TARGET => GameItem::HitTarget(hittarget::HitTarget::biff_read(&mut reader)),
        other_item_type => load_other_item(&mut reader, other_item_type),
    };
    println!("  Name: {}", item.name());
    //dbg!(&item);
    item
}

// TODO also make this a real (Generic) item type that keeps all the data as binary blobs
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
