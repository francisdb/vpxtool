//! List menus for the text frontend, with keys that can be configured.
//!
//! They draw through the dialoguer theme, so they look the same as the prompts
//! that still come from dialoguer.

use console::{Key, Term, strip_ansi_codes, style, truncate_str};
use dialoguer::theme::{ColorfulTheme, Theme};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::{fmt, io};

/// What a key can do in a menu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    PageUp,
    PageDown,
    Select,
    Toggle,
    ToggleAll,
    Back,
    Search,
}

impl Action {
    fn name(self) -> &'static str {
        match self {
            Action::Up => "up",
            Action::Down => "down",
            Action::PageUp => "page_up",
            Action::PageDown => "page_down",
            Action::Select => "select",
            Action::Toggle => "toggle",
            Action::ToggleAll => "toggle_all",
            Action::Back => "back",
            Action::Search => "search",
        }
    }
}

/// The `[frontend.keys]` table of the config file.
/// An action that is left out keeps its default keys.
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MenuKeys {
    pub up: Option<Vec<String>>,
    pub down: Option<Vec<String>>,
    pub page_up: Option<Vec<String>>,
    pub page_down: Option<Vec<String>>,
    pub select: Option<Vec<String>>,
    pub toggle: Option<Vec<String>>,
    pub toggle_all: Option<Vec<String>>,
    pub back: Option<Vec<String>>,
    pub search: Option<Vec<String>>,
}

/// An action with its configured keys and its default keys
type ActionKeys<'a> = (Action, &'a Option<Vec<String>>, &'a [&'a str]);

/// Validated key bindings, a key has at most one action
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMap {
    bindings: Vec<(Key, Action)>,
}

impl Default for KeyMap {
    fn default() -> Self {
        KeyMap::new(&MenuKeys::default()).expect("the default keys are valid")
    }
}

impl KeyMap {
    /// Applies the configured keys on top of the defaults. Fails on an unknown
    /// key name, an action without keys or a key bound to two actions.
    pub fn new(keys: &MenuKeys) -> Result<Self, String> {
        let actions: [ActionKeys; 9] = [
            (Action::Up, &keys.up, &["up", "k", "backtab"]),
            (Action::Down, &keys.down, &["down", "j", "tab"]),
            (Action::PageUp, &keys.page_up, &["left", "h"]),
            (Action::PageDown, &keys.page_down, &["right", "l"]),
            (Action::Select, &keys.select, &["enter"]),
            (Action::Toggle, &keys.toggle, &["space"]),
            (Action::ToggleAll, &keys.toggle_all, &["a"]),
            (Action::Back, &keys.back, &["esc", "q"]),
            (Action::Search, &keys.search, &["/"]),
        ];
        let mut bindings: Vec<(Key, Action)> = Vec::new();
        for (action, configured, defaults) in actions {
            let names: Vec<&str> = match configured {
                Some(names) => names.iter().map(String::as_str).collect(),
                None => defaults.to_vec(),
            };
            if names.is_empty() {
                return Err(format!(
                    "frontend key `{}` needs at least one key",
                    action.name()
                ));
            }
            for name in names {
                let key = parse_key(name).ok_or_else(|| {
                    format!(
                        "unknown key \"{name}\" for frontend key `{}`",
                        action.name()
                    )
                })?;
                match bindings.iter().find(|(bound, _)| *bound == key) {
                    Some((_, other)) if *other != action => {
                        return Err(format!(
                            "key \"{name}\" is bound to both `{}` and `{}`",
                            other.name(),
                            action.name()
                        ));
                    }
                    Some(_) => {}
                    None => bindings.push((key, action)),
                }
            }
        }
        Ok(KeyMap { bindings })
    }

    fn action(&self, key: &Key) -> Option<Action> {
        self.bindings
            .iter()
            .find(|(bound, _)| bound == key)
            .map(|(_, action)| *action)
    }

