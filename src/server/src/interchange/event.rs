use capnp::{self, message::TypedBuilder};
use std::fs::{File, OpenOptions};

use crate::pid::Pid;

pub use crate::interchange::proto::event as proto;

pub struct EventWriter(File);

impl EventWriter {
    /// Open the write end of the pipe. This blocks until the client opens the read end.
    /// The pipe must be created by the client before this function is called.
    pub fn open(client_pid: Pid) -> Self {
        EventWriter(
            OpenOptions::new()
                .read(false)
                .write(true)
                .create(false)
                .open(
                    crate::RUNTIME_DIR
                        .join(format!("{}.evt", client_pid))
                        .as_path(),
                )
                .unwrap(),
        )
    }

    pub fn put_event(&mut self, evt: &TypedBuilder<proto::Owned>) -> Result<(), capnp::Error> {
        capnp::serialize::write_message(&mut self.0, evt.borrow_inner())
    }
}
