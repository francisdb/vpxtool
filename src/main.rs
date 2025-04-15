use std::process::ExitCode;
use vpxtool::cli::run;
use vpxtool::fixprint;

fn main() -> ExitCode {
    fixprint::safe_main(run)
}