    /// The keys of an action for showing in a hint, like `Esc/q/Backspace`
    pub fn label(&self, action: Action) -> String {
        self.bindings
            .iter()
            .filter(|(_, bound)| *bound == action)
            .map(|(key, _)| key_name(key))
            .collect::<Vec<_>>()
            .join("/")
    }
}

fn parse_key(name: &str) -> Option<Key> {
    let mut chars = name.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        return Some(Key::Char(c));
    }
    Some(match name.to_ascii_lowercase().as_str() {
        "up" => Key::ArrowUp,
        "down" => Key::ArrowDown,
        "left" => Key::ArrowLeft,
        "right" => Key::ArrowRight,
        "enter" => Key::Enter,
        "esc" | "escape" => Key::Escape,
        "backspace" => Key::Backspace,
        "tab" => Key::Tab,
        "backtab" => Key::BackTab,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "del" | "delete" => Key::Del,
        "insert" => Key::Insert,
        "space" => Key::Char(' '),
        _ => return None,
    })
}

fn key_name(key: &Key) -> String {
    match key {
        Key::ArrowUp => "up".to_string(),
        Key::ArrowDown => "down".to_string(),
        Key::ArrowLeft => "left".to_string(),
        Key::ArrowRight => "right".to_string(),
        Key::Enter => "enter".to_string(),
        Key::Escape => "esc".to_string(),
        Key::Backspace => "backspace".to_string(),
        Key::Tab => "tab".to_string(),
        Key::BackTab => "backtab".to_string(),
        Key::Home => "home".to_string(),
        Key::End => "end".to_string(),
        Key::PageUp => "pageup".to_string(),
        Key::PageDown => "pagedown".to_string(),
        Key::Del => "del".to_string(),
        Key::Insert => "insert".to_string(),
        Key::Char(' ') => "space".to_string(),
        Key::Char(c) => c.to_string(),
        other => format!("{other:?}"),
    }
}

/// How the user left a [`select`] menu
#[derive(Debug, PartialEq, Eq)]
pub enum Selection {
    /// The index of the chosen item
    Picked(usize),
    Back,
    /// The search was requested, with the index the cursor was on
    Search(usize),
}

/// A menu that picks one item. The `toggle` keys also pick here.
pub fn select(
    keys: &KeyMap,
    prompt: &str,
    items: &[impl AsRef<str>],
    default: usize,
) -> io::Result<Selection> {
    if items.is_empty() {
        return Ok(Selection::Back);
    }
    let theme = ColorfulTheme::default();
    let mut screen = Screen::new()?;
    let mut cursor = default.min(items.len() - 1);
    let hint = style(format!("({} to search)", keys.label(Action::Search))).dim();
    let prompt_with_hint = format!("{prompt} {hint}");
    loop {
        let capacity = screen.capacity();
        let page = page_for(cursor, items.len(), capacity);
        let mut lines = vec![heading(&page, |f| {
            theme.format_select_prompt(f, &prompt_with_hint)
        })?];
        for (index, item) in items.iter().enumerate().take(page.end).skip(page.start) {
            let mut line = String::new();
            theme
                .format_select_prompt_item(
                    &mut line,
                    &row_text(item.as_ref(), index == cursor),
                    index == cursor,
                )
                .map_err(io::Error::other)?;
            lines.push(line);
        }
        screen.draw(&lines)?;

        match keys.action(&screen.term.read_key()?) {
            Some(Action::Select | Action::Toggle) => {
                screen.clear()?;
                let mut line = String::new();
                theme
                    .format_select_prompt_selection(&mut line, prompt, items[cursor].as_ref())
                    .map_err(io::Error::other)?;
                screen.term.write_line(&line)?;
                return Ok(Selection::Picked(cursor));
            }
            Some(Action::Back) => {
                screen.clear()?;
                return Ok(Selection::Back);
            }
            Some(Action::Search) => {
                screen.clear()?;
                return Ok(Selection::Search(cursor));
            }
            Some(movement @ (Action::Up | Action::Down | Action::PageUp | Action::PageDown)) => {
                cursor = step(cursor, items.len(), capacity, movement);
            }
            Some(Action::ToggleAll) | None => {}
        }
    }
}

