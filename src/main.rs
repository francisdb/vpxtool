use std::process::ExitCode;
use vpxtool::cli::run;
use vpxtool::fixprint;

fn main() -> ExitCode {
    // Windows consoles only show colors once virtual terminal processing is
    // on and print the escape codes otherwise; a console that cannot turn
    // it on gets no colors
    #[cfg(windows)]
    if colored::control::set_virtual_terminal(true).is_err() {
        colored::control::set_override(false);
    }
    fixprint::safe_main(run)
}
