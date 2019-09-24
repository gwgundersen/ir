use libc::{pid_t, rusage};
//use serde::{Serialize, Serializer};

//------------------------------------------------------------------------------

//#[derive(Serialize)]
pub struct Result {
    /// The pid with which the process ran.
    pub pid: pid_t,

    /// Pid status, which combines exit code and signum.
    pub status: i32,

    /// Resource usage for the process itself.
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

