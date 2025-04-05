use is_executable::IsExecutable;
use log::error;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, ExitStatus};

/// A simple vpinball pocess manager
/// Keeps track of the current state of the process
/// Monitors the output of the process and updates the state accordingly

#[derive(Clone)]
pub enum ProcessState {
    /// The process is running
    Starting,
    /// The process is running
    Running,
    /// The process has crashed
    Crashed,
    /// The process has exited
    Exited,
    /// The process has been killed
    Killed,
}

pub struct Vpinball {
    /// The process that is being managed
    process: Child,
    /// The current state of the process
    state: ProcessState,
    /// The output of the process
    output: String,
}

impl Vpinball {
    /// Creates a new vpinball process
    pub fn launch(vpinball_executable: &Path, vpx_path: &PathBuf) -> io::Result<Vpinball> {
        // start rust process

        if !vpinball_executable.is_executable() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Unable to launch table, {} is not executable",
                    vpinball_executable.display()
                ),
            ));
        }

        let process = launch_table(vpx_path, vpinball_executable)?;

        Ok(Vpinball {
            process,
            state: ProcessState::Running,
            output: String::new(),
        })
    }

    pub fn status(&mut self) -> ProcessState {
        let state = match self.process.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    ProcessState::Exited
                } else {
                    match status.code() {
                        Some(11) => {
                            error!(
                                "Visual Pinball exited with segfault, you might want to report this to the Visual Pinball team."
                            );
                        }
                        Some(139) => {
                            // SIGSEGV signal from the operating system on its host node.
                            error!(
                                "Visual Pinball exited with segfault, you might want to report this to the Visual Pinball team."
                            );
                        }
                        Some(code) => {
                            error!("Visual Pinball exited with code {}", code);
                        }
                        None => {
                            error!(
                                "Visual Pinball probably crashed, you might want to report this to the Visual Pinball team."
                            );
                        }
                    }
                    ProcessState::Crashed
                }
            }
            Ok(None) => ProcessState::Running,
            Err(err) => {
                error!(
                    "Error while trying to get the status of the process: {:?}",
                    err
                );
                ProcessState::Killed
            }
        };
        self.state = state;
        self.state.clone()
    }

    pub fn kill(&mut self) -> io::Result<ExitStatus> {
        // TODO: we probably want to first send a SIGTERM and then a SIGKILL
        self.process.kill()?;
        self.process.wait()
    }
}

fn launch_table(selected_path: &Path, vpinball_executable: &Path) -> io::Result<Child> {
    // start process ./VPinballX_GL -play [table path]
    let mut cmd = std::process::Command::new(vpinball_executable);
    cmd.arg("-play");
    cmd.arg(selected_path);
    cmd.spawn()
}

// match launch_table(vpx_path, vpinball_executable) {
//     Ok(status) => match status.code() {
//         Some(0) => {
//             //println!("Table exited normally");
//         }
//         Some(11) => {
//             prompt(&format!(
//                 "{} Visual Pinball exited with segfault, you might want to report this to the vpinball team.",
//                 CRASH
//             ));
//         }
//         Some(139) => {
//             prompt(&format!(
//                 "{} Visual Pinball exited with segfault, you might want to report this to the vpinball team.",
//                 CRASH
//             ));
//         }
//         Some(code) => {
//             prompt(&format!(
//                 "{} Visual Pinball exited with code {}",
//                 CRASH, code
//             ));
//         }
//         None => {
//             prompt("Visual Pinball exited with unknown code");
//         }
//     },
//     Err(e) => {
//         if e.kind() == io::ErrorKind::NotFound {
//             report_and_exit(format!(
//                 "Unable to launch table, vpinball executable not found at {}",
//                 vpinball_executable.display()
//             ));
//         } else {
//             report_and_exit(format!("Unable to launch table: {:?}", e));
//         }
//     }
// }
