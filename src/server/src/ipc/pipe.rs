use nix::{sys::stat, unistd::mkfifo};
use std::{
    fs::{File, OpenOptions},
    io,
    path::PathBuf,
    sync::LazyLock,
};

use crate::util::filter_err;

pub mod connector;

pub static CLIENT_EVENT: LazyLock<File> = LazyLock::new(|| {
    OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(CLIENT_EVENT_PATH.as_path())
        .unwrap()
});

static CLIENT_EVENT_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| crate::RUNTIME_DIR.join("client.event"));

// TODO: Only let this be called once.
pub fn create_client_event_pipe() -> io::Result<()> {
    filter_err(
        mkfifo(
            CLIENT_EVENT_PATH.as_path(),
            stat::Mode::S_IRUSR | stat::Mode::S_IWUSR,
        )
        .map_err(io::Error::from),
        |err| err.kind() == io::ErrorKind::AlreadyExists,
    )
}