/// A menu that turns items on and off. Returns the indices that are on,
/// or `None` when the user backed out.
pub fn multi_select(
    keys: &KeyMap,
    prompt: &str,
    items: &[impl AsRef<str>],
    defaults: &[bool],
) -> io::Result<Option<Vec<usize>>> {
    if items.is_empty() {
        return Ok(None);
    }
    let theme = ColorfulTheme::default();
    let mut screen = Screen::new()?;
    let mut checked: Vec<bool> = (0..items.len())
        .map(|index| defaults.get(index).copied().unwrap_or(false))
        .collect();
    let mut cursor = 0;
    loop {
        let capacity = screen.capacity();
        let page = page_for(cursor, items.len(), capacity);
        let mut lines = vec![heading(&page, |f| {
            theme.format_multi_select_prompt(f, prompt)
        })?];
        for (index, item) in items.iter().enumerate().take(page.end).skip(page.start) {
            let mut line = String::new();
            theme
                .format_multi_select_prompt_item(
                    &mut line,
                    item.as_ref(),
                    checked[index],
                    index == cursor,
                )
                .map_err(io::Error::other)?;
            lines.push(line);
        }
        screen.draw(&lines)?;

        match keys.action(&screen.term.read_key()?) {
            Some(Action::Toggle) => checked[cursor] = !checked[cursor],
            Some(Action::ToggleAll) => {
                let on = !checked.iter().all(|c| *c);
                checked.iter_mut().for_each(|c| *c = on);
            }
            Some(Action::Select) => {
                screen.clear()?;
                let picked: Vec<usize> = (0..items.len()).filter(|i| checked[*i]).collect();
                let names: Vec<&str> = picked.iter().map(|i| items[*i].as_ref()).collect();
                let mut line = String::new();
                theme
                    .format_multi_select_prompt_selection(&mut line, prompt, &names)
                    .map_err(io::Error::other)?;
                screen.term.write_line(&line)?;
                return Ok(Some(picked));
            }
            Some(Action::Back) => {
                screen.clear()?;
                return Ok(None);
            }
            Some(movement @ (Action::Up | Action::Down | Action::PageUp | Action::PageDown)) => {
                cursor = step(cursor, items.len(), capacity, movement);
            }
            Some(Action::Search) | None => {}
        }
    }
}

fn heading(page: &Page, write: impl FnOnce(&mut String) -> fmt::Result) -> io::Result<String> {
    let mut line = String::new();
    write(&mut line).map_err(io::Error::other)?;
    if page.count > 1 {
        line.push_str(&format!(" [Page {}/{}]", page.number, page.count));
    }
    Ok(line)
}

/// The gray that table lines use after the table name
const GRAY: &str = "\x1b[90m";
/// A dimmed version of the color of the selected row
const DIM_SELECTED: &str = "\x1b[2;36m";

/// On the selected row gray text becomes a dimmed version of the row color
fn row_text(text: &str, selected: bool) -> String {
    if selected {
        text.replace(GRAY, DIM_SELECTED)
    } else {
        text.to_string()
    }
}

/// The text and position of a [`search`]
#[derive(Default)]
pub struct SearchState {
    input: TextInput,
    /// Nothing is selected until the user types or moves
    cursor: Option<usize>,
}

