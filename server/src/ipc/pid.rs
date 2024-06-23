use core::fmt;
use log::debug;
use nix::{
    fcntl::{Flock, FlockArg},
    unistd::geteuid,
};
use std::{
    fs::{remove_file, File, OpenOptions},
    io::{self, ErrorKind, Read, Write},
    mem::{self, ManuallyDrop},
    ops::Deref,
    path::{Path, PathBuf},
    process,
};

const PID_FILE_DIR: &'static str = "/var/run/user";

#[derive(Debug)]
pub struct Pid(u32);

impl Pid {
    pub fn get_mine() -> Self {
        process::id().into()
    }

    pub fn read_from(mut reader: impl Read) -> io::Result<Self> {
        let mut buf = [0; 4];
        reader.read(&mut buf)?;
        Ok(Self(u32::from_ne_bytes(buf)))
    }

    pub fn write_to(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write(&self.0.to_ne_bytes())?;
        Ok(())
    }
}

impl From<u32> for Pid {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Pid> for u32 {
    fn from(value: Pid) -> Self {
        value.0
    }
}

/// Represents a process that should only be run once.
pub enum SingletonProcessHandle {
    TheSingleton(PidFile),
    AlreadyRunning(Pid),
}

impl SingletonProcessHandle {
    /// Creates a new singleton process handle identified by the provided name.
    ///
    /// `Singleton` owns a `PidFile` which will exclude other identical processes for as long as it lives.
    /// `AlreadyRunning` provides the pid of the already running singleton process.
    pub fn new(name: String) -> io::Result<Self> {
        let path = Path::new(PID_FILE_DIR)
            .join(geteuid().to_string())
            .join(name + ".pid");

        match PidFile::new(&path) {
            Ok(file) => {
                debug!("created pid file");
                Ok(Self::TheSingleton(file))
            }
            Err(err) => {
                if let io::ErrorKind::WouldBlock = err.kind() {
                    PidFile::read_pid_from_file(path).map(Self::AlreadyRunning)
                } else {
                    Err(err)
                }
            }
        }
    }
}

/// A handle to a .pid file. The file is unlocked and removed on `drop`.
///
/// This API is similar to BSD's [https://man.freebsd.org/cgi/man.cgi?query=pidfile](https://man.freebsd.org/cgi/man.cgi?query=pidfile)
/// with some differences to allow for reads by client processes.
pub struct PidFile {
    path: ManuallyDrop<PathBuf>,
    file: ManuallyDrop<Flock<File>>,
}

impl PidFile {
    /// Open a new pid file if it isn't already locked by another process. This acquires an
    /// exclusive lock on the file.
    pub fn new(path: impl Into<PathBuf>) -> io::Result<PidFile> {
        let path: PathBuf = path.into();

        let file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(io::Error::from)?;

        Ok(PidFile {
            path: ManuallyDrop::new(path),
            file: ManuallyDrop::new(
                Flock::lock(file, FlockArg::LockExclusiveNonblock)
                    .map_err(|(_, errno)| io::Error::from(errno))?,
            ),
        })
    }

    /// Write the current process' id to the pid file in native endian binary and convert
    /// The held lock to shared. This will allow clients to read our pid from the file but
    /// prevent them from taking exclusive ownership of it.
    pub fn write_my_pid(&mut self) -> io::Result<()> {
        Pid::get_mine().write_to(self.file.deref().deref())?;

        self.file
            .relock(FlockArg::LockSharedNonblock)
            .map_err(io::Error::from)?;

        Ok(())
    }

    /// Read the pid contained within the file. Blocks until a shared lock can be taken.
    pub fn read_pid_from_file(path: impl Into<PathBuf>) -> io::Result<Pid> {
        debug!("reading pid from file");
        let file = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(&path.into())?;

        match Flock::lock(file, FlockArg::LockShared) {
            Ok(file) => Pid::read_from(file.deref()),
            Err((_, err)) => Err(io::Error::from(err)),
        }
    }

    /// Drop the pid file without unlocking or deleting it.
    ///
    /// Creating a `PidFile` and then calling `fork` will allow both the child and parent process
    /// to obtain unique descriptors for the same file on disk and therefore share the same
    /// exclusive lock. The lock will not be released until both file descriptors have been closed,
    /// so this function is necessary to safely drop one of the descriptors without unlocking the
    /// file.
    pub fn drop_without_unlocking(mut self) {
        debug!(
            "closing pid file without unlocking it in process {}",
            process::id()
        );
        unsafe {
            ManuallyDrop::<PathBuf>::drop(&mut self.path);
        }
        mem::forget(self)
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        debug!("calling drop in process {}", process::id());
        let _ = remove_file(self.path.as_path());
    }
}
