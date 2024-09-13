use crate::frontend::state::State;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn update(state: &mut State, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => state.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                state.quit()
            }
        }
        // KeyCode::Right | KeyCode::Char('j') => app.increment_counter(),
        // KeyCode::Left | KeyCode::Char('k') => app.decrement_counter(),
        KeyCode::Up => {
            state.tables.up(1);
        }
        KeyCode::Down => {
            state.tables.down(1);
        }
        KeyCode::PageUp | KeyCode::Left => {
            state.tables.up(10);
        }
        KeyCode::PageDown | KeyCode::Right => {
            state.tables.down(10);
        }
        KeyCode::Char('s') => {
            state.tables.switch_sort();
        }
        KeyCode::Enter => {

            // show dialog
        }
        _ => {}
    };
}
