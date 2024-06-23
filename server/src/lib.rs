use log::{debug, info};
use mio::unix::pipe;
use nix::{
    sys::prctl,
    unistd::{fork, geteuid, setsid, ForkResult},
};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::{
    env,
    ffi::CString,
    fs::File,
    io::{self, ErrorKind, Read, Stdout, Write},
    process::exit,
};

pub mod common;
pub mod ipc;
pub mod ui;

use common::TermSize;
use ipc::{
    pid::{Pid, PidFile, SingletonProcessHandle},
    pty,
};

const SERVER_PROCESS_NAME: &'static str = "brutusd";

#[no_mangle]
pub extern "C" fn spawn_server_if_not_running() -> u32 {
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(format!("/tmp/{}.log", SERVER_PROCESS_NAME)).unwrap(),
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

    match SingletonProcessHandle::new(String::from(SERVER_PROCESS_NAME)).unwrap() {
        SingletonProcessHandle::TheSingleton(pid_file) => {
            let server_pid = spawn_server_daemon(pid_file);
            info!("spawned brutusd with pid {:?}", server_pid);
            server_pid
        }
        SingletonProcessHandle::AlreadyRunning(server_pid) => {
            info!("brutusd already running with pid {:?}", server_pid);
            server_pid
        }
    }
    .into()
}

/// Traditional daemon implementation using double fork method. See
/// [https://man7.org/linux/man-pages/man7/daemon.7.html](https://man7.org/linux/man-pages/man7/daemon.7.html)
///
/// - `pid_file`: Pid file for the server-- should be exclusively locked. The server process will
///   write its pid to the file and convert the locked to shared. This protocol lets new clients
///   know if the server process is already running and exposes its pid to them.
pub fn spawn_server_daemon(mut pid_file: PidFile) -> Pid {
    let (pid_sender, pid_receiver) = pipe::new().unwrap();
    pid_sender.set_nonblocking(false).unwrap();
    pid_receiver.set_nonblocking(false).unwrap();

    match unsafe { fork() }.unwrap() {
        ForkResult::Parent { child: _ } => {
            // We are the client process.
            pid_file.drop_without_unlocking();

            Pid::read_from(pid_receiver).unwrap()
        }
        ForkResult::Child => {
            setsid().unwrap();

            match unsafe { fork() }.unwrap() {
                ForkResult::Parent { child: server_pid } => {
                    // We are a useless intermediary process, but we know what the server's pid is!
                    // Let's send that to the client so that our short life will have some meaning.
                    pid_file.drop_without_unlocking();

                    let server_pid = Pid::from(u32::try_from(server_pid.as_raw()).unwrap());
                    server_pid.write_to(pid_sender).unwrap();

                    exit(0);
                }
                ForkResult::Child => {
                    // We are the server process.
                    pid_file.write_my_pid().unwrap();
                    prctl::set_name(CString::new(SERVER_PROCESS_NAME).unwrap().as_c_str()).unwrap();

                    main();

                    unreachable!();
                }
            }
        }
    }
}

/// Runs the server.
pub fn main() {
    loop {}
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
