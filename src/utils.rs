use nix::pty;

#[derive(Debug, Clone, Copy)]
pub struct TermRect {
    pub rows: u16,
    pub cols: u16,
}

impl From<(u16, u16)> for TermRect {
    fn from(value: (u16, u16)) -> Self {
        TermRect {
            rows: value.0,
            cols: value.1,
        }
    }
}

impl From<TermRect> for pty::Winsize {
    fn from(value: TermRect) -> Self {
        return pty::Winsize {
            ws_row: value.rows,
            ws_col: value.cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
    }
}
