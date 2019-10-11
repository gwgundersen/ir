extern crate exitcode;

// Used for tests.
#[allow(unused_imports)]
#[macro_use] extern crate maplit;

use ir::environ;
use ir::fd::parse_fd;
use ir::res;
use ir::spec;
use ir::sys;

fn main() {
    let json_path = match std::env::args().skip(1).next() {
        Some(p) => p,
        None => panic!("no file given"),  // FIXME
    };

    let spec = spec::load_spec_file(&json_path).unwrap_or_else(|err| {
        eprintln!("failed to load {}: {}", json_path, err);
        std::process::exit(exitcode::OSFILE);
    });
    eprintln!("spec: {:?}", spec);
    eprintln!("");

    let env = environ::build(std::env::vars(), &spec.env);

    // Build fd managers.
    let mut fds = spec.fds.iter().map(|(fd_str, fd_spec)| {
        let fd = parse_fd(fd_str).unwrap_or_else(|err| {
            eprintln!("failed to parse fd {}: {}", fd_str, err);
            std::process::exit(exitcode::OSERR);
        });
        ir::fd::create_fd(fd, &fd_spec).unwrap()
    }).collect::<Vec<_>>();

    for fd in &mut fds {
        (*fd).set_up_in_parent().unwrap_or_else(|err| {
            eprintln!("failed to set up fd {}: {}", fd.get_fd(), err);
            std::process::exit(exitcode::OSERR);
        });
    }

    let child_pid = sys::fork().unwrap_or_else(|err| {
        eprintln!("failed to fork: {}", err);
        std::process::exit(exitcode::OSERR);
    });
    if child_pid == 0 {
        // Child process.
        // FIXME: Collect errors and send to parent.

        for fd in &mut fds {
            (*fd).set_up_in_child().unwrap_or_else(|err| {
                eprintln!("failed to set up fd {}: {}", fd.get_fd(), err);
                std::process::exit(exitcode::OSERR);
            });
        }

        let exe = &spec.argv[0];
        let err = sys::execve(exe.clone(), spec.argv.clone(), env).unwrap_err();

        for fd in &mut fds {
            (*fd).clean_up_in_child().unwrap_or_else(|err| {
                eprintln!("failed to clean up fd {}: {}", fd.get_fd(), err);
                std::process::exit(exitcode::OSERR);
            });
        }

        // FIXME: Send this back to the parent process.
        eprintln!("failed to exec: {}", err);
    }
    else {
        // Parent process.
        let (wait_pid, status, rusage) = sys::wait4(child_pid, 0).ok().unwrap();
        assert_eq!(wait_pid, child_pid);  // FIXME: Errors.
        let mut proc_res = res::ProcRes::new(child_pid, status, rusage);

        for fd in &mut fds {
            let res = (*fd).clean_up_in_parent().unwrap_or_else(|err| {
                eprintln!("failed to clean up fd {}: {}", fd.get_fd(), err);
                std::process::exit(exitcode::OSERR);
            });
            if let Some(fd_result) = res {
                proc_res.fds.insert(ir::fd::get_fd_name(fd.get_fd()), fd_result);
            };
        }

        let mut result = res::Res::new();
        result.procs.push(proc_res);

        eprintln!("");
        res::print(&result);
        println!("");
    }

    std::process::exit(exitcode::OK);
}

