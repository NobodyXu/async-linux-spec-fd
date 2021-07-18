use num_enum::{IntoPrimitive, TryFromPrimitive};

// Here it relies on the compiler to check that i32 == c_int
#[repr(i32)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum Signal {
    Sigchld   = libc::SIGCHLD,
    Sigcont   = libc::SIGCONT,
    Sigtstp   = libc::SIGTSTP,
    Sigttin   = libc::SIGTTIN,
    Sigttou   = libc::SIGTTOU,
    Sigurg    = libc::SIGURG,
    Sigwinch  = libc::SIGWINCH,
    Sigabrt   = libc::SIGABRT,
    Sigalrm   = libc::SIGALRM,
    Sigbus    = libc::SIGBUS,
    Sigfpe    = libc::SIGFPE,
    Sighup    = libc::SIGHUP,
    Sigill    = libc::SIGILL,
    Sigint    = libc::SIGINT,
    Sigio     = libc::SIGIO,
    Sigkill   = libc::SIGKILL,
    Sigpipe   = libc::SIGPIPE,
    Sigprof   = libc::SIGPROF,
    Sigpwr    = libc::SIGPWR,
    Sigquit   = libc::SIGQUIT,
    Sigsegv   = libc::SIGSEGV,
    Sigsys    = libc::SIGSYS,
    Sigterm   = libc::SIGTERM,
    Sigtrap   = libc::SIGTRAP,
    Sigusr1   = libc::SIGUSR1,
    Sigusr2   = libc::SIGUSR2,
    Sigvtalrm = libc::SIGVTALRM,
    Sigxcpu   = libc::SIGXCPU,
    Sigxfsz   = libc::SIGXFSZ,
}
