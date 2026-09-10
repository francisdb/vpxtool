//! Script line ending fixes

use std::io;
use std::path::Path;
use vpin::vpx::model::StringWithEncoding;
use vpin::vpx::{read_script_file, write_script_file};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum LineEndingsResult {
    NoChanges,
    Unified,
}

pub fn unify_line_endings_vbs_file(vbs_path: &Path) -> io::Result<LineEndingsResult> {
    let script = read_script_file(vbs_path)?;
    let text = script.string;

    let patched_text = unify_line_endings(&text);
    let changed = text != patched_text;

    write_script_file(
        vbs_path,
        &StringWithEncoding {
            encoding: script.encoding,
            string: patched_text,
        },
    )?;

    if changed {
        Ok(LineEndingsResult::Unified)
    } else {
        Ok(LineEndingsResult::NoChanges)
    }
}

pub fn unify_line_endings(script: &str) -> String {
    // first replace all \r\n with \n
    // then replace all \r with \n (this is the main issue, as some files have mixed \r\n and \r)
    // then go back to standard vbs line endings \r\n
    script
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_unify_line_endings_vbs_file_latin1_keeps_encoding() -> io::Result<()> {
        let dir = testdir::testdir!();
        let vbs_path = dir.join("table.vbs");
        // "cafe" with a latin1 e-acute (0xE9), not valid utf8, mixed endings
        let latin1_script = vec![b'c', b'a', b'f', 0xE9, b'\r', b'x', b'\n'];
        std::fs::write(&vbs_path, &latin1_script)?;

        let result = unify_line_endings_vbs_file(&vbs_path)?;

        assert_eq!(result, LineEndingsResult::Unified);
        // line endings unified to \r\n, e-acute still a single latin1 byte
        let expected = vec![b'c', b'a', b'f', 0xE9, b'\r', b'\n', b'x', b'\r', b'\n'];
        assert_eq!(std::fs::read(&vbs_path)?, expected);
        Ok(())
    }

    #[test]
    fn test_unify_line_endings() {
        let script = "first\nsecond\r\nthird\rfourth";
        let expected = "first\r\nsecond\r\nthird\r\nfourth";

        let result = unify_line_endings(script);

        assert_eq!(expected, result);
    }
}
