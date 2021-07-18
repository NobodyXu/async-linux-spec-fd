use std::io::{Result, Error};
use std::os::unix::io::{AsRawFd, RawFd};
use std::ptr::null;

use libc::{c_int, c_uint, syscall};

use tokio::io::unix::AsyncFd;
use tokio::io::Interest;

use crate::fd::Fd;
use crate::{pid_t, Signal, siginfo_t};

pub struct PidFd {
    inner: Fd
}
impl PidFd {
    /// # Creating `PidFd` from the pid of children
    ///
    /// Make sure that:
    ///  - the disposition of `SIGCHLD` has not been explicitly set to `SIG_IGN`;
    ///  - the `SA_NOCLDWAIT` flag was not specified while establishing a handler
    ///    for `SIGCHLD` or while setting the disposition of that signal to `SIG_DFL`;
    ///  - the zombie process was not reaped elsewhere in the program (e.g., either by
    ///    an asynchronously executed signal handler or by wait(2) or
    ///    similar in another thread).
    ///
    /// If any of these conditions does not hold, then the child process
    /// (along with a PID file descriptor that refers to it) should instead be created
    /// using `clone` with the `CLONE_PIDFD` flag and uses the `from_raw` function to
    /// create `PidFd`.
    ///
    /// # Creating `PidFd` from arbitary pid
    ///
    /// Make sure to verify that the process pointed to by this pid is the one you
    /// want.
    pub fn open(pid: pid_t) -> Result<Self> {
        let flags: c_uint = 0;
        let ret = unsafe {
            syscall(libc::SYS_pidfd_open, pid, flags)
        };
        if ret < 0 {
            Err(Error::last_os_error())
        } else {
            Ok(unsafe { Self::from_raw(ret as RawFd) })
        }
    }

    /// # Safety
    ///
    /// Make sure `fd` is actually created via `clone` with the `CLONE_PIDFD` flag or
    /// by using `pidfd_open`.
    pub const unsafe fn from_raw(fd: RawFd) -> Self {
        Self { inner: Fd::new(fd) }
    }

    /// * `self` - The calling process must either be in the same PID namespace
    ///   as the process referred to by `self`, or be in an ancestor of that namespace.
    /// * `info` - If equals to `Some(buffer)`, then `buffer` should be
    ///   populated as described in `rt_sigqueueinfo`.
    ///   Or, it is equivalent to specifing to a buffer whose fields are implicily
    ///   filled in as follows:
    ///    - `si_signo` is set to the `signal`;
    ///    - `si_errno` is set to `0`;
    ///    - `si_code` is set to `SI_USER`;
    ///    - `si_pid` is set to the caller's PID;
    ///    - `si_uid` is set to the caller's real user ID.
    pub fn send_signal(&self, signal: Signal, info: Option<&siginfo_t>) -> Result<()> {
        let flags: libc::c_uint = 0;

        let pidfd = self.inner.as_raw_fd();
        let sig: c_int = signal.into();
        let info = info.map_or(null(), |info_ref| info_ref as *const _);

        let ret = unsafe {
            syscall(libc::SYS_pidfd_send_signal, pidfd, sig, info, flags)
        };
        if ret < 0 {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }

    /// Asynchronously wait for the process to terminate.
    pub async fn wait_for_terminate(&self) -> Result<()> {
        let pidfd = self.inner.as_raw_fd();
        let pidfd = AsyncFd::with_interest(pidfd, Interest::READABLE)?;

        pidfd.readable().await?.retain_ready();

        Ok(())
    }
}
