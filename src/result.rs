use libc::{pid_t, rusage};
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

#[derive(Serialize)]
pub struct Result {
    /// The pid with which the process ran.
    pub pid: pid_t,

    /// Pid status, which combines exit code and signum.
    pub status: i32,

    /// Resource usage for the process itself.
    #[serde(with = "libc_serde::Rusage")]
    pub rusage: rusage,

}

fn time_to_sec(time: libc::timeval) -> f64 {
    time.tv_sec as f64 + 1e-6 * time.tv_usec as f64
}

impl Result {
    /// User time in s.
    pub fn utime(&self) -> f64 {
        time_to_sec(self.rusage.ru_utime)
    }

    /// System time in s.
    pub fn stime(&self) -> f64 {
        time_to_sec(self.rusage.ru_stime)
    }
}

pub fn print(result: &Result) {
    serde_json::to_writer_pretty(std::io::stdout(), result);
}

