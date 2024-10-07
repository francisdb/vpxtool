/// Application.
pub mod state;

/// Terminal events handler.
pub mod event;

/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod tui;

/// Application updater.
pub mod update;

use crate::config::ResolvedConfig;
use crate::indexer::IndexedTable;
use crate::simplefrontend::{launch, TableOption};
use anyhow::Result;
use event::{Event, EventHandler};
use ratatui::backend::CrosstermBackend;
use state::State;
use std::collections::HashSet;
use std::io::stdin;
use tui::Tui;
use update::update;

type Terminal = ratatui::Terminal<CrosstermBackend<std::io::Stderr>>;

pub enum Action {
    External(TableOption),
    Quit,
    None,
}

pub fn main(config: ResolvedConfig, items: Vec<IndexedTable>, roms: HashSet<String>) -> Result<()> {
    // Create an application.
    let mut state = State::new(config, roms, items);

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    run(&mut state, &mut tui)?;

    // Exit the user interface.
    tui.exit()?;
    // TODO is this the same or better?
    //ratatui::restore();
    Ok(())
}

fn run(state: &mut State, tui: &mut Tui) -> Result<()> {
    loop {
        // Render the user interface.
        tui.draw(state)?;
        // Handle events.
        let action = match tui.events.next()? {
            Event::Tick => Action::None,
            Event::Key(key_event) => update(state, key_event),
            // Event::Mouse(_) => {}
            // Event::Resize(_, _) => {}
        };
        let done = run_action(state, tui, action)?;
        if done {
            break;
        }
    }
    Ok(())
}

fn run_action(state: &mut State, tui: &mut Tui, action: Action) -> Result<bool> {
    match action {
        Action::External(table_action) => {
            if let Some(selected) = state.tables.selected() {
                let selected_path = &selected.path;
                let vpinball_executable = &state.config.vpx_executable;
                match table_action {
                    TableOption::Launch => run_external(tui, || {
                        launch(selected_path, vpinball_executable, None);
                        Ok(())
                    }),
                    TableOption::LaunchFullscreen => run_external(tui, || {
                        launch(selected_path, vpinball_executable, Some(true));
                        Ok(())
                    }),
                    TableOption::LaunchWindowed => run_external(tui, || {
                        launch(selected_path, vpinball_executable, Some(false));
                        Ok(())
                    }),
                    not_implemented => run_external(tui, || {
                        eprintln!(
                            "Action not implemented: {:?}. Press enter to continue.",
                            not_implemented.display()
                        );
                        // read line
                        let _ = stdin().read_line(&mut String::new())?;
                        Ok(())
                    }),
                }?;
            } else {
                unreachable!("At this point, a table should be selected.");
            }
            Ok(false)
        }
        Action::Quit => Ok(true),
        Action::None => Ok(false),
    }
}

fn run_external<T>(tui: &mut Tui, run: impl Fn() -> Result<T>) -> Result<T> {
    // TODO most of this stuff is duplicated in Tui
    tui.disable()?;
    let result = run();
    tui.enable()?;
    result
}
