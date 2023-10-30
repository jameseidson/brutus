use anyhow::Result;
use core::task::Poll;
use log::{debug, info};
use nix::{
    errno::Errno,
    libc,
    // libc::{ioctl, EWOULDBLOCK, TIOCGWINSZ, TIOCSWINSZ},
    pty,
    unistd,
};
use std::{
    env,
    io::Error,
    os::fd::{AsRawFd, FromRawFd, RawFd},
    process::Stdio,
};
use tokio::{
    io::{self, unix::AsyncFd, AsyncRead, AsyncWrite},
    process::Command,
};

use crate::utils;

pub struct Pty {
    size: utils::TermRect,
    master: PtyMaster,
    slave: RawFd,
}

impl Pty {
    pub fn new(size: utils::TermRect) -> Result<Self> {
        let pty::OpenptyResult { master, slave } = pty::openpty(Some(&size.into()), None)?;
        let ptm = master.as_raw_fd();
        let pts = slave.as_raw_fd();

        unsafe {
            libc::unlockpt(ptm);
            // libc::login_tty(pts);
        }

        Ok(Pty {
            size,
            master: PtyMaster::new(master.as_raw_fd())?,
            slave: slave.as_raw_fd(),
        })
    }

    pub fn attach_shell(self) -> Result<PtyMaster> {
        // let shell = env::var("SHELL")?;
        // self.attach_cmd(Command::new(shell))
        self.attach(Command::new("/bin/bash"))
    }

    /// Spawns a child process running the provided command and points its `stdin`, `stdout`, and `stderr` streams at the slave end of the pty. Returns the master end.
    pub fn attach(self, mut cmd: Command) -> Result<PtyMaster> {
        cmd.spawn().map_err(anyhow::Error::from)?;
        Ok(self.master)

        // cmd.stdin(unsafe { Stdio::from_raw_fd(self.slave) })
        //     .stdout(unsafe { Stdio::from_raw_fd(self.slave) })
        //     .stderr(unsafe { Stdio::from_raw_fd(self.slave) })
        //     .spawn()
        //     .map_err(anyhow::Error::from)?;
        //
        // Ok(self.master)
    }
}

/// Represents the master end of the pty
#[derive(Debug)]
pub struct PtyMaster(AsyncFd<RawFd>);

impl PtyMaster {
    fn new(fd: RawFd) -> Result<Self> {
        Ok(Self(AsyncFd::new(fd)?))
    }

    pub fn resize(size: utils::TermRect) {
        // TODO: with ioctl
        todo!()
    }
}

impl AsRawFd for PtyMaster {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl AsyncRead for PtyMaster {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            return match self.0.poll_read_ready(cx)? {
                Poll::Ready(mut ready) => {
                    let unfilled = buf.initialize_unfilled();
                    match unistd::read(self.0.as_raw_fd(), unfilled) {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(Errno::EWOULDBLOCK) => {
                            ready.clear_ready();
                            continue;
                        }
                        Err(err) => Poll::Ready(Err(io::Error::from(err))),
                    }
                }
                Poll::Pending => Poll::Pending,
            };
        }
    }
}

impl AsyncWrite for PtyMaster {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        loop {
            return match self.0.poll_write_ready(cx)? {
                Poll::Ready(mut ready) => match unistd::write(self.0.as_raw_fd(), buf) {
                    Ok(n_written) => Poll::Ready(Ok(n_written)),
                    Err(Errno::EWOULDBLOCK) => {
                        ready.clear_ready();
                        continue;
                    }
                    Err(err) => Poll::Ready(Err(io::Error::from(err))),
                },
                Poll::Pending => Poll::Pending,
            };
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        Poll::Ready(if unsafe { libc::fsync(self.0.as_raw_fd()) } == 0 {
            Ok(())
        } else {
            Err(Error::last_os_error())
        })
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        Poll::Ready(nix::unistd::close(self.0.as_raw_fd()).map_err(io::Error::from))
    }
}
