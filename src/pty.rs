use mio::{event, unix::SourceFd, Interest, Registry, Token};
use nix::{pty, unistd};
use std::{
    io::{self, Read, Write},
    os::fd::{AsRawFd, OwnedFd},
    process,
    sync::Arc,
};

use crate::common::TermSize;

pub struct Master {
    pub reader: Reader,
    pub writer: Writer,
    #[allow(dead_code)]
    fd: Arc<OwnedFd>,
}

pub fn open(cmd: String, size: TermSize) -> Master {
    let forked = unsafe { pty::forkpty(Some(&size.into()), Option::None) }.unwrap();

    if let unistd::ForkResult::Child = forked.fork_result {
        process::Command::new(&cmd).spawn().unwrap().wait().unwrap();
        process::exit(1);
    }

    let fd = Arc::new(forked.master);
    Master {
        reader: Reader(Arc::clone(&fd)),
        writer: Writer(Arc::clone(&fd)),
        fd,
    }
}

pub struct Reader(Arc<OwnedFd>);

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        unistd::read(self.0.as_raw_fd(), buf).map_err(io::Error::from)
    }
}

impl event::Source for Reader {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.0.as_raw_fd()).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.0.as_raw_fd()).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        SourceFd(&self.0.as_raw_fd()).deregister(registry)
    }
}

pub struct Writer(Arc<OwnedFd>);

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unistd::write(self.0.as_raw_fd(), buf).map_err(io::Error::from)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
