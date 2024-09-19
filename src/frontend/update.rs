use crate::frontend::state::State;
use crate::frontend::Action;
use crate::simplefrontend::TableOption;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn update(state: &mut State, key_event: KeyEvent) -> Action {
    match &mut state.table_dialog {
        Some(dialog) => match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                state.table_dialog = None;
                Action::None
            }
            KeyCode::Up => {
                dialog.items.select_previous();
                Action::None
            }
            KeyCode::Down => {
                dialog.items.select_next();
                Action::None
            }
            KeyCode::Enter => {
                let selected = dialog.items.selected().and_then(TableOption::from_index);
                if let Some(selected) = selected {
                    state.table_dialog = None;
                    Action::External(selected)
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        },
        None => {
            match &mut state.tables.filter {
                Some(filter) => match key_event.code {
                    //KeyCode::Enter => self.submit_message(),
                    KeyCode::Char(to_insert) => {
                        filter.enter_char(to_insert);
                        state.tables.apply_filter();
                        Action::None
                    }
                    KeyCode::Backspace => {
                        filter.delete_char();
                        state.tables.apply_filter();
                        Action::None
                    }
                    KeyCode::Esc => {
                        state.tables.disable_filter();
                        Action::None
                    }
                    KeyCode::Up => {
                        state.tables.up(1);
                        Action::None
                    }
                    KeyCode::Down => {
                        state.tables.down(1);
                        Action::None
                    }
                    KeyCode::PageUp | KeyCode::Left => {
                        state.tables.page_up();
                        Action::None
                    }
                    KeyCode::PageDown | KeyCode::Right => {
                        state.tables.page_down();
                        Action::None
                    }
                    KeyCode::Enter => {
                        state.open_dialog();
                        Action::None
                    }
                    _ => Action::None,
                },
                None => {
                    match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => Action::Quit,
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if key_event.modifiers == KeyModifiers::CONTROL {
                                Action::Quit
                            } else {
                                Action::None
                            }
                        }
                        // KeyCode::Right | KeyCode::Char('j') => app.increment_counter(),
                        // KeyCode::Left | KeyCode::Char('k') => app.decrement_counter(),
                        KeyCode::Up => {
                            state.tables.up(1);
                            Action::None
                        }
                        KeyCode::Down => {
                            state.tables.down(1);
                            Action::None
                        }
                        KeyCode::PageUp | KeyCode::Left => {
                            state.tables.page_up();
                            Action::None
                        }
                        KeyCode::PageDown | KeyCode::Right => {
                            state.tables.page_down();
                            Action::None
                        }
                        KeyCode::Char('s') => {
                            state.tables.switch_sort();
                            Action::None
                        }
                        KeyCode::Char('f') => {
                            state.tables.enable_filter();
                            Action::None
                        }
                        KeyCode::Enter => {
                            state.open_dialog();
                            Action::None
                        }
                        _ => Action::None,
                    }
                }
            }
        }
    }
}
