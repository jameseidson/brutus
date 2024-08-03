use std::{
    fs::{File, OpenOptions},
    io,
    path::PathBuf,
    sync::LazyLock,
};

use nix::{sys::stat, unistd::mkfifo};

use crate::util::filter_err;

static PIPE_PATH: LazyLock<PathBuf> = LazyLock::new(|| crate::RUNTIME_DIR.join("cmd.pipe"));
pub static PIPE: LazyLock<File> = LazyLock::new(|| {
    OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(PIPE_PATH.as_path())
        .unwrap()
});

pub fn create_pipe() -> io::Result<()> {
    filter_err(
        mkfifo(
            PIPE_PATH.as_path(),
            stat::Mode::S_IRUSR | stat::Mode::S_IWUSR,
        )
        .map_err(io::Error::from),
        |err| err.kind() == io::ErrorKind::AlreadyExists,
    )
}
