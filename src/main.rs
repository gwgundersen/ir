extern crate exitcode;

// Used for tests.
#[allow(unused_imports)]
#[macro_use] extern crate maplit;

mod environ;
mod result;
mod spec;
mod sys;

fn main() {
    let json_path = match std::env::args().skip(1).next() {
        Some(p) => p,
        None => panic!("no file given"),  // FIXME
    };

    let spec = spec::load_spec_file(&json_path).unwrap_or_else(|err| {
        println!("failed to load {}: {}", json_path, err);
        std::process::exit(exitcode::OSFILE);
    });
    println!("spec: {:?}", spec);
    println!("");

    let env = environ::build(std::env::vars(), &spec.env);

    let child_pid = sys::fork().unwrap_or_else(|err| {
        println!("failed to fork: {}", err);
        std::process::exit(exitcode::OSERR);
    });
    if child_pid == 0 {
        let exe = &spec.argv[0];
        let err = sys::execve(exe.clone(), spec.argv.clone(), env).unwrap_err();
        println!("failed to exec: {}", err);
    }
    else {
        let (wait_pid, status, rusage) = sys::wait4(child_pid, 0).ok().unwrap();
        assert_eq!(wait_pid, child_pid);  // FIXME: Errors.
        let result = result::Result { pid: child_pid, status, rusage };

        println!("");
        result::print(&result);
    }

    std::process::exit(exitcode::OK);
}

