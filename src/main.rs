use log::{debug, info};
use mio::{Events, Interest, Poll, Token};
use simple_logger::SimpleLogger;
use std::{
    env,
    io::{self, Read},
    str, thread,
};
use termion;

use brutus::{common::TermSize, pty};

fn main() {
    SimpleLogger::new().init().unwrap();

    info!("welcome to brutus");

    let shell_cmd = env::var("SHELL").unwrap();
    debug!("shell: {}", shell_cmd);

    let (cols, rows) = termion::terminal_size().unwrap();

    let mut ptmx = pty::PtyMaster::open(shell_cmd, TermSize { rows, cols });

    thread::spawn(move || {
        io::copy(&mut io::stdin(), ptmx.sender.borrow_inner_mut()).unwrap();
    });

    const PTY_RECV: Token = Token(0);

    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);

    poll.registry()
        .register(
            ptmx.receiver.borrow_inner_mut(),
            PTY_RECV,
            Interest::READABLE,
        )
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
                    let bytes_read = ptmx.receiver.borrow_inner().read(&mut buf).unwrap();
                    debug!("{:?}", str::from_utf8(&buf[0..bytes_read]).unwrap());
                }
                _ => unreachable!(),
            }
        }
    }
}
