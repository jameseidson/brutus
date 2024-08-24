use capnp::message::TypedBuilder;
use log::{debug, info};
use mio::unix::pipe as unix_pipe;
use nix::{
    sys::prctl::set_name,
    unistd::{fork, geteuid, setsid, ForkResult},
};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::{
    collections::HashMap,
    ffi::CString,
    fs::{create_dir, File},
    io::ErrorKind,
    mem::ManuallyDrop,
    path::PathBuf,
    process::exit,
    sync::LazyLock,
};

use interchange::{
    command::{self, CommandReader},
    event::{self, EventWriter},
};
use ipc::pid::{self, Pid, PidFile, SingletonProcessHandle};
use util::filter_err;

pub(crate) mod interchange;
pub(crate) mod ipc;
pub(crate) mod ui;
pub(crate) mod util;

pub static PROCESS_NAME: &str = "brutusd";

pub(crate) static RUNTIME_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = PathBuf::from("/var/run/user")
        .join(geteuid().to_string())
        .join("brutus");

    filter_err(create_dir(dir.as_path()), |err| {
        err.kind() == ErrorKind::AlreadyExists
    })
    .unwrap();

    dir
});

#[no_mangle]
pub extern "C" fn spawn_server_if_not_running() -> Pid {
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(PathBuf::from("/tmp").join(PROCESS_NAME).as_path()).unwrap(),
        ),
        TermLogger::new(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
    ])
    .unwrap();
    log_panics::init();

    match SingletonProcessHandle::new(String::from(PROCESS_NAME)).unwrap() {
        SingletonProcessHandle::TheSingleton(pid_file) => {
            let server_pid = spawn_server(pid_file);
            info!("spawned brutusd with pid {:?}", server_pid);
            server_pid
        }
        SingletonProcessHandle::AlreadyRunning(server_pid) => {
            info!("brutusd already running with pid {:?}", server_pid);
            server_pid
        }
    }
}

/// Traditional daemon implementation using double fork method. See
/// [https://man7.org/linux/man-pages/man7/daemon.7.html](https://man7.org/linux/man-pages/man7/daemon.7.html)
///
/// - `pid_file`: Pid file for the server-- should be exclusively locked. The server process will
///   write its pid to the file and convert the locked to shared. This protocol lets new clients
///   know if the server process is already running and exposes its pid to them.
fn spawn_server(mut pid_file: PidFile) -> Pid {
    let (pid_sender, pid_receiver) = unix_pipe::new().unwrap();
    pid_sender.set_nonblocking(false).unwrap();
    pid_receiver.set_nonblocking(false).unwrap();

    match unsafe { fork() }.unwrap() {
        ForkResult::Parent { child: _ } => {
            // We are the client process.
            pid_file.drop_without_unlocking();

            pid::read(pid_receiver).unwrap()
        }
        ForkResult::Child => {
            setsid().unwrap();

            match unsafe { fork() }.unwrap() {
                ForkResult::Parent { child: server_pid } => {
                    // We are a useless intermediary process, but we know what the server's pid is!
                    // Let's send that to the client so that our short existence will have some meaning.
                    pid_file.drop_without_unlocking();

                    let server_pid = Pid::try_from(server_pid.as_raw()).unwrap();

                    CommandReader::create(server_pid).unwrap();
                    pid::write(server_pid, pid_sender).unwrap();

                    exit(0);
                }
                ForkResult::Child => {
                    // We are the server process.
                    set_name(CString::new(crate::PROCESS_NAME).unwrap().as_c_str()).unwrap();

                    pid_file.write_my_pid().unwrap();

                    Server::new().run();

                    unsafe { ManuallyDrop::drop(&mut ManuallyDrop::new(pid_file)) };
                    exit(0)
                }
            }
        }
    }
}

struct Server {
    cmd_reader: CommandReader,
    evt_writers: HashMap<Pid, EventWriter>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            cmd_reader: CommandReader::open(),
            evt_writers: HashMap::new(),
        }
    }

    /// Runs the server.
    pub fn run(mut self) {
        // loop {
        match self.cmd_reader.get_command() {
            Ok((pid, cmd)) => self.handle_command(pid, cmd),
            Err(err) => match err.kind {
                capnp::ErrorKind::PrematureEndOfFile => self.cmd_reader = CommandReader::open(),
                err => {
                    self.cmd_reader.destroy();
                    panic!("{:?}", err)
                }
            },
        }
        // }

        // let (cols, rows) = termion::terminal_size().unwrap();

        // let (_, reader, mut writer) = ui::Pane::new(
        //     shell_cmd,
        //     TermSize {
        //         rows: 71,
        //         cols: 253,
        //     },
        // );
        //
        // let connector = pipe::Connector::<pty::Reader, Stdout, 1>::spawn();
        // connector.add_connection(reader, io::stdout()).unwrap();
        //
        // io::copy(&mut io::stdin(), &mut writer).unwrap();
    }

    fn handle_command(&mut self, pid: Pid, cmd: command::proto::Which) {
        match cmd {
            command::proto::Connect(()) => {
                debug!("connect client {:?}", pid);
                let mut response = TypedBuilder::<event::proto::Owned>::default();
                response.init_root().set_connected(());

                self.evt_writers.insert(pid, EventWriter::open(pid));
                self.evt_writers
                    .get_mut(&pid)
                    .unwrap()
                    .put_event(&response)
                    .unwrap();
            }
            command::proto::Empty(()) => unreachable!(),
        }
    }
}
