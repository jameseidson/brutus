use kanal;
use log::debug;
use mio::{Events, Interest, Poll, Token, Waker};
use std::{
    io::{self, Read, Write},
    str, thread,
};

use crate::{common::TermSize, pty};

const WAKE_BY_CHANNEL: Token = Token(0);
const READ_FROM_PTY: Token = Token(1);

pub struct Pane {
    events: EventProducer,
}

impl Pane {
    pub fn new(cmd: String, size: TermSize) -> io::Result<Self> {
        let (event_producer, event_handler) = EventHandler::new(pty::PtyMaster::open(cmd, size))?;

        thread::spawn(move || event_handler.run());

        Ok(Pane {
            events: event_producer,
        })
    }
}

impl Write for Pane {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.events.produce(Event::WriteToPty(Vec::from(buf)))?;
        io::Result::Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.events.produce(Event::FlushPty)?;
        io::Result::Ok(())
    }
}

enum Event {
    // TODO: We don't want to allocate a vector for every write.
    WriteToPty(Vec<u8>),
    FlushPty,
}

struct EventHandler {
    poll: Poll,
    events: Events,
    pty: pty::PtyMaster,
    channel: kanal::AsyncReceiver<Event>,
}

impl EventHandler {
    const EVENT_CAPACITY: usize = 1024;

    pub fn new(mut ptmx: pty::PtyMaster) -> io::Result<(EventProducer, EventHandler)> {
        let (sender, receiver) = kanal::bounded_async::<Event>(Self::EVENT_CAPACITY);
        let poll = Poll::new()?;
        let events = Events::with_capacity(Self::EVENT_CAPACITY);

        poll.registry().register(
            ptmx.receiver.borrow_inner_mut(),
            READ_FROM_PTY,
            Interest::READABLE,
        )?;

        Ok((
            EventProducer {
                waker: Waker::new(poll.registry(), WAKE_BY_CHANNEL)?,
                channel: sender,
            },
            EventHandler {
                pty: ptmx,
                poll,
                events,
                channel: receiver,
            },
        ))
    }

    fn run(mut self) -> io::Result<()> {
        let mut buf = [0u8; 256];

        loop {
            self.poll.poll(&mut self.events, None).unwrap();
            for event in self.events.iter() {
                match event.token() {
                    READ_FROM_PTY if event.is_read_closed() => {
                        return Err(io::Error::from(io::ErrorKind::BrokenPipe))
                    }
                    READ_FROM_PTY => {
                        let bytes_read = self.pty.receiver.borrow_inner().read(&mut buf).unwrap();
                        debug!("{:?}", str::from_utf8(&buf[0..bytes_read]).unwrap());
                    }
                    WAKE_BY_CHANNEL => {
                        if let Some(channel_event) = self
                            .channel
                            .try_recv()
                            .map_err(|_| io::Error::from(io::ErrorKind::BrokenPipe))?
                        {
                            match channel_event {
                                Event::WriteToPty(buf) => {
                                    self.pty.sender.borrow_inner().write(&buf)?;
                                }
                                Event::FlushPty => self.pty.sender.borrow_inner().flush()?,
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}

struct EventProducer {
    waker: Waker,
    channel: kanal::AsyncSender<Event>,
}

impl EventProducer {
    pub fn produce(&self, event: Event) -> io::Result<()> {
        if self
            .channel
            .try_send(event)
            .map_err(|_| io::Error::from(io::ErrorKind::BrokenPipe))?
        {
            self.waker.wake()?;
            io::Result::Ok(())
        } else {
            io::Result::Err(io::Error::from(io::ErrorKind::OutOfMemory))
        }
    }
}
