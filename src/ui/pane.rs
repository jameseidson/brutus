use crate::{common::TermSize, pty};

pub struct Pane {
    ptmx: pty::Master,
}

impl Pane {
    pub fn new(cmd: String, size: TermSize) -> Self {
        Self {
            ptmx: pty::open(cmd, size),
        }
    }

    pub fn borrow_io_handles(&mut self) -> (&mut pty::Reader, &mut pty::Writer) {
        (&mut self.ptmx.reader, &mut self.ptmx.writer)
    }
}
