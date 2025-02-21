// manages background vpinball process

use bevy::prelude::*;
use crossbeam_channel::Sender;
use is_executable::IsExecutable;
use std::path::Path;
use std::process::{ExitStatus, exit};

use crate::event_channel::ChannelExternalEvent;
use std::{io, thread};

pub(crate) fn do_launch(tx: Sender<ChannelExternalEvent>, path: &Path, executable: &Path) {
    info!("Launching table {}", path.display());
    let tx = tx.clone();
    let path = path.to_path_buf();
    let executable = executable.to_path_buf();

    let _vpinball_thread = thread::spawn(move || {
        launch(&path, &executable, None);
        info!("Vpinball done, sending event");
        tx.send(ChannelExternalEvent::VpxDone).unwrap();
    });
}

fn launch(selected_path: &Path, vpinball_executable: &Path, fullscreen: Option<bool>) {
    if !vpinball_executable.is_executable() {
        report_and_exit(format!(
            "Unable to launch table, {} is not executable",
            vpinball_executable.display()
        ));
    }

    match launch_table(selected_path, vpinball_executable, fullscreen) {
        Ok(status) => match status.code() {
            Some(0) => {
                //println!("Table exited normally");
            }
            Some(11) => {
                eprintln!(
                    "Visual Pinball exited with segfault, you might want to report this to the vpinball team."
                );
            }
            Some(139) => {
                eprintln!(
                    "Visual Pinball exited with segfault, you might want to report this to the vpinball team."
                );
            }
            Some(code) => {
                eprintln!("Visual Pinball exited with code {}", code);
            }
            None => {
                eprintln!("Visual Pinball exited with unknown code");
            }
        },
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                report_and_exit(format!(
                    "Unable to launch table, vpinball executable not found at {}",
                    vpinball_executable.display()
                ));
            } else {
                report_and_exit(format!("Unable to launch table: {:?}", e));
            }
        }
    }
}

fn report_and_exit(msg: String) -> ! {
    eprintln!("CRASH {}", msg);
    exit(1);
}

fn launch_table(
    selected_path: &Path,
    vpinball_executable: &Path,
    fullscreen: Option<bool>,
) -> io::Result<ExitStatus> {
    // start process ./VPinballX_GL -play [table path]
    let mut cmd = std::process::Command::new(vpinball_executable);
    match fullscreen {
        Some(true) => {
            cmd.arg("-EnableTrueFullscreen");
        }
        Some(false) => {
            cmd.arg("-DisableTrueFullscreen");
        }
        None => (),
    }
    cmd.arg("-play");
    cmd.arg(selected_path);
    let mut child = cmd.spawn()?;
    let result = child.wait()?;
    Ok(result)
}
