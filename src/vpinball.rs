use crate::config::LaunchTemplate;
use std::io;
use std::path::PathBuf;
use std::process::{Child, ExitStatus, Stdio};

// TODO we probably want to check the log file for errors and launch completion
//   this would involve creating a struct that holds the process

pub(crate) enum OutputHandling {
    Show,
    Hide,
}

pub(crate) fn launch_table(
    selected_path: &PathBuf,
    launch_template: &LaunchTemplate,
    output: OutputHandling,
) -> io::Result<Child> {
    let mut cmd = std::process::Command::new(&launch_template.executable);
    if let Some(env) = &launch_template.env {
        for (key, value) in env.iter() {
            cmd.env(key, value);
        }
    }
    if let Some(args) = &launch_template.arguments {
        cmd.args(args);
    }
    cmd.arg("-play");
    cmd.arg(selected_path);

    match output {
        OutputHandling::Show => {
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());
        }
        OutputHandling::Hide => {
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
        }
    }

    println!("Spawning command: {:?}", cmd);

    cmd.spawn()
}

pub(crate) fn launch_table_wait(
    selected_path: &PathBuf,
    launch_template: &LaunchTemplate,
    output: OutputHandling,
) -> io::Result<ExitStatus> {
    let mut child = launch_table(selected_path, launch_template, output)?;
    child.wait()
}
