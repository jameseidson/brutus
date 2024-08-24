use capnp::{
    message::{ReaderOptions, TypedReader},
    serialize,
};
use nix::{sys::stat, unistd::mkfifo};
use std::{
    fs::{remove_file, File, OpenOptions},
    io,
    path::PathBuf,
    sync::LazyLock,
};

use crate::{
    ipc::pid::{self, Pid},
    util::filter_err,
};

pub use crate::interchange::proto::command as proto;

static PIPE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| crate::RUNTIME_DIR.join(format!("{}.cmd", pid::get_mine())));

pub struct CommandReader(File);

impl CommandReader {
    /// Create the command reader. This must be called before the client starts so that it
    /// can open the write end.
    pub fn create(server_pid: Pid) -> io::Result<()> {
        filter_err(
            mkfifo(
                crate::RUNTIME_DIR
                    .join(format!("{}.cmd", server_pid))
                    .as_path(),
                stat::Mode::S_IRUSR | stat::Mode::S_IWUSR,
            )
            .map_err(io::Error::from),
            |err| err.kind() == io::ErrorKind::AlreadyExists,
        )
    }

    /// Open the read end of the pipe. This blocks until the client opens the write end.
    pub fn open() -> Self {
        CommandReader(
            OpenOptions::new()
                .read(true)
                .write(false)
                .create(false)
                .open(PIPE_PATH.as_path())
                .unwrap(),
        )
    }

    /// Blocks until a command is issued by the client, then returns it.
    pub fn get_command(&self) -> Result<(Pid, proto::Which), capnp::Error> {
        let reader = TypedReader::<_, proto::Owned>::new(serialize::read_message(
            &self.0,
            ReaderOptions::default(),
        )?);
        let cmd = reader.get().unwrap();
        Ok(cmd.which().map(|which| (cmd.get_pid(), which))?)
    }

    pub fn destroy(self) {
        let _ = remove_file(PIPE_PATH.as_path());
    }
}
