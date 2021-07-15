extern crate libc;
extern crate tokio;
extern crate waitmap;
extern crate arrayvec;

pub mod utility;
mod fd;
mod signal_fd;

pub use signal_fd::{Signal, SignalFd, ArrayVec};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
