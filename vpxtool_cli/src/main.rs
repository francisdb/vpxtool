use std::process::ExitCode;
use vpxtool_cli::{fixprint, run};

fn main() -> ExitCode {
    fixprint::safe_main(run)
}
