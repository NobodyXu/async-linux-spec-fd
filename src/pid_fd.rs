use std::convert::TryFrom;
use std::io::{Result, Error};
use std::os::unix::io::{AsRawFd, RawFd};
use std::ptr::null;
use std::mem::MaybeUninit;

use libc::{c_int, c_uint, syscall};

use tokio::io::unix::AsyncFd;
use tokio::io::Interest;

use crate::fd::Fd;
use crate::{pid_t, Signal, siginfo_t};

fn waitid(idtype: libc::idtype_t, id: libc::id_t, options: c_int)
    -> Result<Option<libc::siginfo_t>>
{
    let mut siginfo = MaybeUninit::<libc::siginfo_t>::zeroed();

    let ret = unsafe {
        libc::waitid(idtype, id, siginfo.as_mut_ptr(), options)
    };
    if ret < 0 {
        return Err(Error::last_os_error());
    }

    let siginfo = unsafe { siginfo.assume_init() };
    if unsafe { siginfo.si_pid() } == 0 {
        Ok(None)
    } else {
        Ok(Some(siginfo))
    }
}

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

    /// Asynchronously wait for the child process to terminate and reap it
    /// using `waitid`.
    pub async fn waitpid(&self) -> Result<ExitInfo> {
        self.wait_for_terminate().await?;

        let waitid_option = libc::WEXITED | libc::WNOHANG;

        let pidfd = self.inner.as_raw_fd();
        let siginfo = waitid(libc::P_PIDFD, pidfd as u32, waitid_option)?.unwrap();

        Ok(unsafe { ExitInfo::new(siginfo) })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ExitCode {
    Killed(Signal),
    Exited(c_int),
}

#[derive(Copy, Clone, Debug)]
pub struct ExitInfo {
    /// uid of the child when it exits
    uid: libc::uid_t,
    /// exit code of the child
    code: ExitCode,
}
impl ExitInfo {
    /// * `siginfo` - Must be retrieved via either `waitid` or `SignalFd` or handler
    ///   registered via `sigaction`.
    pub unsafe fn new(siginfo: siginfo_t) -> ExitInfo {
        let status = siginfo.si_status();
        let code =
            if siginfo.si_code == libc::CLD_EXITED {
                ExitCode::Exited(status)
            } else {
                ExitCode::Killed(Signal::try_from(status).unwrap())
            }
        ;

        ExitInfo {
            uid: siginfo.si_uid(),
            code,
        }
    }

    /// uid of the process when it exits
    pub fn get_uid(&self) -> libc::uid_t {
        self.uid
    }

    /// exit code of the child
    pub fn get_code(&self) -> ExitCode {
        self.code
    }
}
