use crate::indexer::IndexedTable;
use ratatui::widgets::{ListState, ScrollbarState};
use std::collections::HashSet;

/// Application.

#[derive(Debug, Default)]
pub struct App {
    /// should the application exit?
    pub should_quit: bool,
    /// counter
    pub counter: u8,
    #[deprecated(note = "The indexer should handle this rom check")]
    pub roms: HashSet<String>,
    pub tables: TableList,
}

#[derive(Debug, Default)]
pub struct TableList {
    pub items: Vec<IndexedTable>,
    pub state: ListState,
    pub vertical_scroll_state: ScrollbarState,
}

impl TableList {
    pub fn new(items: Vec<IndexedTable>) -> Self {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        let vertical_scroll_state = ScrollbarState::default().content_length(items.len());

        Self {
            items,
            state,
            vertical_scroll_state,
        }
    }

    pub fn down(&mut self, amount: usize) {
        let i = match self.state.selected() {
            Some(i) => (i + amount) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
        self.vertical_scroll_state = self.vertical_scroll_state.position(i);
    }

    pub fn up(&mut self, amount: usize) {
        let amount_capped = if amount > self.items.len() {
            amount % self.items.len()
        } else {
            amount
        };
        let i = match self.state.selected() {
            Some(i) => (i + self.items.len() - amount_capped) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
        self.vertical_scroll_state = self.vertical_scroll_state.position(i);
    }
}

impl App {
    pub fn new(roms: HashSet<String>, tables: Vec<IndexedTable>) -> Self {
        let tables = TableList::new(tables);
        Self {
            should_quit: false,
            counter: 0,
            roms,
            tables,
        }
    }

    /// Handles the tick event of the terminal.
    //pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }
}

mod tests {
    use super::*;
    use crate::indexer::{IndexedTableInfo, IsoSystemTime};
    use std::time::SystemTime;
    #[test]
    fn test_app_increment_counter() {
        let mut app = App::default();
        app.increment_counter();
        assert_eq!(app.counter, 1);
    }

    #[test]
    fn test_app_decrement_counter() {
        let mut app = App::default();
        app.decrement_counter();
        assert_eq!(app.counter, 0);
    }

    #[test]
    fn test_app_scroll_down() {
        let items = vec![test_table(), test_table(), test_table()];
        let mut app = App::new(Default::default(), items);
        assert_eq!(app.tables.state.selected(), Some(0));
        app.tables.down(1);
        assert_eq!(app.tables.state.selected(), Some(1));
    }

    #[test]
    fn test_app_scroll_up() {
        let items = vec![test_table(), test_table(), test_table()];
        let mut app = App::new(Default::default(), items);
        assert_eq!(app.tables.state.selected(), Some(0));
        app.tables.up(1);
        assert_eq!(app.tables.state.selected(), Some(2));
    }

    #[test]
    fn test_app_scroll_down_more_than_length() {
        let items = vec![test_table(), test_table(), test_table()];
        let mut app = App::new(Default::default(), items);
        assert_eq!(app.tables.state.selected(), Some(0));
        app.tables.down(10);
        assert_eq!(app.tables.state.selected(), Some(1));
    }

    #[test]
    fn test_app_scroll_up_more_than_length() {
        let items = vec![test_table(), test_table(), test_table()];
        let mut app = App::new(Default::default(), items);
        assert_eq!(app.tables.state.selected(), Some(0));
        app.tables.up(10);
        assert_eq!(app.tables.state.selected(), Some(2));
    }

    fn test_table() -> IndexedTable {
        IndexedTable {
            path: Default::default(),
            table_info: IndexedTableInfo {
                table_name: None,
                author_name: None,
                table_blurb: None,
                table_rules: None,
                author_email: None,
                release_date: None,
                table_save_rev: None,
                table_version: None,
                author_website: None,
                table_save_date: None,
                table_description: None,
                properties: Default::default(),
            },
            game_name: None,
            b2s_path: None,
            local_rom_path: None,
            requires_pinmame: false,
            last_modified: IsoSystemTime::from(SystemTime::now()),
        }
    }
}
