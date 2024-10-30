use bevy::render::view::visibility;
use bevy::{input::common_conditions::*,prelude::*};
use bevy::color::palettes::css::*;
use bevy_asset_loader::prelude::*;
use bevy_asset::*;
use bevy::{core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,}};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy::sprite::{Wireframe2dConfig, Wireframe2dPlugin,
    MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::*;


pub fn createinfobox(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut contexts: EguiContexts, wtitle: String, wtext: String)
{ 
    let mut window = window_query.get_single().unwrap();
    let color = Color::hsl(20., 0.95, 0.7);
    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Capsule2d::new((window.height()/2.)*0.75, 150.0))),
        material: materials.add(color),
        transform: Transform::from_xyz(
            // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
            (window.width())*0.25,
            0.0,
            0.0,
        ),
        ..default()
    });

 /*   egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");});

        let mut loopstop=false;
        
        println!("herere");
        //println!("key: {:?}",keys.get_pressed());
        if keys.pressed(KeyCode::ShiftRight) {println!("broken"); loopstop= true;}
   */     
        /* 
    let window = Window {
        // Enable transparent support for the window
        transparent: true,
        decorations: true,
        window_level: WindowLevel::AlwaysOnTop,
        cursor: Cursor {
            // Allow inputs to pass through to apps behind this app.
            hit_test: false,
            ..default()
        },
        ..default()
    };
    */
    println!("hello there");
}   
    
     

/*let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_decorations(false) // Hide the OS-specific "chrome" around the window
        .with_inner_size([400.0, 100.0])
        .with_min_inner_size([400.0, 100.0])
        .with_transparent(true), // To have rounded corners we need transparency

    ..Default::default()
}; */
