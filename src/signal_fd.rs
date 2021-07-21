use std::io::{Result, Error};
use std::mem::size_of;

pub use libc::signalfd_siginfo;

use libc::{signalfd, SFD_CLOEXEC, SFD_NONBLOCK};

use tokio::io::unix::AsyncFd;
use tokio::io::Interest;

pub use arrayvec::ArrayVec;

use crate::fd::Fd;
use crate::SignalMask;

/// Due to the fact that epoll on signalfd would fail after fork, you cannot reuse
/// SignalFd after forked.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use libc::{kill, getpid};
/// use async_linux_spec_fd::*;
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn f() {
///     let signalfd = SignalFd::new({
///         let mut signal_mask = SignalMask::new();
///         signal_mask.add(Signal::Sigusr1);
///         signal_mask
///     }).unwrap();
///
///     let need_to_stop = Arc::new(AtomicBool::new(false));
///
///     let need_to_stop_cloned = need_to_stop.clone();
///     std::thread::spawn(move || {
///         let pid = unsafe { getpid() };
///
///         let need_to_stop = need_to_stop_cloned;
///
///         while !need_to_stop.load(Ordering::Relaxed) {
///             assert_eq!(0, unsafe { kill(pid, Signal::Sigusr1.into()) });
///         }
///     });
///
///     let mut cnt = 0;
///
///     while cnt < 1000 {
///         for siginfo in signalfd.read().await.unwrap() {
///             assert_eq!(siginfo.ssi_signo as i32, Signal::Sigusr1.into());
///             cnt += 1;
///         }
///     }
///
///     need_to_stop.store(true, Ordering::Relaxed);
/// }
///
/// f();
/// ```
pub struct SignalFd {
    inner: AsyncFd<Fd>,
}
impl SignalFd {
    /// Returns a `SignalFd` that is close-on-exec.
    ///
    /// If you creates multiple `SignalFd`, then you will be able
    /// to read signals sent to this process from any one of them.
    ///
    /// However, once you read them from one `SignalFd`, you won't 
    /// be able to read it again from another `SignalFd`.
    ///
    /// After `SignalFd` is created, the corresponding signal will be
    /// masked so that your signal handler won't receive them.
    ///
    ///  * `sigmask` - must not contain signals:
    ///     - `SIGKILL`;
    ///     - `SIGSTOP`;
    ///     - `SIGBUS`;
    ///     - `SIGFPE`;
    ///     - `SIGILL`;
    ///     - `SIGSEGV`
    pub fn new(sigmask: SignalMask) -> Result<Self> {
        sigmask.block()?;

        let fd = unsafe {
            signalfd(-1, sigmask.as_sigset(), SFD_NONBLOCK | SFD_CLOEXEC)
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

    /// **NOTE that signals can be coalesced together unless the sender employs
    /// `sigqueue` to send the signals.**
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
