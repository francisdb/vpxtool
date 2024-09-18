use crate::config::ResolvedConfig;
use crate::indexer::IndexedTable;
use ratatui::widgets::{ListState, ScrollbarState};
use std::collections::HashSet;

#[derive(Debug)]
pub struct State {
    pub config: ResolvedConfig,
    pub roms: HashSet<String>,
    pub tables: TableList,
    pub table_dialog: Option<TableActionsDialog>,
}

#[derive(Debug, Default)]
pub enum TablesSort {
    #[default]
    Name,
    LastModified,
}

#[derive(Debug, Default)]
pub struct TableList {
    pub items: Vec<IndexedTable>,
    pub state: ListState,
    pub vertical_scroll_state: ScrollbarState,
    pub sort: TablesSort,
}

#[derive(Debug, Default)]
pub struct TableActionsDialog {
    pub items: ListState,
    pub vertical_scroll_state: ScrollbarState,
    pub selected: usize,
}

impl TableActionsDialog {
    pub fn new(selected: usize) -> Self {
        Self {
            items: ListState::default(),
            vertical_scroll_state: ScrollbarState::default(),
            selected,
        }
    }
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
            sort: TablesSort::Name,
        }
    }

    pub fn down(&mut self, amount: usize) {
        let i = match self.state.selected() {
            Some(i) => (i + amount) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
        self.sync_list_scoll();
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
        self.sync_list_scoll();
    }

    pub fn switch_sort(&mut self) {
        match self.sort {
            TablesSort::Name => {
                self.sort = TablesSort::LastModified;
                self.items
                    .sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
            }
            TablesSort::LastModified => {
                self.sort = TablesSort::Name;
                self.items.sort_by_key(|a| a.displayed_name());
            }
        }
        self.state.select_first();
        self.sync_list_scoll();
    }

    fn sync_list_scoll(&mut self) {
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .position(self.state.selected().unwrap_or(0));
    }
}

impl State {
    pub fn new(
        resolved_config: ResolvedConfig,
        roms: HashSet<String>,
        tables: Vec<IndexedTable>,
    ) -> Self {
        let tables = TableList::new(tables);
        Self {
            config: resolved_config,
            roms,
            tables,
            table_dialog: None,
        }
    }

    /// Handles the tick event of the terminal.
    //pub fn tick(&self) {}

    /// Returns the key bindings.
    pub fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        match self.table_dialog {
            Some(_) => vec![("⏎", "Select"), ("↑↓", "Navigate"), ("q/esc", "Back")],
            None => vec![
                ("⏎", "Table actions"),
                ("↑↓", "Select"),
                ("←→", "Scroll"),
                ("s", "Sort"),
                ("f", "Filter"),
                ("q/esc", "Quit"),
            ],
        }
    }

    pub(crate) fn open_dialog(&mut self) {
        if let Some(selected) = self.tables.state.selected() {
            let mut dialog = TableActionsDialog::new(selected);
            dialog.items.select(Some(0));
            self.table_dialog = Some(dialog);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::{IndexedTableInfo, IsoSystemTime};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn test_config() -> ResolvedConfig {
        ResolvedConfig {
            vpx_executable: PathBuf::from("vpinball"),
            tables_folder: Default::default(),
            tables_index_path: Default::default(),
            editor: None,
        }
    }

    #[test]
    fn test_app_scroll_down() {
        let items = vec![test_table(), test_table(), test_table()];
        let mut app = State::new(test_config(), Default::default(), items);
        assert_eq!(app.tables.state.selected(), Some(0));
        app.tables.down(1);
        assert_eq!(app.tables.state.selected(), Some(1));
    }

    #[test]
    fn test_app_scroll_up() {
        let items = vec![test_table(), test_table(), test_table()];
        let mut app = State::new(test_config(), Default::default(), items);
        assert_eq!(app.tables.state.selected(), Some(0));
        app.tables.up(1);
        assert_eq!(app.tables.state.selected(), Some(2));
    }

    #[test]
    fn test_app_scroll_down_more_than_length() {
        let items = vec![test_table(), test_table(), test_table()];
        let mut app = State::new(test_config(), Default::default(), items);
        assert_eq!(app.tables.state.selected(), Some(0));
        app.tables.down(10);
        assert_eq!(app.tables.state.selected(), Some(1));
    }

    #[test]
    fn test_app_scroll_up_more_than_length() {
        let items = vec![test_table(), test_table(), test_table()];
        let mut app = State::new(test_config(), Default::default(), items);
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
