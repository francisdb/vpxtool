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
use crate::simplefrontend::{launch, prompt, TableOption};
use crate::{info_diff, info_edit, write_info_json};
use anyhow::Result;
use colored::Colorize;
use event::{Event, EventHandler};
use ratatui::backend::CrosstermBackend;
use state::State;
use std::collections::HashSet;
use std::io::{stdin, Cursor, Write};
use tui::Tui;
use update::update;
use vpin::vpx::{extractvbs, ExtractResult};

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
                    TableOption::InfoShow => run_external(tui, || {
                        // echo pipe to less
                        let mut memory_file = Cursor::new(Vec::new());
                        write_info_json(selected_path, &mut memory_file)?;
                        let output = memory_file.into_inner();
                        // execute less with the data piped in
                        let mut less = std::process::Command::new("less")
                            .stdin(std::process::Stdio::piped())
                            .spawn()?;
                        let mut stdin = less.stdin.take().unwrap();
                        stdin.write_all(&output)?;
                        // wait for less to finish
                        less.wait()?;
                        Ok(())
                    }),
                    TableOption::InfoEdit => run_external(tui, || {
                        let config = Some(&state.config);
                        match info_edit(selected_path, config) {
                            Ok(path) => {
                                println!("Launched editor for {}", path.display());
                            }
                            Err(err) => {
                                let msg = format!("Unable to edit table info: {}", err);
                                prompt(msg.truecolor(255, 125, 0).to_string());
                            }
                        }
                        Ok(())
                    }),
                    TableOption::InfoDiff => run_external(tui, || {
                        match info_diff(selected_path) {
                            Ok(diff) => {
                                prompt(diff);
                            }
                            Err(err) => {
                                let msg = format!("Unable to diff info: {}", err);
                                prompt(msg.truecolor(255, 125, 0).to_string());
                            }
                        };
                        Ok(())
                    }),
                    TableOption::ExtractVBS => {
                        // TODO is this guard thing a good idea? I prefer the closure approach
                        //   but we got some issues with the borrow checker
                        let _guard = TuiGuard::new(tui)?;
                        match extractvbs(selected_path, false, None) {
                            Ok(ExtractResult::Extracted(path)) => {
                                state.prompt_info(format!(
                                    "VBS extracted to {}",
                                    path.to_string_lossy()
                                ));
                            }
                            Ok(ExtractResult::Existed(path)) => {
                                let msg =
                                    format!("VBS already exists at {}", path.to_string_lossy());
                                state.prompt_warning(msg);
                            }
                            Err(err) => {
                                let msg = format!("Unable to extract VBS: {}", err);
                                state.prompt_error(msg);
                            }
                        }
                        Ok(())
                    }
                    TableOption::EditVBS => run_external(tui, || {
                        let config = Some(&state.config);
                        match info_edit(selected_path, config) {
                            Ok(path) => {
                                println!("Launched editor for {}", path.display());
                            }
                            Err(err) => {
                                let msg = format!("Unable to edit table info: {}", err);
                                prompt(msg.truecolor(255, 125, 0).to_string());
                            }
                        }
                        Ok(())
                    }),
                    TableOption::PatchVBS => run_external(tui, || Ok(())),
                    TableOption::UnifyLineEndings => run_external(tui, || Ok(())),
                    TableOption::ShowVBSDiff => run_external(tui, || Ok(())),
                    TableOption::CreateVBSPatch => run_external(tui, || Ok(())),
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

struct TuiGuard<'a> {
    tui: &'a mut Tui,
}

impl<'a> TuiGuard<'a> {
    fn new(tui: &'a mut Tui) -> Result<Self> {
        tui.disable()?;
        Ok(Self { tui })
    }
}

impl<'a> Drop for TuiGuard<'a> {
    fn drop(&mut self) {
        if let Err(err) = self.tui.enable() {
            eprintln!("Failed to re-enable TUI: {}", err);
        }
    }
}

fn run_external<T>(tui: &mut Tui, run: impl Fn() -> Result<T>) -> Result<T> {
    tui.disable()?;
    let result = run();
    tui.enable()?;
    result
}
