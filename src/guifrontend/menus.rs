use bevy::color::palettes::css::*;
use bevy::core_pipeline::{
    bloom::{BloomCompositeMode, BloomSettings},
    tonemapping::Tonemapping,
};
use bevy::render::view::visibility;
use bevy::sprite::{MaterialMesh2dBundle, Wireframe2dConfig, Wireframe2dPlugin};
use bevy::window::*;
use bevy::{input::common_conditions::*, prelude::*};
use bevy_asset::*;
use bevy_asset_loader::prelude::*;
use bevy_egui::egui::Align2;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

pub fn create_info_box(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
            ((height / 2.0) - 250.0),
        ))
        .min_width(500.0)
        .min_height(500.0)
        .pivot(Align2::LEFT_TOP)
        .show(contexts.ctx_mut(), |ui| {
            //  ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            //egui::ScrollArea::vertical()
            //   .min_scrolled_width(500.0)
            ///  .auto_shrink(false)
            // .animated(true)
            //.show(ui, |ui| {
            ui.add(egui::Label::wrap(egui::Label::new(&wtext)));
            // });
        });
    //});

    let mut loopstop = false;

    //println!("key: {:?}",keys.get_pressed());
    if keys.pressed(KeyCode::ShiftRight) {
        println!("broken");
        loopstop = true;
    }

    let window = Window {
        // Enable transparent support for the window
        transparent: true,
        decorations: true,
        window_level: WindowLevel::AlwaysOnTop,
        //       cursor: Cursor {
        //           // Allow inputs to pass through to apps behind this app.
        //           hit_test: false,
        //           ..default()
        //       },
        ..default()
    };
}

pub fn gui_update() {}

/*let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_decorations(false) // Hide the OS-specific "chrome" around the window
        .with_inner_size([400.0, 100.0])
        .with_min_inner_size([400.0, 100.0])
        .with_transparent(true), // To have rounded corners we need transparency

    ..Default::default()
}; */
