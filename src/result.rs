use crate::sys::fd_t;
use libc::{c_int, pid_t, rusage};
use std::path::PathBuf;
use serde::{Serialize};

//------------------------------------------------------------------------------

/// Mirror types to derive Serialize for libc remote types.
/// https://serde.rs/remote-derive.html
mod libc_serde {

    use libc::{c_long, suseconds_t, time_t};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "libc::timeval")]
    pub struct Timeval {
        pub tv_sec: time_t,
        pub tv_usec: suseconds_t,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "libc::rusage")]
    pub struct Rusage {
        #[serde(with = "Timeval")]
        pub ru_utime: libc::timeval,
        #[serde(with = "Timeval")]
        pub ru_stime: libc::timeval,
        pub ru_maxrss: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad1: u32,
        pub ru_ixrss: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad2: u32,
        pub ru_idrss: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad3: u32,
        pub ru_isrss: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad4: u32,
        pub ru_minflt: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad5: u32,
        pub ru_majflt: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad6: u32,
        pub ru_nswap: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad7: u32,
        pub ru_inblock: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad8: u32,
        pub ru_oublock: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad9: u32,
        pub ru_msgsnd: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad10: u32,
        pub ru_msgrcv: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad11: u32,
        pub ru_nsignals: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad12: u32,
        pub ru_nvcsw: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad13: u32,
        pub ru_nivcsw: c_long,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        __pad14: u32,

        #[cfg(any(target_env = "musl", target_os = "emscripten"))]
        __reserved: [c_long; 16],
    }

}

//------------------------------------------------------------------------------

#[derive(Serialize)]
pub enum FdResult {
    File {
        path: PathBuf,
    },

    Capture {
        bytes: Vec<u8>,
    },
}

#[derive(Serialize)]
pub struct ProcResult {
    /// The pid with which the process ran.
    pub pid: pid_t,

    /// Pid status, which combines exit code and signum.
    pub status: c_int,
    /// Exit code (low 8 bits), if terminated with exit.
    pub exit_code: Option<i32>,
    /// Signal number, if terminated by signal.
    pub signum: Option<i32>,
    /// Whether the process produced a core dump, if terminated by signal.
    pub core_dump: bool,

    /// Fd results.
    /// FIXME: Associative map from fd instead?
    pub fds: Vec<(fd_t, FdResult)>,

    /// Resource usage for the process itself.
    #[serde(with = "libc_serde::Rusage")]
    pub rusage: rusage,
}

impl ProcResult {
    pub fn new(pid: pid_t, status: c_int, rusage: rusage) -> ProcResult {
        let (exit_code, signum, core_dump)= unsafe {
            if libc::WIFEXITED(status) {
                (Some(libc::WEXITSTATUS(status)), None, false)
            } else {
                (None, Some(libc::WTERMSIG(status)), libc::WCOREDUMP(status))
            }
        };
        let fds: Vec<(fd_t, FdResult)> = Vec::new();
        ProcResult {
            pid,
            status,
            exit_code, signum, core_dump,
            fds,
            rusage,
        }
    }
}

fn time_to_sec(time: libc::timeval) -> f64 {
    time.tv_sec as f64 + 1e-6 * time.tv_usec as f64
}

impl ProcResult {
    /// User time in s.
    pub fn utime(&self) -> f64 {
        time_to_sec(self.rusage.ru_utime)
    }

    /// System time in s.
    pub fn stime(&self) -> f64 {
        time_to_sec(self.rusage.ru_stime)
    }
}

pub fn print(result: &ProcResult) {
    serde_json::to_writer(std::io::stdout(), result).unwrap();
}

