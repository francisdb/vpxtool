use shared::indexer::VoidProgress;
use shared::{config, indexer};
use std::io;
use std::process::ExitCode;
use vpxgui::guifrontend;

fn main() -> ExitCode {
    run().unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        ExitCode::FAILURE
    })
}

fn run() -> io::Result<ExitCode> {
    if let Some((_, resolved_config)) = config::load_config()? {
        let recursive = true;
        let index = indexer::index_folder(
            recursive,
            &resolved_config.tables_folder,
            &resolved_config.tables_index_path,
            Some(&resolved_config.global_pinmame_rom_folder()),
            &VoidProgress,
            Vec::new(),
        )?;
        // TODO we want to run the indexer once the frontend has started and report progress in the frontend
        guifrontend::guifrontend(resolved_config.clone(), index.tables());
        Ok(ExitCode::SUCCESS)
    } else {
        let warning = "No config file found. Run vpxtool to create one.";
        eprintln!("{}", warning);
        Ok(ExitCode::FAILURE)
    }
}
