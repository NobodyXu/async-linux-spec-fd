use std::io::{Result, ErrorKind};

/// auto restart functions failed due to interrpted
pub fn autorestart<T, F>(mut f: F)
    -> Result<T>
    where F: FnMut() -> Result<T>
{
    loop {
        let ret = f();

        if let Err(err) = &ret {
            if let ErrorKind::Interrupted = err.kind() {
                continue
            }
        }

        break ret
    }
}

/// auto restart functions failed due to interrpted
#[macro_export]
macro_rules! autorestart {
    ( { $( $tt:tt )* } ) => {
        $crate::utility::autorestart(
            || { $( $tt )* }
        )
    };
}
