use crate::guifrontend::Globals;
use crate::wheel::Wheel;
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::prelude::{KeyCode, Query, Res, ResMut, Transform, Visibility, With};

pub(crate) fn table_selection(
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel_query: Query<(&mut Visibility, &mut Wheel, &mut Transform), With<Wheel>>,
    mut globals: ResMut<Globals>,
) {
    // arbitrary number to indicate there is no selected item.
    let mut selected_item: i16 = -2;

    // Count entities
    let mut num = 1;
    num += wheel_query.iter().count() as i16;

    // Find current selection
    for (_visibility, wheel, _transform) in wheel_query.iter() {
        if wheel.selected {
            selected_item = wheel.item_number;
        }
    }
    // If no selection, set it to item 0
    if selected_item == -2 {
        for (_visibility, mut wheel, _transform) in wheel_query.iter_mut() {
            if wheel.item_number == 0 {
                wheel.selected = true;
                selected_item = 0;
            }
        }
    };

    // TODO: use magsave keys to scroll in pages
    if keys.just_pressed(KeyCode::ShiftRight) {
        selected_item += 1;
    } else if keys.just_pressed(KeyCode::ShiftLeft) {
        selected_item -= 1;
    }

    // Wrap around if one of the bounds are hit.
    if selected_item == num - 1 {
        selected_item = 0;
    } else if selected_item == -1 {
        selected_item = num - 2;
    }
    if globals.selected_item != Some(selected_item) {
        info!("Selected item: {}", selected_item);
    }
    globals.selected_item = Some(selected_item);
}
