use log::{debug, info};
use simple_logger::SimpleLogger;
use std::{env, io};
use termion;

use brutus::{common::TermSize, ui};

fn main() {
    SimpleLogger::new().init().unwrap();

    info!("welcome to brutus");

    let shell_cmd = env::var("SHELL").unwrap();
    debug!("shell: {}", shell_cmd);

    let (cols, rows) = termion::terminal_size().unwrap();

    let mut active_pane = ui::Pane::new(shell_cmd, TermSize { rows, cols }).unwrap();

    io::copy(&mut io::stdin(), &mut active_pane).unwrap();
}
