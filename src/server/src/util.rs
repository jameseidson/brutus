use std::{fs::File, path::PathBuf};

use log::LevelFilter;
use nix::pty::Winsize;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

pub struct TermSize {
    pub rows: u16,
    pub cols: u16,
}

impl From<TermSize> for Winsize {
    fn from(ts: TermSize) -> Self {
        Winsize {
            ws_row: ts.rows,
            ws_col: ts.cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }
}

/// Filter errors from the provided `Result` based on the a predicate.
pub fn filter_err<F, E>(result: Result<(), E>, predicate: F) -> Result<(), E>
where
    F: Fn(&E) -> bool,
{
    match result {
        Ok(_) => Ok(()),
        Err(err) if predicate(&err) => Ok(()),
        Err(err) => Err(err),
    }
}

pub fn init_logging() {
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(PathBuf::from("/tmp").join(crate::PROCESS_NAME).as_path()).unwrap(),
        ),
        TermLogger::new(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
    ])
    .unwrap();
    log_panics::init();
}
