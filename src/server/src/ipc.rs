use capnp::{self, message::TypedBuilder, serialize};

use crate::{
    ffi,
    interchange::{command, event},
};

pub mod pid;
pub mod pipe;
pub mod pty;

struct Ipc(ffi::ipc_t);

impl Ipc {
    fn new() -> Self {
        Self(unsafe { ffi::ipc_open(crate::RUNTIME_DIR) })
        // ffi::ipc_open(crate::RUNTIME_DIR, TypedBuilder<event::proto::Owned>::default().)
    }

    fn send(&self, evt: TypedBuilder<proto::Owned>) {
        let buf = capnp::serialize::write_message_to_words(evt.borrow_inner());
        unsafe { ffi::ipc_send(self.0, buf, buf.len()) };
    }

    fn recv(&self) {
        capnp::serialize::read
    }
}
