use capnp::{
    message::{ReaderOptions, TypedReader},
    serialize,
};
use log::{debug, error};
use std::{
    fs::{remove_file, File, OpenOptions},
    io,
    path::PathBuf,
    sync::LazyLock,
};

use nix::{sys::stat, unistd::mkfifo};

use crate::{
    ipc::pid::{self, Pid},
    proto::{self},
    util::filter_err,
};

static PIPE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| crate::RUNTIME_DIR.join(format!("{}.cmd", pid::get_mine())));

pub fn handle(pid: Pid, cmd: proto::command::Which) {
    match cmd {
        proto::command::Connect(()) => debug!("connect client {:?}", pid),
        proto::command::Empty(()) => unreachable!(),
    }
}

pub struct CommandPipe {
    pipe: File,
}

impl CommandPipe {
    /// Create the command pipe. This must be called before the client starts so that it
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
    pub fn open_read_end() -> Self {
        CommandPipe {
            pipe: OpenOptions::new()
                .read(true)
                .write(false)
                .create(false)
                .open(PIPE_PATH.as_path())
                .unwrap(),
        }
    }

    pub fn read_cmd(&self) -> Result<(Pid, proto::command::Which), capnp::Error> {
        let reader = TypedReader::<_, proto::command::Owned>::new(serialize::read_message(
            &self.pipe,
            ReaderOptions::default(),
        )?);
        let cmd = reader.get().unwrap();
        Ok(cmd.which().map(|which| (cmd.get_pid(), which))?)
    }

    pub fn destroy(self) {
        let _ = remove_file(PIPE_PATH.as_path());
    }
}
