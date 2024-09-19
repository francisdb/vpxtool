use crate::frontend::state::{FilterState, State, TableActionsDialog, TablesSort};
use crate::indexer::IndexedTable;
use crate::simplefrontend::{capitalize_first_letter, TableOption};
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::*;
use ratatui::style::palette::tailwind::{AMBER, CYAN, SLATE};
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Clear, HighlightSpacing, ListItem, Wrap};
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use std::collections::HashSet;
use std::time::SystemTime;

const GRAY: Color = Color::Rgb(100, 100, 100);

const LIST_SELECTED_STYLE: Style = Style::new()
    .bg(SLATE.c800)
    .fg(CYAN.c500)
    .add_modifier(Modifier::BOLD);

const INFO_ITEM_HEADER_STYLE: Style = Style::new().fg(CYAN.c500);

const KEY_BINDING_STYLE: Style = Style::new().fg(AMBER.c500);

pub fn render(state: &mut State, f: &mut Frame) {
    let mut main_enabled = true;
    if state.table_dialog.is_some() {
        main_enabled = false;
    }

    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Fill(1), Constraint::Length(1)],
    )
    .direction(Direction::Vertical)
    .margin(1)
    .split(f.area());

    render_main(state, f, main_enabled, chunks[0]);

    if let Some(ref mut table_dialog) = state.table_dialog {
        let table = &state.tables.items[table_dialog.selected];
        render_action_dialog(table_dialog, f, table);
    }

    render_key_bindings(state, f, chunks[1]);
}

fn render_main(state: &mut State, f: &mut Frame, enabled: bool, area: Rect) {
    let [list_filter_aea, info_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .areas(area);

    // if filtering is enabled, render the filter above the list
    match state.tables.filter {
        Some(ref filter) => {
            let [filter_area, list_area] =
                Layout::vertical([Constraint::Length(3), Constraint::Fill(1)])
                    .areas(list_filter_aea);

            let input_enabled = state.table_dialog.is_none();
            render_filter(filter, input_enabled, f, filter_area);
            render_list(state, f, enabled, list_area);
        }
        None => {
            render_list(state, f, enabled, list_filter_aea);
        }
    }
    render_info(state, enabled, f, info_area);
}

fn render_list(state: &mut State, f: &mut Frame, enabled: bool, area: Rect) {
    let items: Vec<ListItem> = state
        .tables
        .filtered_items()
        .iter()
        .map(|i| ListItem::from(*i))
        .collect();

    let sorting = match state.tables.sort {
        TablesSort::Name => "Alphabetical",
        TablesSort::LastModified => "Last Modified",
    };
    let title =
        Span::from("Tables") + Span::from(format!(" ({}) ", sorting)).add_modifier(Modifier::DIM);
    let mut items_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title);
    if !enabled {
        items_block = items_block.dim();
    }
    let tables = ratatui::widgets::List::new(items)
        .block(items_block)
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_style(LIST_SELECTED_STYLE);
    let tables_scrollbar = ratatui::widgets::Scrollbar::default().style(Style::default());

    // Table List
    f.render_stateful_widget(tables, area, &mut state.tables.state);
    f.render_stateful_widget(
        tables_scrollbar,
        area,
        &mut state.tables.vertical_scroll_state,
    );
}

fn render_info(state: &State, enabled: bool, f: &mut Frame, area: Rect) {
    let mut paragraph_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Table Info");
    if !enabled {
        paragraph_block = paragraph_block.dim();
    }
    let paragraph_text = match state.tables.selected() {
        Some(table) => table_to_paragraph(table, &state.roms),
        None => Text::from("No table selected").style(Style::default().italic()),
    };
    let paragraph = Paragraph::new(paragraph_text)
        .wrap(Wrap { trim: true })
        .block(paragraph_block);
    f.render_widget(paragraph, area);
}

pub fn render_filter(state: &FilterState, enabled: bool, frame: &mut Frame, area: Rect) {
    let mut block = Block::default().title("Filter").borders(Borders::ALL);
    if !enabled {
        block = block.dim();
    }
    let paragraph = Paragraph::new(Line::from(state.input.clone()))
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);

    // Make the cursor visible and ask ratatui to put it at the specified coordinates after
    // rendering. Only when no dialog is open.
    if enabled {
        frame.set_cursor_position(Position::new(
            // Draw the cursor at the current position in the input field.
            // Move one to the right to account for the border.
            area.x + 1 + state.input.len() as u16,
            // Move one line down, from the border to the input line
            area.y + 1,
        ));
    }
}

