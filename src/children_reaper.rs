use std::io::{Result, Error};
use std::os::raw::c_int;
use std::sync::Arc;
use std::mem::MaybeUninit;

pub use libc::pid_t;

use waitmap::WaitMap;

use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

use crate::{Signal, SignalFd};

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

// Workaround for WaitMap's strange requirement in wait
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct Pid(libc::pid_t);
impl From<&Pid> for Pid {
    fn from(pid: &Pid) -> Pid {
        *pid
    }
}

/// Currently, Reaper uses WaitMap::wait to wait for the completion of a process, which
/// has the disadvantage that the waiting itself does not removes the entry, thus not
/// freeing any memory at all.
///
/// TODO:
/// Either replaced with something else or fixed it by making a PR to waitmap.
pub struct Reaper {
    signal_fd: SignalFd,
    map: WaitMap<Pid, ExitInfo>,
}
impl Reaper {
    pub fn new() -> Result<Arc<Self>> {
        let ret = Arc::new(Self {
            signal_fd: SignalFd::new(Signal::Sigchld)?,
            map: WaitMap::new(),
        });

        let reaper = ret.clone();
        // Run the reaper in another task so that the zombies won't piled up.
        tokio::spawn(async move {
            Reaper::reap(reaper).await.unwrap()
        });

        Ok(ret)
    }

    async fn reap(reaper: Arc<Self>) -> Result<()> {
        use libc::P_ALL;

        let waitid_option = libc::WEXITED | libc::WNOHANG;

        while Arc::strong_count(&reaper) != 1 {
            // Given that signal is an unreliable way of detecting 
            // SIGCHLD and can cause race condition when using waitid
            // (E.g. after reading all siginfo, some new SIGCHLD is generated
            // but these zombies are already released via watid)
            //
            // Thus it is considered better to just ignore the siginfo at all
            // and just use waitid instead.
            reaper.signal_fd.read().await?;

            // Continue to collect zombies whose SIGCHLD might get coalesced
            while let Some(siginfo) = waitid(P_ALL, 0, waitid_option)? {
                reaper.map.insert(
                    Pid(unsafe { siginfo.si_pid() }),
                    ExitInfo {
                        uid: unsafe { siginfo.si_uid() },
                        wstatus: unsafe { siginfo.si_status() },
                        utime: unsafe { siginfo.si_utime() },
                        stime: unsafe { siginfo.si_stime() }
                    }
                );
            }
        }

        Ok(())
    }

    pub async fn wait(&self, pid: pid_t) -> ExitInfo {
        let pid = Pid(pid);
        loop {
            match self.map.wait(&pid).await {
                Some(val) => break *(val.value()),
                None => continue,
            }
        }
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, TryFromPrimitive)]
pub enum ChildTermSignal {
    Sigabrt   = libc::SIGABRT,
    Sigalrm   = libc::SIGALRM,
    Sigbus    = libc::SIGBUS,
    Sigfpe    = libc::SIGFPE,
    Sighup    = libc::SIGHUP,
    Sigill    = libc::SIGILL,
    Sigint    = libc::SIGINT,
    Sigio     = libc::SIGIO,
    Sigkill   = libc::SIGKILL,
    Sigpipe   = libc::SIGPIPE,
    Sigprof   = libc::SIGPROF,
    Sigpwr    = libc::SIGPWR,
    Sigquit   = libc::SIGQUIT,
    Sigsegv   = libc::SIGSEGV,
    Sigsys    = libc::SIGSYS,
    Sigterm   = libc::SIGTERM,
    Sigtrap   = libc::SIGTRAP,
    Sigusr1   = libc::SIGUSR1,
    Sigusr2   = libc::SIGUSR2,
    Sigvtalrm = libc::SIGVTALRM,
    Sigxcpu   = libc::SIGXCPU,
    Sigxfsz   = libc::SIGXFSZ,
}

#[derive(Copy, Clone, Debug)]
pub struct ExitInfo {
    /// uid of the child when it exits
    uid: libc::uid_t,
    /// exit status of the child
    wstatus: c_int,
    /// user time consumed
    utime: libc::clock_t,
    /// system time consumed
    stime: libc::clock_t,
}
impl ExitInfo {
    /// uid of the process when it exits
    pub fn get_uid(&self) -> libc::uid_t {
        self.uid
    }

    /// user time consumed by the process
    pub fn get_utime(&self) -> libc::clock_t {
        self.utime
    }

    /// system time consumed by the process
    pub fn get_stime(&self) -> libc::clock_t {
        self.stime
    }

    /// Get exit status if the child terminated normally instead of terminated
    /// by signal
    pub fn get_exit_status(&self) -> Option<c_int> {
        if libc::WIFEXITED(self.wstatus) {
            Some(libc::WEXITSTATUS(self.wstatus))
        } else {
            None
        }
    }

    /// Get the signal that terminated the process if it is killed by signal
    pub fn get_term_sig(&self) -> Option<ChildTermSignal> {
        if libc::WIFSIGNALED(self.wstatus) {
            // It will be an internal error if the platform reports that the signal
            // is killed by signal not listed in ChildTermSignal
            Some(ChildTermSignal::try_from(libc::WTERMSIG(self.wstatus)).unwrap())
        } else {
            None
        }
    }
}
