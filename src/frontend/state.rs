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
    /// Dialog to display messages to the user.
    /// It can be a warning or an error.
    /// It will be displayed until the user dismisses it.
    /// It will be displayed on top of everything else.
    pub message_dialog: Option<MessageDialog>,
}

#[derive(Debug)]
pub enum MessageDialog {
    Info(String),
    Warning(String),
    Error(String),
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
    pub filter: Option<FilterState>,
}

#[derive(Debug, Default)]
pub struct FilterState {
    /// Current value of the input box
    pub input: String,
}

impl FilterState {
    pub(crate) fn delete_char(&mut self) {
        self.input.pop();
    }
}

impl FilterState {
    pub(crate) fn enter_char(&mut self, c: char) {
        self.input.push(c);
    }
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
            filter: None,
        }
    }

    pub(crate) fn enable_filter(&mut self) {
        // in filtering mode only alphabetical sorting makes sense
        self.sort_by_name();
        self.filter = Some(Default::default());
    }

    pub(crate) fn disable_filter(&mut self) {
        self.filter = None;
        self.apply_filter();
    }

    pub fn filtered_items(&self) -> Vec<&IndexedTable> {
        match &self.filter {
            Some(filter) => {
                let input = filter.input.to_lowercase();
                self.items
                    .iter()
                    .filter(|t| t.displayed_name().to_lowercase().contains(&input))
                    .collect()
            }
            None => self.items.iter().collect(),
        }
    }

    fn filtered_len(&self) -> usize {
        match &self.filter {
            Some(filter) => {
                let input = filter.input.to_lowercase();
                self.items
                    .iter()
                    .filter(|t| t.displayed_name().to_lowercase().contains(&input))
                    .count()
            }
            None => self.items.len(),
        }
    }

    pub fn selected(&self) -> Option<&IndexedTable> {
        self.state.selected().map(|i| self.filtered_items()[i])
    }

    pub(crate) fn apply_filter(&mut self) {
        // update the list state
        self.state.select_first();
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(self.filtered_items().len());
        self.sync_list_scoll();
    }

    pub fn down(&mut self, amount: usize) {
        let filtered_len = self.filtered_len();
        let i = match self.state.selected() {
            Some(i) => (i + amount) % filtered_len,
            None => 0,
        };
        self.state.select(Some(i));
        self.sync_list_scoll();
    }

    pub fn up(&mut self, amount: usize) {
        let filtered_len = self.filtered_len();
        let amount_capped = if amount > filtered_len {
            amount % filtered_len
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

    pub(crate) fn page_up(&mut self) {
        self.up(10)
    }

    pub(crate) fn page_down(&mut self) {
        self.down(10)
    }

    pub fn switch_sort(&mut self) {
        match self.sort {
            TablesSort::Name => {
                self.sort_by_lastmodified();
            }
            TablesSort::LastModified => {
                self.sort_by_name();
            }
        }
    }

    fn select_first(&mut self) {
        self.state.select_first();
        self.sync_list_scoll();
    }

    fn sort_by_lastmodified(&mut self) {
        self.sort = TablesSort::LastModified;
        self.items
            .sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        self.select_first();
    }

    fn sort_by_name(&mut self) {
        self.sort = TablesSort::Name;
        self.items.sort_by_key(|a| a.displayed_name());
        self.select_first();
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
            message_dialog: None,
        }
    }

    /// Handles the tick event of the terminal.
    //pub fn tick(&self) {}

    /// Returns the key bindings.
    pub fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        if let Some(_) = self.message_dialog {
            return vec![("⏎", "Dismiss")];
        }
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

    pub(crate) fn close_dialog(&mut self) {
        self.table_dialog = None;
    }

    pub(crate) fn prompt_info(&mut self, message: String) {
        self.message_dialog = Some(MessageDialog::Info(message));
    }

    pub(crate) fn prompt_warning(&mut self, message: String) {
        self.message_dialog = Some(MessageDialog::Warning(message));
    }

    pub(crate) fn prompt_error(&mut self, message: String) {
        self.message_dialog = Some(MessageDialog::Error(message));
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
