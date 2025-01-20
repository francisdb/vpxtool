use bevy::math::{Quat, Vec3};
use bevy::prelude::{
    default, Bundle, Commands, Component, Query, Res, Sprite, Transform, Visibility, Window, With,
};
use bevy::window::PrimaryWindow;
use bevy_asset::AssetServer;

#[derive(Component)]
pub struct Flipper;

#[derive(Component)]
pub struct Flipper1;

#[derive(Bundle)]
struct FlipperBundle {
    sprite: Sprite,
    transform: Transform,
    // translate: Translate,
    //global_transform: GlobalTransform,
    //    visibility: Visibility,
    //    wheel: Wheel,
    //inherited_visibility: InheritedVisibility,
    visibility: Visibility,
    flipper: Flipper,
}

#[derive(Bundle)]
struct FlipperBundle1 {
    sprite: Sprite,
    transform: Transform,
    // translate: Translate,
    //global_transform: GlobalTransform,
    //    visibility: Visibility,
    //    wheel: Wheel,
    //inherited_visibility: InheritedVisibility,
    visibility: Visibility,
    flipper1: Flipper1,
}

pub(crate) fn create_flippers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    let window_width = window.width();
    let window_height = window.height();
    commands.spawn(FlipperBundle {
        sprite: Sprite {
            // texture: asset_server.load("/usr/tables/wheels/Sing Along (Gottlieb 1967).png"),
            image: asset_server.load("left-flipper.png"),
            ..default()
        },
        visibility: Visibility::Hidden,

        transform: Transform {
            translation: Vec3::new(
                window_width - (window_width * 0.60) - 225.,
                (window_height * 0.25) + 60.,
                0.,
            ),
            scale: (Vec3::new(0.5, 0.5, 1.0)),
            rotation: Quat::from_rotation_z(-0.25),
        },
        flipper: Flipper,
    });

    commands.spawn(FlipperBundle1 {
        sprite: Sprite {
            image: asset_server.load("right-flipper.png"),
            ..default()
        },
        visibility: Visibility::Hidden,

        transform: Transform {
            translation: Vec3::new(
                window_width - (window_width * 0.60),
                window_height * 0.25 + 60.,
                0.,
            ),
            scale: (Vec3::new(0.5, 0.5, 1.0)),
            rotation: Quat::from_rotation_z(0.25),
        },
        flipper1: Flipper1,
    });
}
