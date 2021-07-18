extern crate libc;
extern crate tokio;
extern crate waitmap;
extern crate arrayvec;
extern crate num_enum;

pub use libc::{pid_t, siginfo_t};

pub mod utility;
mod fd;
mod signal_fd;
mod children_reaper;
mod pid_fd;

pub use signal_fd::{Signal, SignalFd, ArrayVec};
pub use children_reaper::{Reaper, ExitInfo, ChildTermSignal};
pub use pid_fd::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
