use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::tonemapping::Tonemapping;
/// A DMD Shader that renders first to a texture
/// and then renders the texture with a DMD shader.
///
/// https://www.airtightinteractive.com/demos/js/ledeffect/
///
/// TODO we need a multi-pass shader to add glow to the resulting dmd texture.
/// https://github.com/freezy/dmd-extensions/blob/82cbba59048ac11b158f35457e5a3cdfd0073c1e/LibDmd/Output/Network/www/main.js#L194
/// https://github.com/freezy/dmd-extensions/blob/82cbba59048ac11b158f35457e5a3cdfd0073c1e/LibDmd/Output/Network/www/shaders.js#L187
///
///
///
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::view::RenderLayers;
use bevy::sprite::{Material2d, Material2dPlugin};
use bevy_asset::RenderAssetUsages;

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct FirstPassCube;

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct DotMatrixShader {
    // Uniform bindings must implement `ShaderType`, which will be used to convert the value to
    // its shader-compatible equivalent. Most core math types already implement `ShaderType`.
    /// eg 128, 32
    #[uniform(0)]
    dimension: Vec2,

    /// eg 1280, 320
    #[uniform(1)]
    resolution: Vec2,

    // Images can be bound as textures in shaders. If the Image's sampler is also needed, just
    // add the sampler attribute with a different binding index.
    #[texture(2)]
    #[sampler(3)]
    color_texture: Handle<Image>,
}

// All functions on `Material2d` have default impls. You only need to implement the
// functions that are relevant for your material.
impl Material2d for DotMatrixShader {
    fn fragment_shader() -> ShaderRef {
        "shaders/dot_matrix_shader.wgsl".into()
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<DotMatrixShader>::default())
        .add_systems(Update, rotator_system)
        .add_systems(Startup, setup);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut materials2: ResMut<Assets<DotMatrixShader>>,
    mut images: ResMut<Assets<Image>>,
) {
    // TODO add this and try to come up with something realistic
    //   https://bevyengine.org/examples/2d-rendering/bloom-2d/
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true, // 1. HDR is required for bloom
            ..default()
        },
        Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
        Bloom {
            intensity: 0.1,
            low_frequency_boost: 0.0,

            ..default()
        }, // 3. Enable bloom for the camera
    ));
    let size = Extent3d {
        width: 1280,
        height: 320,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    let cube_handle = meshes.add(Cuboid::new(5.0, 5.0, 5.0));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.7, 0.6),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    let first_pass_layer = RenderLayers::layer(1);

    // The cube that will be rendered to the texture.
    commands.spawn((
        Mesh3d(cube_handle),
        MeshMaterial3d(cube_material_handle),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        FirstPassCube,
        first_pass_layer.clone(),
    ));

    // Light
    // NOTE: we add the light to both layers so it affects both the rendered-to-texture cube, and the cube on which we display the texture
    // Setting the layer to RenderLayers::layer(0) would cause the main view to be lit, but the rendered-to-texture cube to be unlit.
    // Setting the layer to RenderLayers::layer(1) would cause the rendered-to-texture cube to be lit, but the main view to be unlit.
    commands.spawn((
        PointLight::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        RenderLayers::layer(1),
    ));

    commands.spawn((
        Camera3d::default(),
        Camera {
            target: image_handle.clone().into(),
            clear_color: Color::linear_rgb(0.02, 0.02, 0.02).into(),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)).looking_at(Vec3::ZERO, Vec3::Y),
        first_pass_layer,
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(
            size.width as f32 / 2.0,
            size.height as f32 / 2.0,
        ))),
        MeshMaterial2d(materials2.add(DotMatrixShader {
            resolution: Vec2::new(size.width as f32, size.height as f32),
            dimension: Vec2::new(128.0, 32.0),
            color_texture: image_handle,
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, 512.0)),
    ));
}

/// Rotates the inner cube (first pass)
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<FirstPassCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(1.5 * time.delta_secs());
        transform.rotate_z(1.3 * time.delta_secs());
    }
}
