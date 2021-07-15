use std::os::unix::io::{AsRawFd, RawFd};
use std::os::raw::c_void;
use std::io::{Result, Error};

use crate::autorestart;

#[derive(Debug)]
pub struct Fd {
    inner: RawFd,
}
impl AsRawFd for Fd {
    fn as_raw_fd(&self) -> RawFd {
        self.inner
    }
}
impl Drop for Fd {
    fn drop(&mut self) {
        let ret = unsafe {
            libc::close(self.inner)
        };

        if cfg!(debug_assertions) && ret < 0 {
            let result: Result<()> = Err(Error::last_os_error());
            result.unwrap();
        }
    }
}
impl Fd {
    pub const unsafe fn new(raw_fd: RawFd) -> Fd {
        Fd { inner: raw_fd }
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;
        let buf_len = buf.len() as libc::size_t;

        autorestart!({
            let result = unsafe {
                libc::read(self.inner, buf_ptr, buf_len)
            };
            if result < 0 {
                Err(Error::last_os_error())
            } else {
                Ok(result as usize)
            }
        })
    }
}
