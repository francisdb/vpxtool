use std::process::ExitCode;
use vpxtool::{fixprint, run};

fn main() -> ExitCode {
    fixprint::safe_main(run)
}
