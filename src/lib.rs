use std::fs::metadata;
use std::io;
use std::path::{Path, PathBuf};

mod backglass;
pub mod fixprint;
mod frontend;
pub mod patcher;

pub mod config;

pub mod indexer;

mod capture;
pub mod cli;
mod vpinball;
pub mod vpinball_config;

pub fn strip_cr_lf(s: &str) -> String {
    s.chars().filter(|c| !c.is_ascii_whitespace()).collect()
}

fn expand_path<S: AsRef<str>>(path: S) -> PathBuf {
    shellexpand::tilde(path.as_ref()).to_string().into()
}

fn expand_path_exists<S: AsRef<str>>(path: S) -> io::Result<PathBuf> {
    // TODO expand all instead of only tilde?
    let expanded_path = shellexpand::tilde(path.as_ref());
    path_exists(&PathBuf::from(expanded_path.to_string()))
}

fn path_exists(expanded_path: &Path) -> io::Result<PathBuf> {
    match metadata(expanded_path) {
        Ok(md) => {
            if !md.is_file() && !md.is_dir() && md.is_symlink() {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{} is not a file", expanded_path.display()),
                ))
            } else {
                Ok(expanded_path.to_path_buf())
            }
        }
        Err(msg) => {
            let warning = format!(
                "Failed to read metadata for {}: {}",
                expanded_path.display(),
                msg
            );
            Err(io::Error::new(io::ErrorKind::InvalidInput, warning))
        }
    }
}

fn os_independent_file_name(file_path: String) -> Option<String> {
    // we can't use path here as this uses the system path encoding
    // we might have to parse windows paths on mac/linux
    if file_path.is_empty() {
        return None;
    }
    file_path.rsplit(['/', '\\']).next().map(|f| f.to_string())
}

/// Path to file that will be removed when it goes out of scope
struct RemoveOnDrop {
    path: PathBuf,
}
impl RemoveOnDrop {
    fn new(path: PathBuf) -> Self {
        RemoveOnDrop { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        if self.path.exists() {
            // silently ignore any errors
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_independent_file_name_windows() {
        let file_path = "C:\\Users\\user\\Desktop\\file.txt";
        let result = os_independent_file_name(file_path.to_string());
        assert_eq!(result, Some("file.txt".to_string()));
    }

    #[test]
    fn test_os_independent_file_unix() {
        let file_path = "/users/joe/file.txt";
        let result = os_independent_file_name(file_path.to_string());
        assert_eq!(result, Some("file.txt".to_string()));
    }

    #[test]
    fn test_os_independent_file_name_no_extension() {
        let file_path = "C:\\Users\\user\\Desktop\\file";
        let result = os_independent_file_name(file_path.to_string());
        assert_eq!(result, Some("file".to_string()));
    }

    #[test]
    fn test_os_independent_file_name_no_path() {
        let file_path = "file.txt";
        let result = os_independent_file_name(file_path.to_string());
        assert_eq!(result, Some("file.txt".to_string()));
    }

    #[test]
    fn test_os_independent_file_name_no_path_no_extension() {
        let file_path = "file";
        let result = os_independent_file_name(file_path.to_string());
        assert_eq!(result, Some("file".to_string()));
    }

    #[test]
    fn test_os_independent_file_name_empty() {
        let file_path = "";
        let result = os_independent_file_name(file_path.to_string());
        assert_eq!(result, None);
    }
}
