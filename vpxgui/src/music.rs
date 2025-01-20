use bevy::prelude::*;

#[derive(Event)]
pub enum ControlMusicEvent {
    Suspend,
    Resume,
}

#[derive(Resource, Default)]
struct MusicState {
    play_on_resume: bool,
}

pub(crate) fn music_plugin(app: &mut App) {
    app.add_event::<ControlMusicEvent>();
    app.insert_resource(MusicState::default());
    app.add_systems(Startup, music_startup);
    app.add_systems(Update, (music_update, volume_update, handle_music_events));
}

pub(crate) fn suspend_music(event_writer: &mut EventWriter<ControlMusicEvent>) {
    event_writer.send(ControlMusicEvent::Suspend);
}

pub(crate) fn resume_music(event_writer: &mut EventWriter<ControlMusicEvent>) {
    event_writer.send(ControlMusicEvent::Resume);
}

fn music_startup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("Pinball.ogg")),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            paused: true,
            ..Default::default()
        },
    ));
}

fn music_update(music_box_query: Query<&AudioSink>, keys: Res<ButtonInput<KeyCode>>) {
    if let Ok(sink) = music_box_query.get_single() {
        if keys.just_pressed(KeyCode::KeyM) {
            if sink.is_paused() {
                info!("Playing music");
                sink.play()
            } else {
                info!("Pausing music");
                sink.pause()
            }
        }
    }
}

fn volume_update(keys: Res<ButtonInput<KeyCode>>, music_box_query: Query<&AudioSink>) {
    if let Ok(sink) = music_box_query.get_single() {
        if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
            sink.set_volume(sink.volume() + 0.1);
        } else if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
            sink.set_volume(sink.volume() - 0.1);
        }
    }
}

fn handle_music_events(
    mut music_events: EventReader<ControlMusicEvent>,
    mut state: ResMut<MusicState>,
    music_box_query: Query<&AudioSink>,
) {
    if let Ok(sink) = music_box_query.get_single() {
        for event in music_events.read() {
            match event {
                ControlMusicEvent::Suspend => {
                    state.play_on_resume = !sink.is_paused();
                    if !sink.is_paused() {
                        info!("Suspending music");
                        sink.pause();
                    }
                }
                ControlMusicEvent::Resume => {
                    if state.play_on_resume {
                        info!("Resuming music");
                        sink.play();
                    }
                }
            }
        }
    }
}
