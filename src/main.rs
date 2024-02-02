use log::{debug, info};
use simple_logger::SimpleLogger;
use std::{env, fs::File, io, os::fd::FromRawFd, thread};
use termion::{self, raw::IntoRawMode, screen::IntoAlternateScreen};

use brutus::pty::PseudoTerm;

fn main() {
    SimpleLogger::new().init().unwrap();

    info!("welcome to brutus");

    let shell_cmd = env::var("SHELL").unwrap();
    debug!("shell: {}", shell_cmd);

    let (cols, rows) = termion::terminal_size().unwrap();

    let pty = PseudoTerm::new(shell_cmd, rows, cols);

    thread::spawn(move || {
        let mut ptm = unsafe { File::from_raw_fd(pty.master) };
        io::copy(&mut io::stdin(), &mut ptm).unwrap();
    });

    thread::spawn(move || {
        let mut ptm = unsafe { File::from_raw_fd(pty.master) };
        let mut stdout = io::stdout()
            .into_raw_mode()
            .and_then(IntoAlternateScreen::into_alternate_screen)
            .unwrap();
        io::copy(&mut ptm, &mut stdout).unwrap();
    });

    loop {}
}
