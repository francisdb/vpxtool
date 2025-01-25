use crate::loading::LoadingState;
use crate::wheel::WheelInfo;
use bevy::math::{Quat, Vec3};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_asset::AssetServer;

#[derive(Component)]
struct FlipperLeft;

#[derive(Component)]
struct FlipperRight;

#[derive(Bundle)]
struct FlipperBundle {
    sprite: Sprite,
    transform: Transform,
    visibility: Visibility,
}

pub(crate) fn flipper_plugin(app: &mut App) {
    app.add_systems(Startup, create_flippers);
    app.add_systems(
        Update,
        update_flippers.run_if(in_state(LoadingState::Ready)),
    );
}

const Z_LEVEL: f32 = -1.;

#[allow(clippy::type_complexity)]
fn update_flippers(
    mut set: ParamSet<(
        Query<(&mut Transform, &mut Visibility), With<FlipperLeft>>,
        Query<(&mut Transform, &mut Visibility), With<FlipperRight>>,
    )>,
    wheel_info: Res<WheelInfo>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    let height = window.height();

    for (mut transform, mut visibility) in set.p0().iter_mut() {
        transform.translation = Vec3::new(
            (wheel_info.wheel_size / 2.) * -1.0,
            (-(height / 2.)) + (wheel_info.wheel_size / 4.),
            Z_LEVEL,
        );
        *visibility = Visibility::Visible;
    }

    // TODO why should flippers get closer to each other when the window height is reduced?
    for (mut transform, mut visibility) in set.p1().iter_mut() {
        transform.translation = Vec3::new(
            wheel_info.wheel_size / 2.,
            (-(height / 2.0)) + (wheel_info.wheel_size / 4.),
            Z_LEVEL,
        );
        *visibility = Visibility::Visible;
    }
}

pub(crate) fn create_flippers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    let window_width = window.width();
    let window_height = window.height();

    commands.spawn((
        FlipperBundle {
            sprite: Sprite {
                image: asset_server.load("left-flipper.png"),
                ..default()
            },
            visibility: Visibility::Hidden,

            transform: Transform {
                translation: Vec3::new(
                    window_width - (window_width * 0.60) - 225.,
                    (window_height * 0.25) + 60.,
                    Z_LEVEL,
                ),
                scale: (Vec3::new(0.5, 0.5, 1.0)),
                rotation: Quat::from_rotation_z(-0.25),
            },
        },
        FlipperLeft,
    ));

    commands.spawn((
        FlipperBundle {
            sprite: Sprite {
                image: asset_server.load("right-flipper.png"),
                ..default()
            },
            visibility: Visibility::Hidden,

            transform: Transform {
                translation: Vec3::new(
                    window_width - (window_width * 0.60),
                    window_height * 0.25 + 60.,
                    Z_LEVEL,
                ),
                scale: (Vec3::new(0.5, 0.5, 1.0)),
                rotation: Quat::from_rotation_z(0.25),
            },
        },
        FlipperRight,
    ));
}
