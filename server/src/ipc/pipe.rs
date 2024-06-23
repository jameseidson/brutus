use kanal::{bounded, Receiver, Sender};
use mio::{event, Events, Interest, Poll, Token, Waker};
use slab::Slab;
use std::{
    io::{self, Read, Write},
    os::fd::AsRawFd,
    thread,
};

pub type ConnectionId = usize;

pub struct Connector<R, W, const N: usize>
where
    R: Read + event::Source + AsRawFd + Send + 'static,
    W: Write + AsRawFd + Send + 'static,
{
    waker: Waker,
    sender: Sender<WakeReason<R, W>>,
    receiever: Receiver<ConnectionId>,
}

impl<R, W, const N: usize> Connector<R, W, N>
where
    R: Read + event::Source + AsRawFd + Send + 'static,
    W: Write + AsRawFd + Send + 'static,
{
    const WAKE_TOKEN: Token = Token(usize::MAX);

    pub fn spawn() -> Self {
        let poll = Poll::new().unwrap();

        let waker = Waker::new(poll.registry(), Self::WAKE_TOKEN).unwrap();

        let (wake_reason_sender, wake_reason_receiver) = bounded(N);
        let (wake_response_sender, wake_response_receiver) = bounded(N);

        thread::spawn(|| Self::polling_thread(poll, wake_reason_receiver, wake_response_sender));

        Self {
            waker,
            sender: wake_reason_sender,
            receiever: wake_response_receiver,
        }
    }

    pub fn add_connection(&self, reader: R, writer: W) -> io::Result<ConnectionId> {
        self.wake_for_reason(WakeReason::AddConnection(reader, writer))?;

        self.receiever
            .recv()
            .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err))
    }

    pub fn remove_connection(&self, id: ConnectionId) -> io::Result<()> {
        self.wake_for_reason(WakeReason::RemoveConnection(id))
    }

    fn wake_for_reason(&self, reason: WakeReason<R, W>) -> io::Result<()> {
        self.sender
            .send(reason)
            .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err))?;

        self.waker.wake()?;

        Ok(())
    }

    fn polling_thread(
        mut poll: Poll,
        receiver: Receiver<WakeReason<R, W>>,
        sender: Sender<ConnectionId>,
    ) {
        let mut events = Events::with_capacity(1024);
        let mut connections = Slab::<(R, W)>::with_capacity(N);
        let mut buf = [0u8; 8192];

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in &events {
                match event.token() {
                    Self::WAKE_TOKEN => match receiver.recv().unwrap() {
                        WakeReason::AddConnection(reader, writer) => {
                            let id = Self::handle_add_connection(
                                reader,
                                writer,
                                &mut connections,
                                &mut poll,
                            )
                            .unwrap();
                            sender.send(id).unwrap();
                        }
                        WakeReason::RemoveConnection(id) => {
                            Self::handle_remove_connection(id, &mut connections, &mut poll).unwrap()
                        }
                    },
                    Token(id) if event.is_read_closed() => {
                        Self::handle_remove_connection(id, &mut connections, &mut poll).unwrap()
                    }
                    Token(id) => {
                        let (reader, writer) = connections.get_mut(id).unwrap();
                        // TODO: Splice requires that at least one fd is a pipe, so we can't test it with STDIN/STDOUT.
                        // It should work once we are piping data to/from a frontend. For now, we will use buffered I/O.

                        // debug!(
                        //     "{:?}",
                        //     nix::sys::stat::fstat(reader.as_raw_fd()).unwrap().st_mode
                        //         & nix::sys::stat::SFlag::S_IFIFO.bits(),
                        // );
                        // debug!(
                        //     "{:?}",
                        //     nix::sys::stat::fstat(writer.as_raw_fd()).unwrap().st_mode
                        //         & nix::sys::stat::SFlag::S_IFIFO.bits()
                        // );
                        // splice(
                        //     reader.as_raw_fd(),
                        //     None,
                        //     writer.as_raw_fd(),
                        //     None,
                        //     usize::MAX,
                        //     SpliceFFlags::SPLICE_F_NONBLOCK,
                        // )
                        // .unwrap();

                        let bytes_read = reader.read(&mut buf).unwrap();
                        writer.write(&buf[0..bytes_read]).unwrap();
                    }
                }
            }
        }
    }

    fn handle_add_connection(
        reader: R,
        writer: W,
        connections: &mut Slab<(R, W)>,
        poll: &mut Poll,
    ) -> io::Result<ConnectionId> {
        let id = connections.insert((reader, writer));

        if id == usize::MAX {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to add connection: too many connections",
            ));
        }

        poll.registry().register(
            &mut connections.get_mut(id).unwrap().0,
            Token(id),
            Interest::READABLE,
        )?;

        Ok(id)
    }

    fn handle_remove_connection(
        id: ConnectionId,
        connections: &mut Slab<(R, W)>,
        poll: &mut Poll,
    ) -> io::Result<()> {
        poll.registry().deregister(&mut connections.remove(id).0)
    }
}

enum WakeReason<R, W>
where
    R: Read + event::Source + AsRawFd + Send + 'static,
    W: Write + AsRawFd + Send + 'static,
{
    AddConnection(R, W),
    RemoveConnection(ConnectionId),
}
