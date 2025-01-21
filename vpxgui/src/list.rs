use crate::guifrontend::VpxTables;
use crate::loading::LoadingState;
use bevy::color::palettes::css::{GHOST_WHITE, GOLD};
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy::window::PrimaryWindow;
use shared::indexer::IndexedTable;

#[derive(Component, Debug)]
pub(crate) struct TableText {
    pub(crate) item_number: usize,
    pub(crate) table_text: String,
}

#[derive(Resource, Default)]
pub struct SelectedItem {
    pub(crate) index: Option<usize>,
}

#[derive(Component)]
pub struct TextItemSelected;

#[derive(Component)]
pub struct TextItem;

#[derive(Bundle)]
struct MenuTextBundle {
    text: Text,
    text_font: TextFont,
    text_color: TextColor,
    text_bundle: Node,
    table_text: TableText,
}

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

fn create_list(
    vpx_tables: Res<VpxTables>,
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let tables = &vpx_tables.indexed_tables;
    let window = window_query.single();

    for (table_index, info) in tables.iter().enumerate() {
        let table_name = display_table_line(info);

        commands.spawn((
            TextItemSelected,
            MenuTextBundle {
                text: Text::new(&table_name),
                text_font: TextFont {
                    font_size: 20.0,
                    ..default()
                },
                text_color: TextColor::from(GHOST_WHITE),
                text_bundle: Node {
                    // Set the justification of the Text
                    //.with_text_justify(JustifyText::Center)
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    left: Val::Px(20.),
                    top: Val::Px(window.height() * 0.025), //-(height-(height/2.+(scale*2.)))),
                    right: Val::Px(0.),
                    ..default()
                },
                table_text: TableText {
                    item_number: table_index,
                    table_text: match info.table_info.table_description.clone() {
                        Some(a) => a,
                        _ => "Empty".to_owned(),
                    },
                },
            },
        ));

        commands.spawn((
            TextItem,
            MenuTextBundle {
                text: Text::new(&table_name),
                text_font: TextFont {
                    font_size: 20.0,
                    ..default()
                },
                text_color: TextColor::from(GHOST_WHITE),
                // Set the justification of the Text
                //.with_text_justify(JustifyText::Center)
                text_bundle: Node {
                    flex_direction: FlexDirection::Row,
                    align_content: AlignContent::FlexEnd,
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    left: Val::Px(20.),
                    top: Val::Px(window.height() * 0.2), //-(height-(height/2.+(scale*2.)))),
                    right: Val::Px(0.),
                    ..default()
                },

                table_text: TableText {
                    item_number: table_index,
                    table_text: match info.table_info.table_description.clone() {
                        Some(a) => a,
                        _ => "Empty".to_owned(),
                    },
                },
            },
        ));
    }
}

const ITEMS_AROUND_SELECTED: usize = 10;
const ITEMS_SHOWN: usize = ITEMS_AROUND_SELECTED * 2 + 1;

fn list_update(
    tables: Res<VpxTables>,
    mut text_items: Query<
        (&mut TableText, &mut TextFont, &mut Node, &mut TextColor),
        With<TextItemSelected>,
    >,
    selected_item: Res<SelectedItem>,
) {
    // from here on no more changes to selected_item, make it immutable
    let selected_item = selected_item.index.unwrap_or(0);

    // change name of game
    for (items, mut font, mut textstyle, mut color) in text_items.iter_mut() {
        if items.item_number != selected_item {
            textstyle.display = Display::None;
            *color = TextColor::from(GHOST_WHITE);
        } else {
            *color = TextColor::from(GHOST_WHITE);
            font.font_size = 20.0;
            textstyle.display = Display::Block;
        }
    }

    let mut counter = 0;

    let table_count = tables.indexed_tables.len() as i16 + 1;

    // clear all game name assets
    for (_items, mut fontsize, mut textstyle, mut color) in text_items.iter_mut() {
        if table_count > 21 {
            textstyle.display = Display::None;
            fontsize.font_size = 20.0;
            *color = TextColor::from(GHOST_WHITE);
        } else {
            textstyle.display = Display::Block;
            fontsize.font_size = 20.0;
            *color = TextColor::from(GHOST_WHITE);

            textstyle.top = Val::Px(255. + (((counter as f32) + 1.) * 20.));
            counter += 1;
        }
    }

    if table_count > ITEMS_SHOWN as i16 {
        let table_indices = generate_table_indices(tables.indexed_tables.len(), selected_item);
        for _name in table_indices {
            for (items, mut fontsize, mut text_style, mut color) in text_items.iter_mut() {
                for (index, item) in table_indices.iter().enumerate().take(9 + 1) {
                    if items.item_number == *item {
                        //wtitle = items;
                        *color = TextColor::from(GHOST_WHITE);
                        text_style.top = Val::Px(25. + (((index as f32) + 1.) * 20.));
                        fontsize.font_size = 15.0;
                        text_style.display = Display::Block;
                        //        if items.itemnumber == selected_item {textstyle.color:GOLD.into(); }
                    }
                }
                for (index, item) in table_indices.iter().enumerate().skip(10) {
                    if items.item_number == *item {
                        fontsize.font_size = 25.0;
                        *color = TextColor::from(GOLD);
                        text_style.top = Val::Px(255. + (((index as f32) - 10.5) * 20.));
                        text_style.display = Display::Block;
                        break;
                    }
                }

                for (index, item) in table_indices.iter().enumerate().skip(11) {
                    if items.item_number == *item {
                        *color = TextColor::from(GHOST_WHITE);
                        fontsize.font_size = 15.0;
                        text_style.top = Val::Px(255. + (((index as f32) - 10.) * 20.));
                        text_style.display = Display::Block;
                        //        if items.itemnumber == selected_item {textstyle.color:GOLD.into(); }
                    }
                }
            }
        }
    }
    //  counter += 1;
}

fn input_handling(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut shift_stop_watch: Local<Stopwatch>,
    mut selected_item_res: ResMut<SelectedItem>,
    tables: Res<VpxTables>,
) {
    // TODO handle 0 table count case?
    let mut selected_item = selected_item_res.index.unwrap_or(0) as i16;

    // Update timers
    shift_stop_watch.tick(time.delta());

    // Adjust increment based on time pressed
    let shift_increment = (shift_stop_watch.elapsed_secs() * 2.0) as i16;

    if keys.just_pressed(KeyCode::ShiftRight) {
        selected_item += 1;
        shift_stop_watch.reset();
    } else if keys.just_pressed(KeyCode::ShiftLeft) {
        selected_item -= 1;
        shift_stop_watch.reset();
    } else if keys.pressed(KeyCode::ShiftRight) {
        selected_item += shift_increment;
    } else if keys.pressed(KeyCode::ShiftLeft) {
        selected_item -= shift_increment;
    }

    let table_count = tables.indexed_tables.len();

    // Wrap around if one of the bounds are hit.
    let selected_item = wrap_around(selected_item, table_count);
    if selected_item_res.index != Some(selected_item) {
        info!("Selected item: {} ({} total)", selected_item, table_count);
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
        .filter(|s| !s.clone().unwrap_or_default().is_empty())
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
