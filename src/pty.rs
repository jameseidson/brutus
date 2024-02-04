use mio::unix::pipe::{Receiver, Sender};
use nix::{pty, unistd};
use std::{
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    process,
    sync::Arc,
};

use crate::common::TermSize;

pub struct PtyMaster {
    pub sender: FdGuard<Sender>,
    pub receiver: FdGuard<Receiver>,
    _fd: Arc<OwnedFd>,
}

impl PtyMaster {
    pub fn open(cmd: String, size: TermSize) -> Self {
        let forked = unsafe { pty::forkpty(Some(&size.into()), Option::None) }.unwrap();

        if let unistd::ForkResult::Child = forked.fork_result {
            process::Command::new(&cmd).spawn().unwrap().wait().unwrap();
            process::exit(1);
        }

        let raw_fd = forked.master.as_raw_fd();
        let fd = Arc::new(forked.master);
        PtyMaster {
            sender: FdGuard(unsafe { Sender::from_raw_fd(raw_fd) }, fd.clone()),
            receiver: FdGuard(unsafe { Receiver::from_raw_fd(raw_fd) }, fd.clone()),
            _fd: fd,
        }
    }
}

pub struct FdGuard<T>(T, Arc<OwnedFd>);

impl<T> FdGuard<T> {
    pub fn borrow_inner(&self) -> &T {
        &self.0
    }

    pub fn borrow_inner_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
