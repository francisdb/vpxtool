use crate::frontend::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit()
            }
        }
        // KeyCode::Right | KeyCode::Char('j') => app.increment_counter(),
        // KeyCode::Left | KeyCode::Char('k') => app.decrement_counter(),
        KeyCode::Up => {
            app.tables.up(1);
        }
        KeyCode::Down => {
            app.tables.down(1);
        }
        KeyCode::PageUp | KeyCode::Left => {
            app.tables.up(10);
        }
        KeyCode::PageDown | KeyCode::Right => {
            app.tables.down(10);
        }
        KeyCode::Enter => {

            // show dialog
        }
        _ => {}
    };
}
