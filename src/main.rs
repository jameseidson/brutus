use log::{debug, info};
use nix::{pty, unistd};
use std::{
    env,
    fs::File,
    os::fd::{FromRawFd, IntoRawFd},
    process, thread,
};
use termion::{self, raw::IntoRawMode, screen::IntoAlternateScreen};
use tokio::{io, join, process::Command};

use brutus::pane;

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();

    info!("welcome to brutus");

    // let shell = env::var("SHELL").unwrap();
    // debug!("shell: {}", shell);
    // let (cols, rows) = termion::terminal_size().unwrap();
    //
    // let forked = unsafe {
    //     pty::forkpty(
    //         Some(&pty::Winsize {
    //             ws_row: rows,
    //             ws_col: cols,
    //             ws_xpixel: 0,
    //             ws_ypixel: 0,
    //         }),
    //         Option::None,
    //     )
    // }
    // .unwrap();
    //
    // if let unistd::ForkResult::Child = forked.fork_result {
    //     process::Command::new(&shell)
    //         .spawn()
    //         .unwrap()
    //         .wait()
    //         .unwrap();
    //     std::process::exit(1);
    // }
    // let ptm_raw = forked.master.into_raw_fd();
    //
    // thread::spawn(move || {
    //     let mut ptm = unsafe { File::from_raw_fd(ptm_raw) };
    //     io::copy(&mut io::stdin(), &mut ptm).unwrap();
    // });
    //
    // thread::spawn(move || {
    //     let mut ptm = unsafe { File::from_raw_fd(ptm_raw) };
    //     let mut stdout = io::stdout()
    //         .into_raw_mode()
    //         .and_then(IntoAlternateScreen::into_alternate_screen)
    //         .unwrap();
    //     io::copy(&mut ptm, &mut stdout).unwrap();
    // });

    let pty = pane::Pty::new(termion::terminal_size().unwrap().into()).unwrap();
    let (mut read_half, mut write_half) = io::split(pty.attach_shell().unwrap());
}
