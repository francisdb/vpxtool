//! Terminal progress reporting through the `OSC 9;4` escape sequence.
//!
//! Terminals that understand it (Konsole, Windows Terminal, WezTerm, Ghostty,
//! ConEmu, Ptyxis) show the progress on the tab and/or the taskbar entry, the
//! way `dnf` does. Other terminals may misinterpret `OSC 9` (iTerm2 used it
//! for notifications), so the sequence is only sent when a supporting
//! terminal is detected.

use std::io::{IsTerminal, Write, stdout};
use std::sync::atomic::{AtomicU64, Ordering};

const NO_VALUE: u64 = u64::MAX;

pub(crate) struct TermProgress {
    enabled: bool,
    last_percent: AtomicU64,
}

impl TermProgress {
    pub(crate) fn detect() -> Self {
        Self {
            enabled: stdout().is_terminal() && terminal_supports_progress(),
            last_percent: AtomicU64::new(NO_VALUE),
        }
    }

    /// Reports `pos` out of `len`, only writing when the percentage changes.
    pub(crate) fn set(&self, pos: u64, len: u64) {
        if !self.enabled || len == 0 {
            return;
        }
        let percent = (pos.min(len) * 100) / len;
        if self.last_percent.swap(percent, Ordering::Relaxed) != percent {
            write_sequence(&format!("1;{percent}"));
        }
    }

    /// Removes the progress indicator.
    pub(crate) fn clear(&self) {
        if self.enabled && self.last_percent.swap(NO_VALUE, Ordering::Relaxed) != NO_VALUE {
            write_sequence("0");
        }
    }
}

impl Drop for TermProgress {
    fn drop(&mut self) {
        self.clear();
    }
}

fn write_sequence(state: &str) {
    let mut out = stdout().lock();
    // Ignore errors: this is decoration, a closed stdout is handled elsewhere.
    let _ = write!(out, "\x1b]9;4;{state}\x1b\\");
    let _ = out.flush();
}

fn terminal_supports_progress() -> bool {
    let env = |name: &str| std::env::var(name).ok();
    env("KONSOLE_VERSION").is_some()
        || env("WT_SESSION").is_some()
        || env("PTYXIS_VERSION").is_some()
        || env("ConEmuANSI").as_deref() == Some("ON")
        || matches!(env("TERM_PROGRAM").as_deref(), Some("WezTerm" | "ghostty"))
}
