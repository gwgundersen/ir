extern crate exitcode;
extern crate libc;

use libc::{c_int, pid_t, rusage, timeval};
use std::env;
use std::io;

fn wait4(pid: pid_t, options: c_int) -> io::Result<(pid_t, c_int, rusage)> {
    let mut status: c_int = 0;
    let mut usage = rusage {
        ru_utime: timeval { tv_sec: 0, tv_usec: 0 },
        ru_stime: timeval { tv_sec: 0, tv_usec: 0 },
        ru_maxrss: 0,
        ru_ixrss: 0,
        ru_idrss: 0,
        ru_isrss: 0,
        ru_minflt: 0,
        ru_majflt: 0,
        ru_nswap: 0,
        ru_inblock: 0,
        ru_oublock: 0,
        ru_msgsnd: 0,
        ru_msgrcv: 0,
        ru_nsignals: 0,
        ru_nvcsw: 0,
        ru_nivcsw: 0,
    };
    match unsafe {
        libc::wait4(pid, &mut status, options, &mut usage)
    } {
        -1 => Err(io::Error::last_os_error()),
        child_pid => Ok((child_pid, status, usage)),
    }
}


fn main() {
    let json_path = match env::args().skip(1).next() {
        Some(p) => p,
        None => panic!("no file given"),  // FIXME
    };
        
    println!("path: {}", json_path);

    let child_pid = unsafe { libc::fork() };
    if child_pid == 0 {
        println!("child, pid={}", unsafe { libc::getpid() });
        std::process::exit(42);
    }
    else {
        println!("parent, child_pid={}", child_pid);
        let (wait_pid, status, usage) = wait4(child_pid, 0).ok().unwrap();
        println!("waited: {}, status={}", wait_pid, status);
        println!("utime: {}", usage.ru_utime.tv_sec as f64 + 1e-6 * usage.ru_utime.tv_usec as f64);
    }

    std::process::exit(exitcode::OK);
}


