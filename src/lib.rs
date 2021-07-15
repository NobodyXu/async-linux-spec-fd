extern crate libc;
extern crate tokio;
extern crate waitmap;

pub mod utility;
mod fd;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
