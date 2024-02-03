use mio::unix::pipe::{Receiver, Sender};
use nix::{pty, unistd};
use std::{
    marker::PhantomData,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    process,
};

use crate::common::TermSize;

pub struct MasterEnd(OwnedFd);

impl MasterEnd {
    pub fn open(cmd: String, size: TermSize) -> Self {
        let forked = unsafe { pty::forkpty(Some(&size.into()), Option::None) }.unwrap();

        if let unistd::ForkResult::Child = forked.fork_result {
            process::Command::new(&cmd).spawn().unwrap().wait().unwrap();
            process::exit(1);
        }

        MasterEnd(forked.master)
    }

    pub fn split(&self) -> (FdGuard<Sender>, FdGuard<Receiver>) {
        (
            FdGuard(
                unsafe { Sender::from_raw_fd(self.0.as_raw_fd()) },
                PhantomData,
            ),
            FdGuard(
                unsafe { Receiver::from_raw_fd(self.0.as_raw_fd()) },
                PhantomData,
            ),
        )
    }
}

pub struct FdGuard<'fd, T>(pub T, PhantomData<&'fd OwnedFd>);
