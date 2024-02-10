use nix::pty::Winsize;

pub struct TermSize {
    pub rows: u16,
    pub cols: u16,
}

impl Into<Winsize> for TermSize {
    fn into(self) -> Winsize {
        Winsize {
            ws_row: self.rows,
            ws_col: self.cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }
}
