use bevy::prelude::*;
use bevy::window::WindowResized;

pub struct WindowEventLoggerPlugin;

impl Plugin for WindowEventLoggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (log_window_moved, log_window_resized));
    }
}

pub(crate) fn log_window_moved(mut events: EventReader<WindowMoved>) {
    for event in events.read() {
        println!(
            "Window moved to: x = {}, y = {}",
            event.position.x, event.position.y
        );
    }
}

pub(crate) fn log_window_resized(mut events: EventReader<WindowResized>) {
    for event in events.read() {
        println!(
            "Window resized to: width = {}, height = {}",
            event.width, event.height
        );
    }
}
