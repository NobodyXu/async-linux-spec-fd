extern crate libc;
extern crate tokio;
extern crate waitmap;
extern crate arrayvec;
extern crate num_enum;

pub mod utility;
mod fd;
mod signal_fd;
mod children_reaper;

pub use signal_fd::{Signal, SignalFd, ArrayVec};
pub use children_reaper::{Reaper, ExitInfo, ChildTermSignal, pid_t};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
