use num_enum::{IntoPrimitive, TryFromPrimitive};

// Here it relies on the compiler to check that i32 == c_int
#[repr(i32)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum Signal {
    Sigabrt   = libc::SIGABRT,
    Sigchld  = libc::SIGCHLD,
    Sigcont  = libc::SIGCONT,
    Sighup   = libc::SIGHUP,
    Sigint   = libc::SIGINT,
    Sigio    = libc::SIGIO,
    Sigkill   = libc::SIGKILL,
    Sigprof   = libc::SIGPROF,
    Sigpipe  = libc::SIGPIPE,
    Sigpwr   = libc::SIGPWR,
    Sigquit  = libc::SIGQUIT,
    Sigtstp  = libc::SIGTSTP,
    Sigttin  = libc::SIGTTIN,
    Sigttou  = libc::SIGTTOU,
    Sigurg   = libc::SIGURG,
    Sigusr1  = libc::SIGUSR1,
    Sigusr2  = libc::SIGUSR2,
    Sigvtalrm = libc::SIGVTALRM,
    Sigwinch = libc::SIGWINCH,
    Sigxcpu  = libc::SIGXCPU,
    Sigxfsz  = libc::SIGXFSZ,
}
