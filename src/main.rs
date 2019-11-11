extern crate exitcode;

// Used for tests.
#[allow(unused_imports)]
#[macro_use] extern crate maplit;

use ir::environ;
use ir::fd::parse_fd;
use ir::res;
use ir::sel;
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
        // FIXME: Parse when deserializing, rather than here.
        let fd_num = parse_fd(fd_str).unwrap_or_else(|err| {
            eprintln!("failed to parse fd {}: {}", fd_str, err);
            std::process::exit(exitcode::OSERR);
        });

        // FIXME: Errors.
        let mut fd = ir::fd::create_fd(fd_num, &fd_spec).unwrap();

        // FIXME: Errors.
        fd.set_up().unwrap_or_else(|err| {
            eprintln!("failed to set up fd {}: {}", fd.get_fd(), err);
            std::process::exit(exitcode::OSERR);
        });

        fd
    }).collect::<Vec<_>>();

    // Fork the child process.
    let child_pid = sys::fork().unwrap_or_else(|err| {
        eprintln!("failed to fork: {}", err);
        // FIXME: Errors.
        std::process::exit(exitcode::OSERR);
    });
    if child_pid == 0 {
        // Child process.
        // FIXME: Collect errors and send to parent.

        for fd in &mut fds {
            // FIXME: Errors.
            (*fd).set_up_in_child().unwrap_or_else(|err| {
                eprintln!("failed to set up fd {}: {}", fd.get_fd(), err);
                std::process::exit(exitcode::OSERR);
            });
        }

        let exe = &spec.argv[0];
        // FIXME: Errors.
        let err = sys::execve(exe.clone(), spec.argv.clone(), env).unwrap_err();

        for fd in &mut fds {
            // FIXME: Errors.
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

        // Set up the selector, which will manage events while the child runs.
        let mut selecter = sel::Selecter::new();
        for fd in &mut fds {
            // FIXME: Errors.
            (*fd).set_up_in_parent(&mut selecter).unwrap_or_else(|err| {
                eprintln!("failed to set up fd {}: {}", fd.get_fd(), err);
                std::process::exit(exitcode::OSERR);
            });
        }

        while selecter.any() {
            match selecter.select(None) {
                Ok(_) => {
                    // select did something.  Keep going.
                },
                Err(ref err) if err.kind() == std::io::ErrorKind::Interrupted => {
                    // select interrupted, possibly by SIGCHLD.  Keep going.
                },
                Err(err) => {
                    panic!("selected failed: {}", err)
                },
            };
        };

        // Might have been interrupted by SIGCHLD.
        // FIXME: Errors.
        let (wait_pid, status, rusage) = match sys::wait4(child_pid, true) {
            Ok(Some(r)) => r,
            Ok(None) => panic!("wait4 empty result"),
            Err(err) => panic!("wait4 failed: {}", err),
        };
        assert_eq!(wait_pid, child_pid);  // FIXME: Errors.

        let mut result = res::Res::new();
        let mut proc_res = res::ProcRes::new(child_pid, status, rusage);

        for fd in &mut fds {
            match (*fd).clean_up_in_parent(&mut selecter) {
                Ok(Some(fd_result)) => {
                    proc_res.fds.insert(ir::fd::get_fd_name(fd.get_fd()), fd_result);
                }
                Ok(None) => {
                },
                Err(err) => {
                    result.errors.push(
                        format!("failed to clean up fd {}: {}", fd.get_fd(), err)
                    );
                    proc_res.fds.insert(ir::fd::get_fd_name(fd.get_fd()), res::FdRes::None {});
                },
            }
        }

        result.procs.push(proc_res);

        res::print(&result);
        println!("");
    }

    std::process::exit(exitcode::OK);
}

