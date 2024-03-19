use core::fmt;
use log::debug;
use nix::{
    fcntl::{flock, FlockArg},
    unistd::geteuid,
};
use std::{
    fs::{remove_file, File, OpenOptions},
    io::{self, ErrorKind, Read, Write},
    mem::{self, ManuallyDrop},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    process,
};

const PID_FILE_DIR: &'static str = "/var/run/user";

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
        let pid_file_path = Path::new(PID_FILE_DIR)
            .join(geteuid().to_string())
            .join(name + ".pid");

        match PidFile::new(&pid_file_path) {
            Ok(pid_file) => {
                debug!("created pid file");
                Ok(Self::TheSingleton(pid_file))
            }
            Err(PidFileError::AlreadyLocked) => Ok(Self::AlreadyRunning(
                PidFile::read_pid_from_file(pid_file_path)?,
            )),
            Err(PidFileError::IoError(err)) => Err(err),
        }
    }
}

/// A handle to a .pid file. The file is closed and removed on `drop`.
///
/// This API closeley matches [https://man.freebsd.org/cgi/man.cgi?query=pidfile](https://man.freebsd.org/cgi/man.cgi?query=pidfile)
/// The implementation differs from BSD's, however, to allow for concurrent reads by client processes.
pub struct PidFile {
    path: ManuallyDrop<PathBuf>,
    file: ManuallyDrop<File>,
}

impl PidFile {
    /// Open a new pid file if it isn't already claimed by another process.
    pub fn new(path: impl Into<PathBuf>) -> Result<PidFile, PidFileError> {
        let path: PathBuf = path.into();

        let file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(PidFileError::IoError)?;

        // Temporarily take the exclusive lock until our pid is written.
        flock(file.as_raw_fd(), FlockArg::LockExclusiveNonblock).map_err(|err| {
            let io_err = io::Error::from(err);
            if io_err.kind() == ErrorKind::WouldBlock {
                PidFileError::AlreadyLocked
            } else {
                PidFileError::IoError(io_err)
            }
        })?;

        file.set_len(0).map_err(PidFileError::IoError)?;

        Ok(PidFile {
            path: ManuallyDrop::new(path),
            file: ManuallyDrop::new(file),
        })
    }

    /// Write the current process' id to the pid file in native endian binary.
    pub fn write_my_pid(&mut self) -> io::Result<()> {
        let file: &File = &self.file;
        Pid::get_mine().write_to(file)?;

        // Convert our lock to shared. This will prevent other servers from opening the file as they
        // must furst acquire the exclusive lock, but will allow any number of clients to read from
        // it.
        flock(file.as_raw_fd(), FlockArg::LockSharedNonblock).map_err(io::Error::from)?;

        Ok(())
    }

    /// Read the pid contained within the file. Blocks until a shared lock can be taken.
    ///
    /// This function is intended to be called by a process that does not own the `PidFile`.
    pub fn read_pid_from_file(path: impl Into<PathBuf>) -> io::Result<Pid> {
        debug!("reading pid from file");
        let file = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(&path.into())?;

        flock(file.as_raw_fd(), FlockArg::LockShared).map_err(io::Error::from)?;

        Pid::read_from(file)
    }

    /// Close the pid file without removing it.
    ///
    /// Creating a `PidFile` and then calling `fork` will allow both the child and parent process
    /// to obtain unique descriptors for the same file on disk and therefore share the same
    /// exclusive lock. The lock will not be released until both file descriptors have been closed,
    /// so this function is provided to safely close one of the descriptors without deleting the
    /// file.
    pub fn close(mut self) {
        debug!("closing pid file without removing it");
        unsafe {
            ManuallyDrop::<PathBuf>::drop(&mut self.path);
            ManuallyDrop::<File>::drop(&mut self.file);
        }
        mem::forget(self)
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        debug!("calling drop in process: {}", process::id());
        let _ = remove_file(self.path.as_path());
    }
}

pub enum PidFileError {
    AlreadyLocked,
    IoError(io::Error),
}

impl fmt::Display for PidFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyLocked => write!(f, "pid file was already locked"),
            Self::IoError(err) => err.fmt(f),
        }
    }
}
