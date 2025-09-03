// Patched version of dialoguer::theme::ColorfulTheme
//
// This extends the colorful theme to partially fix a bug with the fuzzy select prompt rendering
// where the ANSI escape codes were not being handled correctly.
//
// see https://github.com/console-rs/dialoguer/issues/312

use dialoguer::theme::{ColorfulTheme, Theme};
use std::fmt;

#[derive(Default)]
pub struct ColorfulThemePatched {
    inner: ColorfulTheme,
}

impl Theme for ColorfulThemePatched {
    fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.inner.format_prompt(f, prompt)
    }

    fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
        self.inner.format_error(f, err)
    }

    fn format_confirm_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<bool>,
    ) -> fmt::Result {
        self.inner.format_confirm_prompt(f, prompt, default)
    }

    fn format_confirm_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selection: Option<bool>,
    ) -> fmt::Result {
        self.inner
            .format_confirm_prompt_selection(f, prompt, selection)
    }

    fn format_input_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<&str>,
    ) -> fmt::Result {
        self.inner.format_input_prompt(f, prompt, default)
    }

    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selection: &str,
    ) -> fmt::Result {
        self.inner
            .format_input_prompt_selection(f, prompt, selection)
    }

    fn format_password_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.inner.format_password_prompt(f, prompt)
    }

    fn format_password_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
    ) -> fmt::Result {
        self.inner.format_password_prompt_selection(f, prompt)
    }

    fn format_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.inner.format_select_prompt(f, prompt)
    }

    fn format_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selection: &str,
    ) -> fmt::Result {
        self.inner
            .format_select_prompt_selection(f, prompt, selection)
    }

    fn format_multi_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.inner.format_multi_select_prompt(f, prompt)
    }

    fn format_sort_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.inner.format_sort_prompt(f, prompt)
    }

    fn format_multi_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        self.inner
            .format_multi_select_prompt_selection(f, prompt, selections)
    }

    fn format_sort_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        self.inner
            .format_sort_prompt_selection(f, prompt, selections)
    }

    fn format_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
    ) -> fmt::Result {
        self.inner.format_select_prompt_item(f, text, active)
    }

    fn format_multi_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        checked: bool,
        active: bool,
    ) -> fmt::Result {
        self.inner
            .format_multi_select_prompt_item(f, text, checked, active)
    }

    fn format_sort_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        picked: bool,
        active: bool,
    ) -> fmt::Result {
        self.inner.format_sort_prompt_item(f, text, picked, active)
    }

    fn format_fuzzy_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
        highlight_matches: bool,
        matcher: &fuzzy_matcher::skim::SkimMatcherV2,
        search_term: &str,
    ) -> fmt::Result {
        if !active {
            return self.inner.format_fuzzy_select_prompt_item(
                f,
                text,
                active,
                highlight_matches,
                matcher,
                search_term,
            );
        }
        // see https://github.com/console-rs/dialoguer/pull/336
        // we do not support highlight_matches=true here as it's broken anyway
        write!(f, "{} ", &self.inner.active_item_prefix)?;
        write!(f, "{}", self.inner.active_item_style.apply_to(text))
    }

    fn format_fuzzy_select_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        search_term: &str,
        cursor_pos: usize,
    ) -> fmt::Result {
        self.inner
            .format_fuzzy_select_prompt(f, prompt, search_term, cursor_pos)
    }
}
