use std::io::{Result, ErrorKind};

pub fn autorestart<T, F>(mut f: F)
    -> Result<T>
    where F: FnMut() -> Result<T>
{
    loop {
        let ret = f();

        if let Err(err) = &ret {
            match err.kind() {
                ErrorKind::Interrupted => continue,
                _ => (),
            }
        }

        break ret
    }
}
#[macro_export]
macro_rules! autorestart {
    ( { $( $tt:tt )* } ) => {
        $crate::utility::autorestart(
            || { $( $tt )* }
        )
    };
}
