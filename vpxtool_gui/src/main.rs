use std::io;
use std::process::ExitCode;
use vpxtool_gui::guifrontend;
use vpxtool_shared::config;

fn main() -> ExitCode {
    run().unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        ExitCode::FAILURE
    })
}

fn run() -> io::Result<ExitCode> {
    if let Some((_, resolved_config)) = config::load_config()? {
        // TODO we want to run the indexer once the frontend has started and report progress in the frontend
        guifrontend::guifrontend(resolved_config.clone());
        Ok(ExitCode::SUCCESS)
    } else {
        let warning = "No config file found. Run vpxtool to create one.";
        eprintln!("{}", warning);
        Ok(ExitCode::FAILURE)
    }
}
