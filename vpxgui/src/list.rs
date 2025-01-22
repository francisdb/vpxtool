use crate::guifrontend::VpxTables;
use crate::loading::LoadingState;
use bevy::color::palettes::css::{GHOST_WHITE, GOLD};
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use shared::indexer::IndexedTable;
use std::cmp::Ordering;

#[derive(Component, Debug)]
pub(crate) struct TableText {
    pub(crate) list_index: usize,
    pub(crate) table_text: String,
}

#[derive(Resource, Default)]
pub struct SelectedItem {
    pub(crate) index: Option<usize>,
}

#[derive(Component)]
pub struct TextItem;

#[derive(Bundle)]
struct MenuTextBundle {
    text: Text,
    text_font: TextFont,
    text_color: TextColor,
    text_node: Node,
    table_text: TableText,
    text_item: TextItem,
}

const ITEMS_AROUND_SELECTED: usize = 10;
const ITEMS_SHOWN: usize = ITEMS_AROUND_SELECTED * 2 + 1;

pub(crate) fn list_plugin(app: &mut App) {
    app.insert_resource(SelectedItem::default());
    app.add_systems(Startup, create_list);
    app.add_systems(
        Update,
        (input_handling, list_update)
            .chain()
            .run_if(in_state(LoadingState::Ready)),
    );
}

fn create_list(mut commands: Commands) {
    for list_index in 0..ITEMS_SHOWN {
        let distance = (list_index as i32 - ITEMS_AROUND_SELECTED as i32).abs() as f32;
        let alpha = 1.0 - (distance / ITEMS_AROUND_SELECTED as f32);
        let mut text_color = TextColor::from(Color::srgba(
            GHOST_WHITE.red,
            GHOST_WHITE.green,
            GHOST_WHITE.blue,
            alpha,
        ));
        let mut font_size = 15.0;

        if list_index == ITEMS_AROUND_SELECTED {
            text_color = TextColor::from(GOLD);
            font_size = 25.0;
        }

        let top = match list_index.cmp(&ITEMS_AROUND_SELECTED) {
            Ordering::Less => Val::Px(25. + (((list_index as f32) + 1.) * 20.)),
            Ordering::Equal => Val::Px(255. + (((list_index as f32) - 10.5) * 20.)),
            Ordering::Greater => Val::Px(255. + (((list_index as f32) - 10.) * 20.)),
        };

        commands.spawn(MenuTextBundle {
            text: Text::new(""),
            text_font: TextFont {
                font_size,
                ..default()
            },
            text_color,
            text_node: Node {
                // Set the justification of the Text
                //.with_text_justify(JustifyText::Center)
                display: Display::Block,
                position_type: PositionType::Absolute,
                left: Val::Px(20.),
                top,
                right: Val::Px(0.),
                ..default()
            },
            table_text: TableText {
                list_index,
                table_text: "".to_string(),
            },
            text_item: TextItem,
        });
    }
}

fn list_update(
    tables: Res<VpxTables>,
    mut text_items: Query<(&mut TableText, &mut Text), With<TextItem>>,
    selected_item: Res<SelectedItem>,
) {
    // TODO we should only be making changes if the selected item has changed
    let selected_item = selected_item.index.unwrap_or(0);
    let table_indices = generate_table_indices(tables.indexed_tables.len(), selected_item);
    for (mut table_text, mut text) in text_items.iter_mut() {
        let list_index = table_text.list_index;
        let table_index = table_indices[list_index];
        let table = &tables.indexed_tables[table_index];
        let table_name = display_table_line(table);
        let table_description = table
            .table_info
            .table_description
            .clone()
            .unwrap_or("Description missing".to_string());
        table_text.table_text = table_description;
        text.0 = table_name;
    }
}

#[derive(Default)]
struct ShiftIncrement {
    s: f32,
}

fn input_handling(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut shift_stop_watch: Local<Stopwatch>,
    mut shift_applied: Local<ShiftIncrement>,
    mut selected_item_res: ResMut<SelectedItem>,
    tables: Res<VpxTables>,
) {
    let mut selected_item = selected_item_res.index.unwrap_or(0) as i16;

    // Update timers
    shift_stop_watch.tick(time.delta());

    // Adjust increment based on time pressed
    let shift_increment = (shift_stop_watch.elapsed_secs() / 1.5).min(10.0);

    if keys.just_pressed(KeyCode::ShiftRight) {
        selected_item += 1;
        shift_applied.s = 0.0;
        shift_stop_watch.reset();
    } else if keys.just_pressed(KeyCode::ShiftLeft) {
        selected_item -= 1;
        shift_applied.s = 0.0;
        shift_stop_watch.reset();
    } else if keys.pressed(KeyCode::ShiftRight) {
        shift_applied.s += shift_increment;
        if shift_applied.s >= 1.0 {
            selected_item += shift_applied.s.floor() as i16;
            shift_applied.s = shift_applied.s.fract();
        }
    } else if keys.pressed(KeyCode::ShiftLeft) {
        shift_applied.s += shift_increment;
        if shift_applied.s >= 1.0 {
            selected_item -= shift_applied.s.floor() as i16;
            shift_applied.s = shift_applied.s.fract();
        }
    }

    let table_count = tables.indexed_tables.len();

    // Wrap around if one of the bounds are hit.
    let selected_item = wrap_around(selected_item, table_count);
    if selected_item_res.index != Some(selected_item) {
        debug!("Selected item: {} ({} total)", selected_item, table_count);
    }
    selected_item_res.index = Some(selected_item);
}

pub(crate) fn display_table_line(table: &IndexedTable) -> String {
    let file_name = table
        .path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    Some(table.table_info.table_name.to_owned())
        .filter(|s| !s.clone().unwrap_or_default().trim().is_empty())
        .map(|s| {
            match s {
                Some(name) => capitalize_first_letter(&name),
                None => capitalize_first_letter(&file_name),
            }
            // TODO we probably want to show both the file name and the table name
        })
        .unwrap_or(file_name)
}

fn capitalize_first_letter(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
}

fn generate_table_indices(max_index: usize, selected_index: usize) -> [usize; ITEMS_SHOWN] {
    let mut table_indices = [0; ITEMS_SHOWN];
    for (i, item) in table_indices.iter_mut().enumerate() {
        let index = ITEMS_AROUND_SELECTED as i16 - i as i16;
        *item = wrap_around(selected_index as i16 - index, max_index);
    }
    table_indices
}

/// Wraps a number around a maximum value.
fn wrap_around(n: i16, max: usize) -> usize {
    if n >= max as i16 {
        n as usize % max
    } else if n < 0 {
        ((n % max as i16 + max as i16) % max as i16) as usize
    } else {
        n as usize
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_lookup_table_indices() {
        assert_eq!(
            generate_table_indices(10, 0),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0,]
        );
        assert_eq!(
            generate_table_indices(50, 8),
            [48, 49, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18]
        );
    }

    #[test]
    fn test_wrap() {
        assert_eq!(wrap_around(0, 10), 0);
        assert_eq!(wrap_around(10, 10), 0);
        assert_eq!(wrap_around(11, 10), 1);
        assert_eq!(wrap_around(-1, 10), 9);
        assert_eq!(wrap_around(-10, 10), 0);
        assert_eq!(wrap_around(-11, 10), 9);
        assert_eq!(wrap_around(-123, 3), 0);
        assert_eq!(wrap_around(91, 9), 1);
    }
}
