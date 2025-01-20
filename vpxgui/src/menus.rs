use crate::dmd::Dmd;
use bevy::prelude::*;

pub fn dmd_update(
    mut _commands: Commands,
    //flipper: Query<&mut Transform, With<crate::guifrontend::Flipper>>,

    //keys: Res<ButtonInput<KeyCode>>,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<ColorMaterial>>,
    //window_query: Query<&Window, With<PrimaryWindow>>,
    mut visibility: Query<&mut Visibility, With<Dmd>>,
    //mut contexts: EguiContexts,
) {
    let _dmd = (128, 32);
    for mut visibility in visibility.iter_mut() {
        *visibility = Visibility::Visible;
    }
    //let _flipper_transform = flipper.get_single().unwrap();
    //println!("Transform{:?}", flipper_transform.translation);
    //let window = window_query.single();
    //let width = window.resolution.width();
    //let height = window.resolution.height();

    /* let sprite = Sprite {
        color: Color::srgb(1., 0.5, 0.0),
        flip_x: false,
        flip_y: false,
        custom_size: Some(Vec2::new(512.0, 128.0)),
        anchor: Default::default(),
        ..default()
    };  */
    let _color = Color::srgba(0.5, 0., 0., 0.);

    /*/   let paddle = commands
    .spawn(SpriteBundle {
        sprite: sprite,
        transform: Transform::from_xyz(0.0, height / 10. * -1., 0.0),
        //  ..Default::default()
    })
    .id(); */
}

// pub fn gui_update() {}

/*let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_decorations(false) // Hide the OS-specific "chrome" around the window
        .with_inner_size([400.0, 100.0])
        .with_min_inner_size([400.0, 100.0])
        .with_transparent(true), // To have rounded corners we need transparency

    ..Default::default()
}; */

#[derive(Bundle)]
struct SpriteBundle {
    sprite: Sprite,
    transform: Transform,
}
