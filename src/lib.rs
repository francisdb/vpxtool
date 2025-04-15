mod backglass;
pub mod fixprint;
mod frontend;
pub mod patcher;

pub mod config;

pub mod indexer;

pub mod cli;
pub mod vpinball_config;

pub(crate) fn strip_cr_lf(s: &str) -> String {
    s.chars().filter(|c| !c.is_ascii_whitespace()).collect()
}
