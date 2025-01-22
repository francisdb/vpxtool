use bevy::color::palettes::css;
use bevy::color::LinearRgba;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin};
use bevy::window::{PrimaryWindow, WindowResized};
use bevy_asset::Assets;

#[derive(Component)]
struct BackgroundGradient;

#[derive(Asset, AsBindGroup, TypePath, Clone)]
pub struct GradientMaterial {
    #[uniform(0)]
    pub color_start: LinearRgba,
    #[uniform(1)]
    pub color_end: LinearRgba,
}

impl Material2d for GradientMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/gradient_material.wgsl".into()
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_gradient_background);
    app.add_systems(Update, update_background_on_resize);
    app.add_plugins(Material2dPlugin::<GradientMaterial>::default());
}

fn update_background_on_resize(
    mut resize_events: EventReader<WindowResized>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<&Mesh2d, With<BackgroundGradient>>,
    window_query: Query<(Entity, &Window), With<PrimaryWindow>>,
) {
    for event in resize_events.read() {
        let (primary_window_entity, primary_window) = window_query.single();
        if event.window == primary_window_entity {
            let background_mesh = query.single();
            if let Some(mesh) = meshes.get_mut(background_mesh) {
                *mesh = window_mesh(primary_window);
            }
        }
    }
}

fn setup_gradient_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GradientMaterial>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = window_query.single();
    commands.spawn((
        BackgroundGradient,
        Mesh2d(meshes.add(window_mesh(primary_window))),
        MeshMaterial2d(materials.add(GradientMaterial {
            color_start: css::MIDNIGHT_BLUE.darker(0.02).into(),
            color_end: css::MIDNIGHT_BLUE.darker(0.015).into(),
        })),
        Transform::from_xyz(0.0, 0.0, -2.0),
    ));
}

fn window_mesh(primary_window: &Window) -> Mesh {
    Mesh::from(Rectangle::new(
        primary_window.width(),
        primary_window.height(),
    ))
}
