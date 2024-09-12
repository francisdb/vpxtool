use crate::frontend::app::App;
use crate::indexer::IndexedTable;
use crate::simplefrontend::capitalize_first_letter;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::style::palette::tailwind::{AMBER, CYAN, NEUTRAL, SLATE, YELLOW};
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

const LIST_DIM_STYLE: Style = Style::new().fg(NEUTRAL.c500);
const LIST_SELECTED_STYLE: Style = Style::new()
    .bg(SLATE.c800)
    .fg(CYAN.c500)
    .add_modifier(Modifier::BOLD);

const INFO_ITEM_HEADER_STYLE: Style = Style::new().fg(CYAN.c500);

pub fn render(app: &mut App, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(f.area());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app.tables.items.iter().map(|i| ListItem::from(i)).collect();

    let items_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Tables");
    let tables = ratatui::widgets::List::new(items)
        .block(items_block)
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_style(LIST_SELECTED_STYLE);
    let tables_scrollbar = ratatui::widgets::Scrollbar::default().style(Style::default());

    let paragraph_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Table Info");
    let selected = app.tables.state.selected();
    let paragraph_text = match selected {
        Some(i) => {
            let table = &app.tables.items[i];
            table_to_paragraph(table, &app.roms)
        }
        None => Text::from("No table selected").style(Style::default().italic()),
    };
    let paragraph = Paragraph::new(paragraph_text)
        .wrap(Wrap { trim: true })
        .block(paragraph_block);

    f.render_stateful_widget(tables, chunks[0], &mut app.tables.state);
    f.render_stateful_widget(
        tables_scrollbar,
        chunks[0],
        &mut app.tables.vertical_scroll_state,
    );

    f.render_widget(paragraph, chunks[1]);
    //dialog(app, f);
}

fn table_to_paragraph<'a>(table: &IndexedTable, roms: &HashSet<String>) -> Text<'a> {
    // table name rendered as header
    // centered bold table name
    let table_name = table.displayed_name();
    let name_line = Line::from(table_name).style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(YELLOW.c200),
    );
    let name_text = Text::from(name_line);

    let warnings: Vec<Line> = table
        .warnings(roms)
        .iter()
        .map(|w| Line::styled(format!("⚠️ {}", w), Style::default().fg(AMBER.c500)))
        .collect();
    let warning_text = Text::from(warnings);

    let path_line = Span::from("Path:      ").style(INFO_ITEM_HEADER_STYLE)
        + Span::from(table.path.display().to_string());
    let game_name_line = table
        .game_name
        .clone()
        .map(|n| Span::from("Game Name: ").style(INFO_ITEM_HEADER_STYLE) + Span::from(n))
        .unwrap_or_default();
    let rom_line = table
        .local_rom_path
        .clone()
        .map(|p| {
            Span::from("Rom Path:  ").style(INFO_ITEM_HEADER_STYLE)
                + Span::from(p.display().to_string())
        })
        .unwrap_or_default();
    let b2s_line = table
        .b2s_path
        .clone()
        .map(|p| {
            Span::from("B2S Path:  ").style(INFO_ITEM_HEADER_STYLE)
                + Span::from(p.display().to_string())
        })
        .unwrap_or_default();

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
                    + Span::from(file_stem.clone()).style(LIST_DIM_STYLE)
            })
            .unwrap_or(Line::from(file_stem));
        ListItem::new(line)
    }
}

fn dialog(app: &mut App, f: &mut Frame) {
    let dialog_rect = centered_rect(f.area(), 50, 50);
    f.render_widget(Clear, dialog_rect);
    f.render_widget(
        Paragraph::new(format!(
            "
        Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
        Press `j` and `k` to increment and decrement the counter respectively.\n\
        Counter: {}
      ",
            app.counter
        ))
        .block(
            Block::default()
                .title("Counter App")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center),
        dialog_rect,
    )
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
