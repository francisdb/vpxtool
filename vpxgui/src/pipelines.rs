use bevy::{prelude::*, render::render_resource::*, render::*};

// TODO why is this here?
// seems to be available as https://github.com/rparrett/bevy_pipelines_ready

pub struct PipelinesReadyPlugin;
impl Plugin for PipelinesReadyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PipelinesReady::default());

        // In order to gain access to the pipelines status, we have to
        // go into the `RenderApp`, grab the resource from the main App
        // and then update the pipelines status from there.
        // Writing between these Apps can only be done through the
        // `ExtractSchedule`.
        app.sub_app_mut(bevy::render::RenderApp)
            .add_systems(ExtractSchedule, update_pipelines_ready);
    }
}

#[derive(Resource, Debug, Default)]
pub struct PipelinesReady(pub bool);

fn update_pipelines_ready(mut main_world: ResMut<MainWorld>, pipelines: Res<PipelineCache>) {
    if let Some(mut pipelines_ready) = main_world.get_resource_mut::<PipelinesReady>() {
        pipelines_ready.0 = pipelines.waiting_pipelines().count() == 0;
    }
}
