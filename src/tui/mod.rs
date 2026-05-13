/// Application.
pub mod state;

/// Terminal events handler.
pub mod event;

/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod terminal;

/// Application updater.
pub mod update;

use crate::cli::{info_diff, info_edit, info_gather};
use crate::config::ResolvedConfig;
use crate::frontend::{TableOption, launch, prompt};
use crate::indexer::IndexedTable;
use anyhow::Result;
use colored::Colorize;
use event::{Event, EventHandler};
use ratatui::backend::CrosstermBackend;
use state::State;
use std::collections::HashSet;
use std::io::{Write, stdin};
use terminal::Tui;
use update::update;
use vpin::vpx::{ExtractResult, extractvbs};

type Terminal = ratatui::Terminal<CrosstermBackend<std::io::Stderr>>;

pub enum Action {
    External(TableOption),
    Quit,
    None,
}

pub fn main(config: ResolvedConfig, items: Vec<IndexedTable>, roms: HashSet<String>) -> Result<()> {
    let mut state = State::new(config, roms, items);

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    run(&mut state, &mut tui)?;

    tui.exit()?;
    Ok(())
}

fn run(state: &mut State, tui: &mut Tui) -> Result<()> {
    loop {
        tui.draw(state)?;
        let action = match tui.events.next()? {
            Event::Tick => Action::None,
            Event::Key(key_event) => update(state, key_event),
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
                let selected_path = selected.path.clone();
                match table_action {
                    TableOption::Launch { template } => run_external(tui, || {
                        launch(&selected_path, &template);
                        Ok(())
                    })?,
                    TableOption::InfoShow => run_external(tui, || {
                        let output = info_gather(&selected_path)?;
                        let mut less = std::process::Command::new("less")
                            .stdin(std::process::Stdio::piped())
                            .spawn()?;
                        let mut stdin = less.stdin.take().unwrap();
                        stdin.write_all(output.as_bytes())?;
                        drop(stdin);
                        less.wait()?;
                        Ok(())
                    })?,
                    TableOption::InfoEdit => {
                        let config = state.config.clone();
                        run_external(tui, || {
                            match info_edit(&selected_path, Some(&config)) {
                                Ok(path) => {
                                    println!("Launched editor for {}", path.display());
                                }
                                Err(err) => {
                                    let msg = format!("Unable to edit table info: {err}");
                                    prompt(&msg.truecolor(255, 125, 0).to_string());
                                }
                            }
                            Ok(())
                        })?
                    }
                    TableOption::InfoDiff => {
                        let config = state.config.clone();
                        run_external(tui, || {
                            match info_diff(&selected_path, Some(&config)) {
                                Ok(diff) => prompt(&diff),
                                Err(err) => {
                                    let msg = format!("Unable to diff info: {err}");
                                    prompt(&msg.truecolor(255, 125, 0).to_string());
                                }
                            };
                            Ok(())
                        })?
                    }
                    TableOption::ExtractVBS => {
                        let _guard = TuiGuard::new(tui)?;
                        match extractvbs(&selected_path, None, false) {
                            Ok(ExtractResult::Extracted(path)) => {
                                state.prompt_info(format!(
                                    "VBS extracted to {}",
                                    path.to_string_lossy()
                                ));
                            }
                            Ok(ExtractResult::Existed(path)) => {
                                state.prompt_warning(format!(
                                    "VBS already exists at {}",
                                    path.to_string_lossy()
                                ));
                            }
                            Err(err) => {
                                state.prompt_error(format!("Unable to extract VBS: {err}"));
                            }
                        }
                    }
                    not_implemented => run_external(tui, || {
                        eprintln!(
                            "Action not implemented yet in ratatui frontend: {}. Press enter to continue.",
                            not_implemented.display()
                        );
                        let _ = stdin().read_line(&mut String::new())?;
                        Ok(())
                    })?,
                };
            } else {
                unreachable!("At this point, a table should be selected.");
            }
            Ok(false)
        }
        Action::Quit => Ok(true),
        Action::None => Ok(false),
    }
}

struct TuiGuard<'a> {
    tui: &'a mut Tui,
}

impl<'a> TuiGuard<'a> {
    fn new(tui: &'a mut Tui) -> Result<Self> {
        tui.disable()?;
        Ok(Self { tui })
    }
}

impl Drop for TuiGuard<'_> {
    fn drop(&mut self) {
        if let Err(err) = self.tui.enable() {
            eprintln!("Failed to re-enable TUI: {err}");
        }
    }
}

fn run_external<T>(tui: &mut Tui, run: impl Fn() -> Result<T>) -> Result<T> {
    tui.disable()?;
    let result = run();
    tui.enable()?;
    result
}
