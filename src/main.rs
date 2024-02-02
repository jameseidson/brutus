use log::{debug, info};
use mio::{unix::pipe::Receiver, Events, Interest, Poll, Token};
use simple_logger::SimpleLogger;
use std::{
    env,
    fs::File,
    io::{self, Read},
    os::fd::FromRawFd,
    str, thread,
};
use termion;

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

    const PTY_RECV: Token = Token(0);

    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);

    let mut receiver = unsafe { Receiver::from_raw_fd(pty.master) };

    poll.registry()
        .register(&mut receiver, PTY_RECV, Interest::READABLE)
        .unwrap();

    let mut buf = [0u8; 256];

    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                PTY_RECV if event.is_read_closed() => {
                    info!("pty closed");
                    return;
                }
                PTY_RECV => {
                    let bytes_read = receiver.read(&mut buf).unwrap();
                    debug!("{:?}", str::from_utf8(&buf[0..bytes_read]).unwrap());
                }
                _ => unreachable!(),
            }
        }
    }
}