/// Renders the key bindings.
pub fn render_key_bindings(state: &mut State, frame: &mut Frame, rect: Rect) {
    let chunks = Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)]).split(rect);
    let key_bindings = state.get_key_bindings();
    let line = Line::from(
        key_bindings
            .iter()
            .enumerate()
            .flat_map(|(i, (keys, desc))| {
                vec![
                    "[".fg(GRAY),
                    Span::from(*keys).style(KEY_BINDING_STYLE),
                    " → ".fg(GRAY),
                    Span::from(*desc),
                    "]".fg(GRAY),
                    if i != key_bindings.len() - 1 { " " } else { "" }.into(),
                ]
            })
            .collect::<Vec<Span>>(),
    );
    frame.render_widget(Paragraph::new(line.alignment(Alignment::Center)), chunks[1]);
}

fn table_to_paragraph<'a>(table: &IndexedTable, roms: &HashSet<String>) -> Text<'a> {
    // table name rendered as header
    // centered bold table name
    let table_name = table.displayed_name();
    let name_line = Line::from(table_name).style(Style::default().bold().fg(AMBER.c500));
    let name_text = Text::from(name_line);

    let warnings: Vec<Line> = table
        .warnings(roms)
        .iter()
        .map(|w| Line::styled(format!("⚠️ {}", w), Style::default().fg(AMBER.c500)))
        .collect();
    let warning_text = Text::from(warnings);

    let path_line = Span::from("Path:          ").style(INFO_ITEM_HEADER_STYLE)
        + Span::from(table.path.display().to_string());
    let game_name_line = table
        .game_name
        .clone()
        .map(|n| Span::from("Game Name:     ").style(INFO_ITEM_HEADER_STYLE) + Span::from(n))
        .unwrap_or_default();
    let rom_line = table
        .local_rom_path
        .clone()
        .map(|p| {
            Span::from("Rom Path:      ").style(INFO_ITEM_HEADER_STYLE)
                + Span::from(p.display().to_string())
        })
        .unwrap_or_default();
    let b2s_line = table
        .b2s_path
        .clone()
        .map(|p| {
            Span::from("B2S Path:      ").style(INFO_ITEM_HEADER_STYLE)
                + Span::from(p.display().to_string())
        })
        .unwrap_or_default();
    let f = timeago::Formatter::new();
    let time: SystemTime = table.last_modified.into();
    let duration = time.elapsed().unwrap();
    let last_modified_human_readable = f.convert(duration);
    let last_modified_line = Span::from("Last Modified: ").style(INFO_ITEM_HEADER_STYLE)
        + Span::from(last_modified_human_readable);

    let description = table
        .table_info
        .table_description
        .clone()
        .unwrap_or_default();
    name_text
        + Line::from("")
        + warning_text
        + Text::from(path_line)
        + game_name_line
        + rom_line
        + b2s_line
        + last_modified_line
        + Line::from("")
        + Text::from(description)
}

impl From<&IndexedTable> for ListItem<'_> {
    fn from(table: &IndexedTable) -> Self {
        let file_stem = table
            .path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let line = Some(table.table_info.table_name.to_owned())
            .filter(|s| !s.clone().unwrap_or_default().is_empty())
            .map(|s| {
                Span::from(capitalize_first_letter(s.unwrap_or_default().as_str()))
                    + Span::from(" ")
                    + Span::from(file_stem.clone()).add_modifier(Modifier::DIM)
            })
            .unwrap_or(Line::from(file_stem));
        ListItem::new(line)
    }
}

fn render_action_dialog(state: &mut TableActionsDialog, frame: &mut Frame, table: &IndexedTable) {
    let area = frame.area();
    let block = Block::bordered().title(table.displayed_name());
    let area = popup_area(area, 50, 60);
    frame.render_widget(Clear, area); //this clears out the background
    frame.render_widget(block, area);

    // add a list of actions and a scrollbar
    let actions = TableOption::ALL
        .iter()
        .map(|action| ListItem::new(Span::from(action.display())))
        .collect::<Vec<_>>();
    let actions = ratatui::widgets::List::new(actions)
        .block(Block::default().borders(Borders::ALL).title("Actions"))
        .highlight_symbol("> ")
        .highlight_style(LIST_SELECTED_STYLE);
    let actions_scrollbar = ratatui::widgets::Scrollbar::default().style(Style::default());
    frame.render_stateful_widget(actions, area, &mut state.items);
    frame.render_stateful_widget(actions_scrollbar, area, &mut state.vertical_scroll_state);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, width: u16, /*percent_x: u16,*/ percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
