use crate::{common::TermSize, ipc::pty};

pub struct Pane {
    ptmx: pty::Controller,
}

impl Pane {
    pub fn new(cmd: String, size: TermSize) -> (Self, pty::Reader, pty::Writer) {
        let (controller, reader, writer) = pty::open(cmd, size);
        (Self { ptmx: controller }, reader, writer)
    }
}
