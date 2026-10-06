use colored::Colorize;
use std::io;
use std::io::Write;
use std::process::ExitCode;

// These macros are needed because the normal ones panic when there's a broken pipe.
// This is especially problematic for CLI tools that are frequently piped into `head` or `grep -q`
// From https://github.com/rust-lang/rust/issues/46016#issuecomment-1242039016
// clippy.toml disallows the std ones, use `crate::println!` and friends instead.
#[macro_export]
macro_rules! println {
  () => ({
    writeln!(std::io::stdout())
  });
  ($fmt:expr) => ({
    writeln!(std::io::stdout(), $fmt)
  });
  ($fmt:expr, $($arg:tt)*) => ({
    writeln!(std::io::stdout(), $fmt, $($arg)*)
  })
}

#[macro_export]
macro_rules! print {
  ($fmt:expr) => ({
    write!(std::io::stdout(), $fmt)
  });
  ($fmt:expr, $($arg:tt)*) => ({
    write!(std::io::stdout(), $fmt, $($arg)*)
  })
}

#[macro_export]
macro_rules! eprintln {
  () => ({
    writeln!(&mut std::io::stderr())
  });
  ($fmt:expr) => ({
    writeln!(&mut std::io::stderr(), $fmt)
  });
  ($fmt:expr, $($arg:tt)*) => ({
    writeln!(&mut std::io::stderr(), $fmt, $($arg)*)
  })
}

#[macro_export]
macro_rules! eprint {
  ($fmt:expr) => ({
    write!(&mut std::io::stderr(), $fmt)
  });
  ($fmt:expr, $($arg:tt)*) => ({
    write!(&mut std::io::stderr(), $fmt, $($arg)*)
  })
}

pub fn safe_main(main: fn() -> io::Result<ExitCode>) -> ExitCode {
    // from https://github.com/rust-lang/rust/issues/46016#issuecomment-1242039016
    match main() {
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => {
            // Okay, this happens when the output is piped to a program like `head`
            ExitCode::SUCCESS
        }
        Err(err) => {
            let warning = format!("{err}").red();
            eprintln!("{warning}").ok();
            ExitCode::FAILURE
        }
        Ok(exit_code) => exit_code, //::SUCCESS,
    }
}
