// manages background vpinball process

use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver, Sender};
use is_executable::IsExecutable;
use std::path::Path;
use std::process::{exit, ExitStatus};
use std::time::Duration;
use std::{io, thread};

#[derive(Debug)]
pub(crate) enum VpxResult {
    VpxDone,
}

#[derive(Resource, Deref)]
pub(crate) struct StreamReceiver(Receiver<VpxResult>);

#[derive(Resource, Deref)]
pub(crate) struct StreamSender(Sender<VpxResult>);

#[derive(Event, Debug)]
pub(crate) struct VpxEvent(pub(crate) VpxResult);

pub(crate) fn process_plugin(app: &mut App) {
    app.add_systems(Startup, setup_channel);
    app.add_systems(Update, forward_events_to_bevy);
}

fn setup_channel(mut commands: Commands) {
    let (tx, rx) = bounded::<VpxResult>(10);
    commands.insert_resource(StreamSender(tx));
    commands.insert_resource(StreamReceiver(rx));
}

// This system reads from the receiver and sends events to Bevy
pub(crate) fn forward_events_to_bevy(
    receiver: Res<StreamReceiver>,
    mut events: EventWriter<VpxEvent>,
) {
    let _event_writer = &events;
    for from_stream in receiver.try_iter() {
        events.send(VpxEvent(from_stream));
    }
}

pub(crate) fn do_launch(tx: Sender<VpxResult>, path: &Path, executable: &Path) {
    info!("Launching table {}", path.display());
    let tx = tx.clone();
    let path = path.to_path_buf();
    let executable = executable.to_path_buf();

    let _vpinball_thread = thread::spawn(move || {
        launch(&path, &executable, None);
        thread::sleep(Duration::from_millis(2_u64));

        info!("Vpinball done, sending event");
        tx.send(VpxResult::VpxDone).unwrap();

        //resume_music(&mut control_music_event_writer);
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
                eprintln!("Visual Pinball exited with segfault, you might want to report this to the vpinball team.");
            }
            Some(139) => {
                eprintln!("Visual Pinball exited with segfault, you might want to report this to the vpinball team.");
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