/// A fuzzy search over the items. Items are cut off to one line for display
/// but the whole text is matched. Returns the index of the chosen item, or
/// `None` when the user backed out.
///
/// The configured keys do not apply: letters and Backspace edit the search.
///
/// Passing the same `state` again reopens the search as it was left.
pub fn search(
    prompt: &str,
    items: &[impl AsRef<str>],
    state: &mut SearchState,
) -> io::Result<Option<usize>> {
    let theme = ColorfulTheme::default();
    let matcher = SkimMatcherV2::default();
    let plain: Vec<String> = items
        .iter()
        .map(|item| strip_ansi_codes(item.as_ref()).into_owned())
        .collect();
    let mut screen = Screen::new()?;
    loop {
        let text = state.input.text();
        let matches = ranked(&matcher, &plain, &text);
        let capacity = screen.capacity();
        let page = page_for(state.cursor.unwrap_or(0), matches.len(), capacity);
        let mut lines = vec![heading(&page, |f| {
            theme.format_fuzzy_select_prompt(f, prompt, &text, state.input.byte_position())
        })?];
        for (position, index) in matches.iter().enumerate().take(page.end).skip(page.start) {
            let mut line = String::new();
            theme
                .format_select_prompt_item(
                    &mut line,
                    &row_text(items[*index].as_ref(), Some(position) == state.cursor),
                    Some(position) == state.cursor,
                )
                .map_err(io::Error::other)?;
            lines.push(line);
        }
        screen.draw(&lines)?;

        match screen.term.read_key()? {
            Key::Escape => {
                screen.clear()?;
                return Ok(None);
            }
            Key::Enter => {
                if let Some(index) = state.cursor.and_then(|position| matches.get(position)) {
                    screen.clear()?;
                    return Ok(Some(*index));
                }
            }
            Key::ArrowUp | Key::BackTab if !matches.is_empty() => {
                state.cursor = Some(match state.cursor {
                    Some(position) => step(position, matches.len(), capacity, Action::Up),
                    None => matches.len() - 1,
                });
            }
            Key::ArrowDown | Key::Tab if !matches.is_empty() => {
                state.cursor = Some(match state.cursor {
                    Some(position) => step(position, matches.len(), capacity, Action::Down),
                    None => 0,
                });
            }
            key => {
                if state.input.apply(&key) {
                    state.cursor = Some(0);
                }
            }
        }
    }
}

/// The indices of the matching items, best match first.
/// Equal scores keep their order in the list.
fn ranked(matcher: &SkimMatcherV2, items: &[String], term: &str) -> Vec<usize> {
    let mut scored: Vec<(usize, i64)> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| matcher.fuzzy_match(item, term).map(|score| (index, score)))
        .collect();
    scored.sort_by_key(|(_, score)| Reverse(*score));
    scored.into_iter().map(|(index, _)| index).collect()
}

/// The text of a search prompt with its editing position
#[derive(Default)]
struct TextInput {
    chars: Vec<char>,
    position: usize,
}

impl TextInput {
    fn text(&self) -> String {
        self.chars.iter().collect()
    }

    fn byte_position(&self) -> usize {
        self.chars[..self.position]
            .iter()
            .map(|c| c.len_utf8())
            .sum()
    }

    /// Returns true when the text changed
    fn apply(&mut self, key: &Key) -> bool {
        match key {
            Key::Char(c) if !c.is_control() => {
                self.chars.insert(self.position, *c);
                self.position += 1;
                true
            }
            Key::Backspace if self.position > 0 => {
                self.position -= 1;
                self.chars.remove(self.position);
                true
            }
            Key::Del if self.position < self.chars.len() => {
                self.chars.remove(self.position);
                true
            }
            Key::ArrowLeft => {
                self.position = self.position.saturating_sub(1);
                false
            }
            Key::ArrowRight => {
                self.position = (self.position + 1).min(self.chars.len());
                false
            }
            Key::Home => {
                self.position = 0;
                false
            }
            Key::End => {
                self.position = self.chars.len();
                false
            }
            _ => false,
        }
    }
}

/// The visible part of a list
#[derive(Debug, PartialEq, Eq)]
struct Page {
    start: usize,
    end: usize,
    /// Starts at 1
    number: usize,
    count: usize,
}

