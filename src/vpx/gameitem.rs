mod bumper;
mod collection;
mod decal;
mod dragpoint;
mod flasher;
mod flipper;
mod font;
mod gate;
mod generic;
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

use super::biff::{BiffReader, BiffWrite, BiffWriter};

// TODO we might come up with a macro that generates the biff reading from the struct annotations
//   like VPE

trait GameItem: BiffRead {
    fn name(&self) -> &str;
}

#[derive(PartialEq, Debug)]
pub enum GameItemEnum {
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
    Generic(u32, generic::Generic),
}

impl GameItemEnum {
    fn name(&self) -> &str {
        match self {
            GameItemEnum::Wall(wall) => &wall.name,
            GameItemEnum::Flipper(flipper) => flipper.name(),
            GameItemEnum::Timer(timer) => &timer.name,
            GameItemEnum::Plunger(plunger) => &plunger.name,
            GameItemEnum::TextBox(textbox) => &textbox.name,
            GameItemEnum::Bumper(bumper) => bumper.name(),
            GameItemEnum::Trigger(trigger) => &trigger.name,
            GameItemEnum::Light(light) => &light.name,
            GameItemEnum::Kicker(kicker) => &kicker.name,
            GameItemEnum::Decal(decal) => &decal.name,
            GameItemEnum::Gate(gate) => &gate.name,
            GameItemEnum::Spinner(spinner) => &spinner.name,
            GameItemEnum::Ramp(ramp) => &ramp.name,
            GameItemEnum::Table(table) => &table.name,
            GameItemEnum::LightCenter(lightcenter) => &lightcenter.name,
            GameItemEnum::DragPoint(dragpoint) => dragpoint.name(),
            GameItemEnum::Collection(collection) => &collection.name,
            GameItemEnum::Reel(reel) => &reel.name,
            GameItemEnum::LightSequencer(lightsequencer) => &lightsequencer.name,
            GameItemEnum::Primitive(primitive) => &primitive.name,
            GameItemEnum::Flasher(flasher) => &flasher.name,
            GameItemEnum::Rubber(rubber) => &rubber.name,
            GameItemEnum::HitTarget(hittarget) => &hittarget.name,
            GameItemEnum::Generic(_item_type, generic) => generic.name(),
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
// 15: Drag Point (does this make sense on it's own?)
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

// const TYPE_NAMES: [&str; 23] = [
//     "Wall",
//     "Flipper",
//     "Timer",
//     "Plunger",
//     "Text",
//     "Bumper",
//     "Trigger",
//     "Light",
//     "Kicker",
//     "Decal",
//     "Gate",
//     "Spinner",
//     "Ramp",
//     "Table",
//     "LightCenter",
//     "DragPoint",
//     "Collection",
//     "DispReel",
//     "LightSeq",
//     "Prim",
//     "Flasher",
//     "Rubber",
//     "Target",
// ];

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

pub fn read(input: &[u8]) -> GameItemEnum {
    let mut reader = BiffReader::new(input);
    let item_type = reader.get_u32_no_remaining_update();
    match item_type {
        ITEM_TYPE_WALL => GameItemEnum::Wall(wall::Wall::biff_read(&mut reader)),
        ITEM_TYPE_FLIPPER => GameItemEnum::Flipper(flipper::Flipper::biff_read(&mut reader)),
        ITEM_TYPE_TIMER => GameItemEnum::Timer(timer::Timer::biff_read(&mut reader)),
        ITEM_TYPE_PLUNGER => GameItemEnum::Plunger(plunger::Plunger::biff_read(&mut reader)),
        ITEM_TYPE_TEXT_BOX => GameItemEnum::TextBox(textbox::TextBox::biff_read(&mut reader)),
        ITEM_TYPE_BUMPER => GameItemEnum::Bumper(bumper::Bumper::biff_read(&mut reader)),
        ITEM_TYPE_TRIGGER => GameItemEnum::Trigger(trigger::Trigger::biff_read(&mut reader)),
        ITEM_TYPE_LIGHT => GameItemEnum::Light(light::Light::biff_read(&mut reader)),
        ITEM_TYPE_KICKER => GameItemEnum::Kicker(kicker::Kicker::biff_read(&mut reader)),
        ITEM_TYPE_DECAL => GameItemEnum::Decal(decal::Decal::biff_read(&mut reader)),
        ITEM_TYPE_GATE => GameItemEnum::Gate(gate::Gate::biff_read(&mut reader)),
        ITEM_TYPE_SPINNER => GameItemEnum::Spinner(spinner::Spinner::biff_read(&mut reader)),
        ITEM_TYPE_RAMP => GameItemEnum::Ramp(ramp::Ramp::biff_read(&mut reader)),
        ITEM_TYPE_TABLE => GameItemEnum::Table(table::Table::biff_read(&mut reader)),
        ITEM_TYPE_LIGHT_CENTER => {
            GameItemEnum::LightCenter(lightcenter::LightCenter::biff_read(&mut reader))
        }
        ITEM_TYPE_DRAG_POINT => {
            GameItemEnum::DragPoint(dragpoint::DragPoint::biff_read(&mut reader))
        }
        ITEM_TYPE_COLLECTION => {
            GameItemEnum::Collection(collection::Collection::biff_read(&mut reader))
        }
        ITEM_TYPE_REEL => GameItemEnum::Reel(reel::Reel::biff_read(&mut reader)),
        ITEM_TYPE_LIGHT_SEQUENCER => {
            GameItemEnum::LightSequencer(lightsequencer::LightSequencer::biff_read(&mut reader))
        }
        ITEM_TYPE_PRIMITIVE => {
            GameItemEnum::Primitive(primitive::Primitive::biff_read(&mut reader))
        }
        ITEM_TYPE_FLASHER => GameItemEnum::Flasher(flasher::Flasher::biff_read(&mut reader)),
        ITEM_TYPE_RUBBER => GameItemEnum::Rubber(rubber::Rubber::biff_read(&mut reader)),
        ITEM_TYPE_HIT_TARGET => {
            GameItemEnum::HitTarget(hittarget::HitTarget::biff_read(&mut reader))
        }
        other_item_type => {
            GameItemEnum::Generic(other_item_type, generic::Generic::biff_read(&mut reader))
        }
    }
}

pub(crate) fn write(gameitem: &GameItemEnum) -> Vec<u8> {
    match gameitem {
        GameItemEnum::Wall(wall) => write_with_type(ITEM_TYPE_WALL, wall),
        GameItemEnum::Flipper(flipper) => write_with_type(ITEM_TYPE_FLIPPER, flipper),
        GameItemEnum::Timer(timer) => write_with_type(ITEM_TYPE_TIMER, timer),
        GameItemEnum::Plunger(plunger) => write_with_type(ITEM_TYPE_PLUNGER, plunger),
        GameItemEnum::TextBox(textbox) => write_with_type(ITEM_TYPE_TEXT_BOX, textbox),
        GameItemEnum::Bumper(bumper) => write_with_type(ITEM_TYPE_BUMPER, bumper),
        GameItemEnum::Trigger(trigger) => write_with_type(ITEM_TYPE_TRIGGER, trigger),
        GameItemEnum::Light(light) => write_with_type(ITEM_TYPE_LIGHT, light),
        GameItemEnum::Kicker(kicker) => write_with_type(ITEM_TYPE_KICKER, kicker),
        GameItemEnum::Decal(decal) => write_with_type(ITEM_TYPE_DECAL, decal),
        GameItemEnum::Gate(gate) => write_with_type(ITEM_TYPE_GATE, gate),
        GameItemEnum::Spinner(spinner) => write_with_type(ITEM_TYPE_SPINNER, spinner),
        GameItemEnum::Ramp(ramp) => write_with_type(ITEM_TYPE_RAMP, ramp),
        // GameItemEnum::Table(table) => write_with_type(ITEM_TYPE_TABLE, table),
        // GameItemEnum::LightCenter(lightcenter) => {
        //     write_with_type(ITEM_TYPE_LIGHT_CENTER, lightcenter)
        // }
        // GameItemEnum::DragPoint(dragpoint) => write_with_type(ITEM_TYPE_DRAG_POINT, dragpoint),
        // GameItemEnum::Collection(collection) => write_with_type(ITEM_TYPE_COLLECTION, collection),
        GameItemEnum::Reel(reel) => write_with_type(ITEM_TYPE_REEL, reel),
        GameItemEnum::LightSequencer(lightsequencer) => {
            write_with_type(ITEM_TYPE_LIGHT_SEQUENCER, lightsequencer)
        }
        GameItemEnum::Primitive(primitive) => write_with_type(ITEM_TYPE_PRIMITIVE, primitive),
        GameItemEnum::Flasher(flasher) => write_with_type(ITEM_TYPE_FLASHER, flasher),
        GameItemEnum::Rubber(rubber) => write_with_type(ITEM_TYPE_RUBBER, rubber),
        GameItemEnum::HitTarget(hittarget) => write_with_type(ITEM_TYPE_HIT_TARGET, hittarget),
        // GameItemEnum::Generic(item_type, generic) => write_with_type(*item_type, generic),
        _ => {
            unimplemented!("write gameitem {:?}", gameitem);
            //vec![]
        }
    }
}

fn write_with_type<T: BiffWrite>(item_type: u32, item: &T) -> Vec<u8> {
    let mut writer = BiffWriter::new();
    writer.write_u32(item_type);
    item.biff_write(&mut writer);
    writer.get_data().to_vec()
}
