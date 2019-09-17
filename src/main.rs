extern crate exitcode;
extern crate libc;

mod spec;
mod sys;

use std::env;

fn main() {
    let json_path = match env::args().skip(1).next() {
        Some(p) => p,
        None => panic!("no file given"),  // FIXME
    };

    println!("path: {}", json_path);
    let spec = spec::load_spec_file(&json_path).unwrap_or_else(|err| {
        println!("failed to load {}: {}", json_path, err);
        std::process::exit(exitcode::OSFILE);
    });
    println!("spec: {:?}", spec);

    let child_pid = unsafe { libc::fork() };
    if child_pid == 0 {
        println!("child, pid={}", unsafe { libc::getpid() });
        std::process::exit(42);
    }
    else {
        println!("parent, child_pid={}", child_pid);
        let (wait_pid, status, usage) = sys::wait4(child_pid, 0).ok().unwrap();
        println!("waited: {}, status={}", wait_pid, status);
        println!("utime: {}", usage.ru_utime.tv_sec as f64 + 1e-6 * usage.ru_utime.tv_usec as f64);
    }

    std::process::exit(exitcode::OK);
}