fn page_for(cursor: usize, len: usize, capacity: usize) -> Page {
    let index = cursor / capacity;
    let start = index * capacity;
    Page {
        start,
        end: (start + capacity).min(len),
        number: index + 1,
        count: len.div_ceil(capacity),
    }
}

/// The cursor after a movement, moves wrap around at the ends of the list
fn step(cursor: usize, len: usize, capacity: usize, movement: Action) -> usize {
    let pages = len.div_ceil(capacity);
    let page = cursor / capacity;
    match movement {
        Action::Up => (cursor + len - 1) % len,
        Action::Down => (cursor + 1) % len,
        Action::PageUp if pages > 1 => (page + pages - 1) % pages * capacity,
        Action::PageDown if pages > 1 => (page + 1) % pages * capacity,
        _ => cursor,
    }
}

/// Redraws a block of lines in place and restores the cursor when dropped
struct Screen {
    term: Term,
    height: usize,
    size: (u16, u16),
}

impl Screen {
    fn new() -> io::Result<Self> {
        let term = Term::stderr();
        if !term.is_term() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "not a terminal",
            ));
        }
        term.hide_cursor()?;
        let size = term.size();
        Ok(Screen {
            term,
            height: 0,
            size,
        })
    }

    /// How many items fit, leaving room for the prompt and the cursor line
    fn capacity(&self) -> usize {
        (self.term.size().0 as usize).max(3) - 2
    }

    fn draw(&mut self, lines: &[String]) -> io::Result<()> {
        let size = self.term.size();
        if size != self.size && self.height > 0 {
            // a resize reflows what we drew, so the line count is no longer right
            self.term.clear_screen()?;
            self.height = 0;
        }
        self.size = size;
        self.clear()?;
        // lines that wrap would break the redraw
        let width = (size.1 as usize).saturating_sub(1);
        for line in lines {
            let line = line.replace('\n', " ");
            self.term.write_line(&truncate_str(&line, width, "..."))?;
        }
        self.height = lines.len();
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        if self.height > 0 {
            self.term.clear_last_lines(self.height)?;
            self.height = 0;
        }
        Ok(())
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        self.term.show_cursor().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn keys(names: &[&str]) -> Option<Vec<String>> {
        Some(names.iter().map(|name| name.to_string()).collect())
    }

    #[test]
    fn default_keys_keep_the_dialoguer_bindings_and_add_search() {
        let map = KeyMap::default();
        for (key, action) in [
            (Key::Escape, Action::Back),
            (Key::Char('q'), Action::Back),
            (Key::Char('/'), Action::Search),
            (Key::Enter, Action::Select),
            (Key::Char(' '), Action::Toggle),
            (Key::Tab, Action::Down),
            (Key::BackTab, Action::Up),
            (Key::ArrowLeft, Action::PageUp),
            (Key::Char('l'), Action::PageDown),
        ] {
            assert_eq!(map.action(&key), Some(action), "{key:?}");
        }
        assert_eq!(map.action(&Key::Char('x')), None);
        assert_eq!(map.action(&Key::Backspace), None);
    }

    #[test]
    fn configured_keys_replace_only_that_action() {
        let map = KeyMap::new(&MenuKeys {
            back: keys(&["Del", "x"]),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(map.action(&Key::Del), Some(Action::Back));
        assert_eq!(map.action(&Key::Char('x')), Some(Action::Back));
        assert_eq!(map.action(&Key::Escape), None);
        assert_eq!(map.action(&Key::ArrowUp), Some(Action::Up));
        assert_eq!(map.label(Action::Back), "del/x");
    }

    #[test]
    fn a_key_on_two_actions_is_an_error() {
        let result = KeyMap::new(&MenuKeys {
            search: keys(&["q"]),
            ..Default::default()
        });
        assert_eq!(
            result,
            Err("key \"q\" is bound to both `back` and `search`".to_string())
        );
    }

    #[test]
    fn unknown_key_name_is_an_error() {
        let result = KeyMap::new(&MenuKeys {
            up: keys(&["Upp"]),
            ..Default::default()
        });
        assert_eq!(
            result,
            Err("unknown key \"Upp\" for frontend key `up`".to_string())
        );
    }

    #[test]
    fn action_without_keys_is_an_error() {
        let result = KeyMap::new(&MenuKeys {
            select: Some(vec![]),
            ..Default::default()
        });
        assert_eq!(
            result,
            Err("frontend key `select` needs at least one key".to_string())
        );
    }

    #[test]
    fn key_names_are_case_insensitive_but_single_characters_are_not() {
        assert_eq!(parse_key("BACKSPACE"), Some(Key::Backspace));
        assert_eq!(parse_key("q"), Some(Key::Char('q')));
        assert_eq!(parse_key("Q"), Some(Key::Char('Q')));
    }

    #[test]
    fn unknown_action_in_the_config_is_rejected() {
        assert!(toml::from_str::<MenuKeys>("bak = [\"q\"]").is_err());
    }

    #[test]
    fn search_ranks_matches_and_keeps_list_order_for_ties() {
        let matcher = SkimMatcherV2::default();
        let items: Vec<String> = ["Attack from Mars", "Medieval Madness", "Mars God of War"]
            .map(String::from)
            .to_vec();
        assert_eq!(ranked(&matcher, &items, ""), vec![0, 1, 2]);
        assert_eq!(ranked(&matcher, &items, "medieval"), vec![1]);
        assert_eq!(ranked(&matcher, &items, "zzz"), Vec::<usize>::new());
    }

    #[test]
    fn search_text_edits_at_the_position() {
        let mut input = TextInput::default();
        assert!(input.apply(&Key::Char('a')));
        assert!(input.apply(&Key::Char('é')));
        assert!(input.apply(&Key::Char('c')));
        assert!(!input.apply(&Key::ArrowLeft));
        assert_eq!(input.byte_position(), 3);
        assert!(input.apply(&Key::Backspace));
        assert_eq!(input.text(), "ac");
        assert!(input.apply(&Key::Del));
        assert_eq!(input.text(), "a");
        assert!(!input.apply(&Key::Del));
        assert!(!input.apply(&Key::ArrowRight));
        assert!(!input.apply(&Key::Home));
        assert!(!input.apply(&Key::Backspace));
        assert_eq!(input.text(), "a");
    }

    #[test]
    fn gray_text_is_only_replaced_on_the_selected_row() {
        let text = "name \x1b[90m(file)\x1b[0m \x1b[33m!\x1b[0m";
        assert_eq!(row_text(text, false), text);
        assert_eq!(
            row_text(text, true),
            "name \x1b[2;36m(file)\x1b[0m \x1b[33m!\x1b[0m"
        );
    }

    #[test]
    fn up_and_down_wrap_around() {
        assert_eq!(step(0, 5, 10, Action::Up), 4);
        assert_eq!(step(4, 5, 10, Action::Down), 0);
    }

    #[test]
    fn paging_moves_to_the_start_of_the_page_and_wraps() {
        assert_eq!(step(3, 25, 10, Action::PageDown), 10);
        assert_eq!(step(23, 25, 10, Action::PageDown), 0);
        assert_eq!(step(3, 25, 10, Action::PageUp), 20);
    }

    #[test]
    fn paging_does_nothing_when_everything_fits() {
        assert_eq!(step(3, 5, 10, Action::PageDown), 3);
        assert_eq!(step(3, 5, 10, Action::PageUp), 3);
    }

    #[test]
    fn last_page_is_cut_off_at_the_list_end() {
        assert_eq!(
            page_for(23, 25, 10),
            Page {
                start: 20,
                end: 25,
                number: 3,
                count: 3
            }
        );
    }
}
