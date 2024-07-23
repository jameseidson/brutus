use mio::{event, unix::SourceFd, Interest, Registry, Token};
use nix::{
    pty,
    sys::termios::{self, InputFlags, SetArg},
    unistd,
};
use std::{
    io::{self, Read, Write},
    os::fd::{AsRawFd, OwnedFd},
    process,
    sync::Arc,
};

use crate::common::TermSize;

pub fn open(cmd: String, size: TermSize) -> (Controller, Reader, Writer) {
    match unsafe { pty::forkpty(Some(&size.into()), Option::None) }.unwrap() {
        pty::ForkptyResult::Child => {
            process::Command::new(&cmd).spawn().unwrap().wait().unwrap();
            process::exit(1);
        }
        pty::ForkptyResult::Parent { master, .. } => {
            let mut attrs = termios::tcgetattr(&master).unwrap();
            attrs.input_flags.set(InputFlags::IUTF8, true);
            termios::tcsetattr(&master, SetArg::TCSANOW, &attrs).unwrap();

            let fd = Arc::new(master);
            (
                Controller(Arc::clone(&fd)),
                Reader(Arc::clone(&fd)),
                Writer(Arc::clone(&fd)),
            )
        }
    }
}

pub struct Controller(Arc<OwnedFd>);

impl Controller {
    pub fn resize(&self) {
        // TODO: With ioctl.
        todo!()
    }
}

pub struct Reader(Arc<OwnedFd>);

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        unistd::read(self.as_raw_fd(), buf).map_err(io::Error::from)
    }
}

impl AsRawFd for Reader {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.0.as_raw_fd()
    }
}

impl event::Source for Reader {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).deregister(registry)
    }
}

pub struct Writer(Arc<OwnedFd>);

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unistd::write(&self.0, buf).map_err(io::Error::from)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl AsRawFd for Writer {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.0.as_raw_fd()
    }
}
