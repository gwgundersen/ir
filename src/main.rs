extern crate exitcode;

// Used for tests.
#[allow(unused_imports)]
#[macro_use] extern crate maplit;

use ir::environ;
use ir::fd::Fd;
use ir::fd::parse_fd;
use ir::fdio;
use ir::res;
use ir::sel;
use ir::sig;
use ir::spec;
use ir::sys;
use libc::pid_t;
use std::collections::BTreeMap;

// State related to a running proc.
#[derive(Default)]
struct Proc {
    pub order: usize,
    pub env: environ::Env,
    pub fds: Vec<Box<dyn Fd>>,
    pub pid: pid_t,
}

fn main() {
    let json_path = match std::env::args().skip(1).next() {
        Some(p) => p,
        None => panic!("no file given"),  // FIXME
    };


    let input = spec::load_file(&json_path).unwrap_or_else(|err| {
        eprintln!("failed to load {}: {}", json_path, err);
        std::process::exit(exitcode::OSFILE);
    });
    eprintln!("input: {:?}", input);
    eprintln!("");

    let mut result = res::Res::new();

    // Build pipe for passing errors from child to parent.
    let (err_read_fd, err_write_fd) = sys::pipe().unwrap_or_else(|err| {
        eprintln!("failed to create err pipe: {}", err);
        std::process::exit(exitcode::OSERR);
    });

    // Set up the selector, which will manage events while the child runs.
    let mut selecter = sel::Selecter::new();
    // Read errors from the error pipe.
    selecter.insert_reader(
        err_read_fd, sel::Reader::Errors { errs: Vec::new() });

    let mut procs = BTreeMap::<pid_t, Proc>::new();
    for (order, spec) in input.procs.iter().enumerate() {
        let env = environ::build(std::env::vars(), &spec.env);

        // Build fd managers.
        let mut fds = spec.fds.iter().map(|(fd_str, fd_spec)| {
            // FIXME: Parse when deserializing, rather than here.
            let fd_num = parse_fd(fd_str).unwrap_or_else(|err| {
                eprintln!("failed to parse fd {}: {}", fd_str, err);
                std::process::exit(exitcode::OSERR);
            });

            // FIXME: Errors.
            ir::fd::create_fd(fd_num, &fd_spec).unwrap()
        }).collect::<Vec<_>>();

        // Fork the child process.
        let child_pid = sys::fork().unwrap_or_else(|err| {
            panic!("failed to fork: {}", err);
        });

        if child_pid == 0 {
            // Child process.

            // Send errors in the child process to the parent via the pipe.
            let error = |err: &str| {
                fdio::write_str(err_write_fd, err).unwrap();
            };

            // Close the read end of the error pipe.
            sys::close(err_read_fd).unwrap();
            let mut ok = true;

            for fd in &mut fds {
                (*fd).set_up_in_child().unwrap_or_else(|err| {
                    error(&format!("failed to set up fd {}: {}", fd.get_fd(), err));
                    ok = false;
                });
            }
            if !ok {
                std::process::exit(exitcode::OSERR);
            }

            let exe = &spec.argv[0];
            let err = sys::execve(exe.clone(), spec.argv.clone(), env).unwrap_err();
            // If we got here, exec failed; send the error to the parent process.
            error(&format!("exec: {}: {}", exe, err));
            ok = false;

            for fd in &mut fds {
                (*fd).clean_up_in_child().unwrap_or_else(|err| {
                    error(&format!("failed to clean up fd {}: {}", fd.get_fd(), err));
                    ok = false;
                });
            }

            std::process::exit(if ok { exitcode::OK } else { exitcode::OSERR });
        }

        // Parent process.

        for fd in &mut fds {
            (*fd).set_up_in_parent(&mut selecter).unwrap_or_else(|err| {
                result.errors.push(format!("failed to set up fd {}: {}", fd.get_fd(), err));
            });
        }

        procs.insert(child_pid, Proc {order, env, fds, pid: child_pid});
    }

    static mut SIGCHLD_FLAG: bool = false;

    extern "system" fn sigchld_handler(signum: libc::c_int) {
        eprintln!("sigchld handler: {}", signum);
        unsafe { SIGCHLD_FLAG = true; }
    }

    // Set up SIGCHLD handler.
    sig::sigaction(libc::SIGCHLD, Some(sig::Sigaction {
        disposition: sig::Sigdisposition::Handler(sigchld_handler),
        mask: 0,
        flags: libc::SA_NOCLDSTOP,
    })).unwrap_or_else(|err| {
        eprintln!("sigaction failed: {}", err);
        std::process::exit(exitcode::OSERR);
    });

    // Close the write end of the error pipe.
    sys::close(err_write_fd).unwrap();

    // FIXME: Merge select loop and wait loop, by handling SIGCHLD.

    while selecter.any() {
        match selecter.select(None) {
            Ok(_) => {
                // select did something.  Keep going.
            },
            Err(ref err) if err.kind() == std::io::ErrorKind::Interrupted => {
                // select interrupted, possibly by SIGCHLD.  Keep going.
            },
            Err(err) => {
                panic!("select failed: {}", err)
            },
        };
    };

    // Create an empty vector of proc results.  We'll fill in the results in the
    // same order as the original procs.
    let mut proc_ress = Vec::new();
    proc_ress.resize_with(procs.len(), || { None });

    while procs.len() > 0 {
        let (wait_pid, status, rusage) = match sys::wait4(-1, true) {
            Ok(Some(r)) => r,
            Ok(None) => panic!("wait4 empty result"),
            // FIXME: Handle EINTR.
            Err(err) => panic!("wait4 failed: {}", err),
        };

        let mut proc = match procs.remove(&wait_pid) {
            Some(p) => p,
            None => {
                // FIXME: Nothing wrong with this.
                eprintln!("wait4 returned unexpected pid: {}", wait_pid);
                continue;
            }
        };

        let mut proc_res = res::ProcRes::new(proc.pid, status, rusage);

        for fd in &mut proc.fds {
            match (*fd).clean_up_in_parent(&mut selecter) {
                Ok(Some(fd_result)) => {
                    proc_res.fds.insert(ir::fd::get_fd_name(fd.get_fd()), fd_result);
                }
                Ok(None) => {
                },
                Err(err) => {
                    proc_res.fds.insert(ir::fd::get_fd_name(fd.get_fd()), res::FdRes::None {});
                    result.errors.push(format!("failed to clean up fd {}: {}", fd.get_fd(), err));
                },
            }
        }

        proc_ress[proc.order] = Some(proc_res);
    }

    // Collect proc results in the correct order.
    result.procs = proc_ress.into_iter().map(|r| { r.unwrap() }).collect();

    // Transfer errors retrieved from the error pipe buffer into results.
    for err in match selecter.remove_reader(err_read_fd) {
        sel::Reader::Errors { errs } => errs,
        _ => panic!("wrong sel for error pipe"),
    } {
        result.errors.push(err);
    }

    res::print(&result);
    println!("");

    std::process::exit(if result.errors.len() > 0 { 1 } else { exitcode::OK });
}

