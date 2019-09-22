extern crate exitcode;

#[allow(unused_imports)]
#[macro_use] extern crate maplit;

mod environ;
mod spec;
mod sys;

fn main() {
    let json_path = match std::env::args().skip(1).next() {
        Some(p) => p,
        None => panic!("no file given"),  // FIXME
    };

    println!("path: {}", json_path);
    let spec = spec::load_spec_file(&json_path).unwrap_or_else(|err| {
        println!("failed to load {}: {}", json_path, err);
        std::process::exit(exitcode::OSFILE);
    });
    println!("spec: {:?}", spec);

    let env = environ::build(std::env::vars(), &spec.env);

    let child_pid = sys::fork().unwrap_or_else(|err| {
        println!("failed to fork: {}", err);
        std::process::exit(exitcode::OSERR);
    });
    if child_pid == 0 {
        println!("child, pid={}", sys::getpid());
        let exe = &spec.argv[0];
        let err = sys::execve(exe.clone(), spec.argv.clone(), env).unwrap_err();
        println!("failed to exec: {}", err);
    }
    else {
        println!("parent, child_pid={}", child_pid);
        let (wait_pid, status, usage) = sys::wait4(child_pid, 0).ok().unwrap();
        println!("waited: {}, status={}", wait_pid, status);
        println!("utime: {}", usage.ru_utime.tv_sec as f64 + 1e-6 * usage.ru_utime.tv_usec as f64);
    }

    std::process::exit(exitcode::OK);
}


