use crate::guifrontend::VpxTables;
use crate::list::{display_table_line, SelectedItem};
use bevy::input::ButtonInput;
use bevy::prelude::{ColorMaterial, Commands, KeyCode, Mesh, Query, Res, ResMut, Window, With};
use bevy::window::PrimaryWindow;
use bevy_asset::Assets;
use bevy_egui::egui::Align2;
use bevy_egui::{egui, EguiContexts};

#[allow(clippy::too_many_arguments)]
pub(crate) fn show_info(
    commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
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
            create_info_box(
                commands,
                keys,
                meshes,
                materials,
                window,
                contexts,
                wtitle,
                gametext.to_owned(),
            );
        };
    }
}

#[allow(clippy::too_many_arguments)]
fn create_info_box(
    _commands: Commands,
    _keys: Res<ButtonInput<KeyCode>>,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<ColorMaterial>>,
    window: &Window,
    mut contexts: EguiContexts,
    wtitle: String,
    wtext: String,
) {
    /*   MacOS window settings
     pub movable_by_window_background: bool,
    pub fullsize_content_view: bool,
    pub has_shadow: bool,
    pub titlebar_shown: bool,
    pub titlebar_transparent: bool,
    pub titlebar_show_title: bool,
    pub titlebar_show_buttons: bool, */

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
            //  ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            //egui::ScrollArea::vertical()
            //   .min_scrolled_width(500.0)
            //  .auto_shrink(false)
            // .animated(true)
            //.show(ui, |ui| {
            ui.add(egui::Label::wrap(egui::Label::new(&wtext)));
            // });
        });
    //});

    // let mut _loopstop = false;
    //
    // //println!("key: {:?}",keys.get_pressed());
    // if keys.pressed(KeyCode::ShiftRight) {
    //     // println!("broken");
    //     _loopstop = true;
    // }
    //
    // let _window = Window {
    //     // Enable transparent support for the window
    //     transparent: true,
    //     decorations: true,
    //     window_level: WindowLevel::AlwaysOnTop,
    //     //       cursor: Cursor {
    //     //           // Allow inputs to pass through to apps behind this app.
    //     //           hit_test: false,
    //     //           ..default()
    //     //       },
    //     ..default()
    // };
}
