use crate::frontend::state::State;
use crate::frontend::Action;
use crate::indexer::IndexedTable;
use crate::info_gather;
use crate::simplefrontend::TableOption;
use colored::Colorize;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::{Color, Text};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{ListItem, Paragraph};
use vpin::vpx::tableinfo::TableInfo;
use vpin::vpx::version::Version;

pub fn update(state: &mut State, key_event: KeyEvent) -> Action {
    // always allow ctrl-c to quit
    if (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C'))
        && key_event.modifiers == KeyModifiers::CONTROL
    {
        return Action::Quit;
    }

    // give priority to the message dialog
    if let Some(_dialog) = &mut state.message_dialog {
        return match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                state.message_dialog = None;
                Action::None
            }
            _ => Action::None,
        };
    }

    // handle the table dialog
    match &mut state.table_dialog {
        Some(dialog) => match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                state.close_dialog();
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
