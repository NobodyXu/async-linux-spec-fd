use std::io::{Result, Error};
use std::mem::size_of;

pub use libc::signalfd_siginfo;

use libc::{signalfd, SFD_CLOEXEC, SFD_NONBLOCK};
use libc::{sigset_t, SIG_BLOCK, sigemptyset, sigaddset, sigprocmask};

use tokio::io::unix::AsyncFd;
use tokio::io::Interest;

pub use arrayvec::ArrayVec;

use crate::fd::Fd;

// Here it relies on the compiler to check that i32 == c_int
#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum Signal {
    SIGALRM  = libc::SIGALRM,
    SIGCHLD  = libc::SIGCHLD,
    SIGCONT  = libc::SIGCONT,
    SIGHUP   = libc::SIGHUP,
    SIGINT   = libc::SIGINT,
    SIGIO    = libc::SIGIO,
    SIGPIPE  = libc::SIGPIPE,
    SIGPWR   = libc::SIGPWR,
    SIGQUIT  = libc::SIGQUIT,
    SIGTSTP  = libc::SIGTSTP,
    SIGTTIN  = libc::SIGTTIN,
    SIGTTOU  = libc::SIGTTOU,
    SIGURG   = libc::SIGURG,
    SIGUSR1  = libc::SIGUSR1,
    SIGUSR2  = libc::SIGUSR2,
    SIGVTALRM = libc::SIGVTALRM,
    SIGWINCH = libc::SIGWINCH,
    SIGXCPU  = libc::SIGXCPU,
    SIGXFSZ  = libc::SIGXFSZ,
}

/// Due to the fact that epoll on signalfd would fail after fork, you cannot reuse
/// SignalFd after forked.
pub struct SignalFd {
    inner: AsyncFd<Fd>,
}
impl SignalFd {
    pub fn new(signal: Signal) -> Result<Self> {
        let mut mask = std::mem::MaybeUninit::<sigset_t>::uninit();
        unsafe {
            if sigemptyset(mask.as_mut_ptr()) < 0 {
                return Err(Error::last_os_error());
            }
            if sigaddset(mask.as_mut_ptr(), signal as i32) < 0 {
                return Err(Error::last_os_error());
            }
        };
        let mask = unsafe { mask.assume_init() };

        if unsafe {
            sigprocmask(SIG_BLOCK, &mask as *const _, std::ptr::null_mut())
        } < 0 {
            return Err(Error::last_os_error());
        }

        let fd = unsafe {
            signalfd(-1, &mask as *const _, SFD_NONBLOCK | SFD_CLOEXEC)
        };
        if fd < 0 {
            return Err(Error::last_os_error());
        }

        let fd = unsafe { Fd::new(fd) };

        Ok(Self {
            inner: AsyncFd::with_interest(fd, Interest::READABLE)?,
        })
    }

    async fn read_bytes(&self, out: &mut [u8]) -> Result<usize> {
        loop {
            let mut guard = self.inner.readable().await?;

            match guard.try_io(|inner| -> Result<usize> {
                let fd = inner.get_ref();

                fd.read(out)
            }) {
                Ok(result) => break result,
                Err(_would_block) => continue,
            }
        }
    }

    pub async fn read(&self) -> Result<ArrayVec<signalfd_siginfo, 100>> {
        let mut siginfos = ArrayVec::new_const();

        let bytes = unsafe {
            core::slice::from_raw_parts_mut(
                siginfos.as_mut_ptr() as *mut u8,
                siginfos.capacity() * size_of::<signalfd_siginfo>()
            )
        };

        let cnt = self.read_bytes(bytes).await?;
        assert_eq!(cnt % size_of::<signalfd_siginfo>(), 0);

        let items = cnt / size_of::<signalfd_siginfo>();

        unsafe { siginfos.set_len(items) };

        Ok(siginfos)
    }
}
