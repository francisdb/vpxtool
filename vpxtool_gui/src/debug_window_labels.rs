use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::{WindowCreated, WindowRef};

#[derive(Component)]
struct LabelMarker;

#[derive(Resource, Clone, Default)]
pub(crate) struct WindowNameOptions {
    pub enabled: bool,
}

/// Plugin that adds a label to each window with the window's name
pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<WindowNameOptions>();
    app.add_systems(Update, (add_label_to_new_windows, switch_visibility));
}

fn switch_visibility(
    options: Res<WindowNameOptions>,
    mut visibility_query: Query<&mut Visibility, With<LabelMarker>>,
) {
    for mut visibility in visibility_query.iter_mut() {
        *visibility = match options.enabled {
            true => Visibility::Visible,
            false => Visibility::Hidden,
        };
    }
}

fn add_label_to_new_windows(
    mut created_events: EventReader<WindowCreated>,
    mut commands: Commands,
    window_query: Query<(Entity, &Window)>,
    camera_query: Query<(Entity, &Camera)>,
) {
    for event in created_events.read() {
        info!("Window created: {:?}", event);
        let (_, window) = window_query.get(event.window).unwrap();
        // find the camera entity for the window
        //camera {
        //    target: RenderTarget::Window(

        let window_camera_entity = camera_query
            .iter()
            .find(|(_, camera)| {
                if let RenderTarget::Window(WindowRef::Entity(window_entity)) = camera.target {
                    window_entity == event.window
                } else {
                    false
                }
            })
            .map(|(entity, _)| entity);

        // if there is no camera for the window this must be the primary window
        let window_camera_entity = window_camera_entity.unwrap_or_else(|| {
            camera_query
                .iter()
                .find(|(_, camera)| {
                    matches!(camera.target, RenderTarget::Window(WindowRef::Primary))
                })
                .map(|(entity, _)| entity)
                .unwrap()
        });

        label_window(&mut commands, window_camera_entity, window);
    }
}

fn label_window(commands: &mut Commands, window_camera_entity: Entity, window: &Window) {
    let window_label_node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(12.0),
        left: Val::Px(12.0),
        ..default()
    };
    let name = window.name.clone().unwrap_or("Unnamed Window".to_string());
    commands.spawn((
        Text::new(name),
        TextFont::from_font_size(8.0),
        window_label_node,
        TargetCamera(window_camera_entity),
        LabelMarker,
    ));
}
