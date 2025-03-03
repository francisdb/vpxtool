use crate::guifrontend::VpxTables;
use crate::list::{SelectedItem, display_table_line};
use bevy::input::ButtonInput;
use bevy::prelude::{KeyCode, Query, Res, ResMut, Window, With};
use bevy::window::PrimaryWindow;
use bevy_egui::egui::Align2;
use bevy_egui::{EguiContexts, egui};

#[allow(clippy::too_many_arguments)]
pub(crate) fn show_info(
    keys: Res<ButtonInput<KeyCode>>,
    mut globals: ResMut<crate::guifrontend::Globals>,
    selected_item_res: Res<SelectedItem>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    contexts: EguiContexts,
    tables: Res<VpxTables>,
) {
    // TODO why is this modifying a unrelated global?
    if keys.just_pressed(KeyCode::Digit1) {
        globals.vpinball_running = !globals.vpinball_running;
    }
    if let Ok(window) = window_query.get_single() {
        let selected_item = selected_item_res.index.unwrap_or(0);
        let table = &tables.indexed_tables[selected_item];

        let gametext = table
            .table_info
            .table_description
            .clone()
            .filter(|x| !x.trim().is_empty())
            .unwrap_or("[no description]".to_string());
        let wtitle = display_table_line(table);

        // FIXME, this keeps creating windows???
        if globals.vpinball_running {
            create_info_box(window, contexts, wtitle, gametext.to_owned());
        };
    }
}

#[allow(clippy::too_many_arguments)]
fn create_info_box(window: &Window, mut contexts: EguiContexts, wtitle: String, wtext: String) {
    let width = window.resolution.width();
    let height = window.resolution.height();

    egui::Window::new(&wtitle)
        .vscroll(true)
        .current_pos(egui::Pos2::new(
            ((width / 2.0) - 250.0) - 10.0,
            (height / 2.0) - 250.0,
        ))
        .min_width(500.0)
        .min_height(500.0)
        .pivot(Align2::LEFT_TOP)
        .show(contexts.ctx_mut(), |ui| {
            ui.add(egui::Label::wrap(egui::Label::new(&wtext)));
        });
}
