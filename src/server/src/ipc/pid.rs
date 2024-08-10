use log::debug;
use nix::fcntl::{Flock, FlockArg};
use std::{
    fs::{remove_file, File, OpenOptions},
    io::{self, Read, Write},
    mem::{self, ManuallyDrop},
    ops::Deref,
    path::PathBuf,
    process,
};

pub type Pid = u32;

pub fn get_mine() -> Pid {
    process::id().into()
}

pub fn read(mut reader: impl Read) -> io::Result<Pid> {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;
    Ok(Pid::from_ne_bytes(buf))
}

pub fn write(pid: Pid, mut writer: impl Write) -> io::Result<()> {
    writer.write_all(&pid.to_ne_bytes())?;
    Ok(())
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
        let path = crate::RUNTIME_DIR.as_path().join(name + ".pid");

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
        write(get_mine(), self.file.deref().deref())?;

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
            .open(path.into())?;

        match Flock::lock(file, FlockArg::LockShared) {
            Ok(file) => read(file.deref()),
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
