use std::io::{Result, Error};
use libc::{
    sigset_t, SIG_BLOCK,
    sigemptyset, sigfillset, sigaddset, sigdelset, sigismember, sigprocmask
};

use crate::Signal;

#[derive(Copy, Clone)]
pub struct SignalMask {
    mask: sigset_t
}
impl Default for SignalMask {
    fn default() -> Self {
        Self::new()
    }
}
impl SignalMask {
    /// Create an empty `SignalMask`.
    ///
    /// This is the same as `Default::default()` for `SignalMask`.
    pub fn new() -> Self {
        let mut mask = std::mem::MaybeUninit::<sigset_t>::uninit();

        let ret = unsafe { sigemptyset(mask.as_mut_ptr()) };
        if cfg!(debug_assertions) && ret < 0 {
            let result: Result<()> = Err(Error::last_os_error());
            result.unwrap();
        }

        Self { mask: unsafe { mask.assume_init() } }
    }

    /// Creates a full `SignalMask` contains every signal.
    pub fn new_full() -> Self {
        let mut mask = std::mem::MaybeUninit::<sigset_t>::uninit();

        let ret = unsafe { sigfillset(mask.as_mut_ptr()) };
        if cfg!(debug_assertions) && ret < 0 {
            let result: Result<()> = Err(Error::last_os_error());
            result.unwrap();
        }

        Self { mask: unsafe { mask.assume_init() } }
    }

    /// Add `signal` to the mask.
    pub fn add(&mut self, signal: Signal) -> Result<()> {
        if unsafe { sigaddset(&mut self.mask, signal.into()) } < 0 {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn remove(&mut self, signal: Signal) -> Result<()> {
        if unsafe { sigdelset(&mut self.mask, signal.into()) } < 0 {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn is_member(&self, signal: Signal) -> Result<bool> {
        let result = unsafe { sigismember(&self.mask, signal.into()) };
        if result < 0 {
            Err(Error::last_os_error())
        } else {
            Ok(result != 0)
        }
    }

    /// Block the signal specified in mask and return the old signal mask.
    pub fn block(&self) -> Result<SignalMask> {
        let mut old_mask = std::mem::MaybeUninit::<sigset_t>::uninit();

        if unsafe { sigprocmask(SIG_BLOCK, &self.mask, old_mask.as_mut_ptr()) } < 0 {
            Err(Error::last_os_error())
        } else {
            Ok(Self { mask: unsafe { old_mask.assume_init() } })
        }
    }

    pub fn as_sigset(&self) -> &sigset_t {
        &self.mask
    }

    pub fn as_sigset_mut(&mut self) -> &mut sigset_t {
        &mut self.mask
    }
}
