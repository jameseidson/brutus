use nix::{pty, unistd};
use std::{
    os::fd::{IntoRawFd, RawFd},
    process,
};

pub struct PseudoTerm {
    pub master: RawFd,
}

impl PseudoTerm {
    pub fn new(cmd: String, rows: u16, cols: u16) -> Self {
        let forked = unsafe {
            pty::forkpty(
                Some(&pty::Winsize {
                    ws_row: rows,
                    ws_col: cols,
                    ws_xpixel: 0,
                    ws_ypixel: 0,
                }),
                Option::None,
            )
        }
        .unwrap();

        if let unistd::ForkResult::Child = forked.fork_result {
            process::Command::new(&cmd).spawn().unwrap().wait().unwrap();
            process::exit(1);
        }

        PseudoTerm {
            master: forked.master.into_raw_fd(),
        }
    }
}
