use log::{debug, info};
use simple_logger::SimpleLogger;
use std::{
    env,
    io::{self, Stdout},
    thread,
};
use termion;

use brutus_server::{
    common::TermSize,
    ipc::{pipe, pty},
    ui,
};

fn main() {
    SimpleLogger::new().init().unwrap();

    info!("welcome to brutus");

    let shell_cmd = env::var("SHELL").unwrap();
    debug!("shell: {}", shell_cmd);

    let (cols, rows) = termion::terminal_size().unwrap();

    let (_, reader, mut writer) = ui::Pane::new(shell_cmd, TermSize { rows, cols });

    let connector = pipe::Connector::<pty::Reader, Stdout, 1>::spawn();
    connector.add_connection(reader, io::stdout()).unwrap();

    thread::spawn(move || io::copy(&mut io::stdin(), &mut writer).unwrap());

    loop {}
}
