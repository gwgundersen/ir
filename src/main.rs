extern crate exitcode;

// Used for tests.
#[allow(unused_imports)]
#[macro_use] extern crate maplit;

use ir::environ;
use ir::err::Error;
use ir::fd::Fd;
use ir::fd::parse_fd;
use ir::fdio;
use ir::res;
use ir::sel;
use ir::sig;
use ir::spec;
use ir::sys;
use ir::sys::fd_t;
use libc::pid_t;
use std::collections::BTreeMap;

// State related to a running proc.
struct Proc {
    pub env: environ::Env,
    pub fds: Vec<Box<dyn Fd>>,
    pub pid: pid_t,
    pub wait: Option<(libc::c_int, libc::rusage)>,
    pub fd_res: BTreeMap<String, res::FdRes>,
}

impl Proc {
    pub fn new(env: environ::Env, fds: Vec<Box<dyn Fd>>, pid: pid_t) -> Self {
        Self {env, fds, pid, wait: None, fd_res: BTreeMap::new()}
    }
}

struct ErrPipeRead {
    fd: fd_t,
    errs: Vec<String>,
}

impl sel::Read for ErrPipeRead {
    fn get_fd(&self) -> fd_t {
        self.fd
    }

    fn read(&mut self) -> bool {
        let err = match fdio::read_str(self.fd) {
            Ok(str) => str,
            Err(Error::Eof) => return true,
            Err(err) => panic!("error: {}", err),
        };
        self.errs.push(err);
        false
    }
}

struct ErrPipeWrite {
    fd: fd_t,
}

impl ErrPipeWrite {
    pub fn send(&self, err: &str) {
        fdio::write_str(self.fd, err).unwrap();
    }
}

fn new_err_pipe() -> (ErrPipeRead, ErrPipeWrite) {
    let (read_fd, write_fd) = sys::pipe().unwrap_or_else(|err| {
        eprintln!("failed to create err pipe: {}", err);
        std::process::exit(exitcode::OSERR);
    });
    let err_read = ErrPipeRead {fd: read_fd, errs: Vec::new()};
    let err_write = ErrPipeWrite {fd: write_fd};
    (err_read, err_write)
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

    // Set up the selector, which will manage events while the child runs.
    let mut select = sel::Select::new();

    // Build pipe for passing errors from child to parent.
    let (mut err_read, err_write) = new_err_pipe();
    // Read errors from the error pipe.
    select.insert_reader(&mut err_read);

    let mut procs = BTreeMap::<pid_t, Proc>::new();
    for spec in input.procs {
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

            // Close the read end of the error pipe.
            sys::close(err_read.fd).unwrap();
            let mut ok = true;

            for fd in &mut fds {
                (*fd).set_up_in_child().unwrap_or_else(|err| {
                    err_write.send(&format!("failed to set up fd {}: {}", fd.get_fd(), err));
                    ok = false;
                });
            }
            if !ok {
                std::process::exit(exitcode::OSERR);
            }

            let exe = &spec.argv[0];
            let err = sys::execve(exe.clone(), spec.argv.clone(), env).unwrap_err();
            // If we got here, exec failed; send the error to the parent process.
            err_write.send(&format!("exec: {}: {}", exe, err));
            ok = false;

            for fd in &mut fds {
                (*fd).clean_up_in_child().unwrap_or_else(|err| {
                    err_write.send(&format!("failed to clean up fd {}: {}", fd.get_fd(), err));
                    ok = false;
                });
            }

            std::process::exit(if ok { exitcode::OK } else { exitcode::OSERR });
        }

        else {
            // Parent process.  Construct the record of this running proc.
            procs.insert(child_pid, Proc::new(env, fds, child_pid));
        }
    }

    for proc in procs.values_mut() {
        // Set up file descriptors.
        for fd in &mut proc.fds {
            let f = fd.get_fd();
            match (*fd).set_up_in_parent() {
                Err(err) => result.errors.push(format!("failed to set up fd {}: {}", f, err)),
                Ok(None) => (),
                Ok(Some(read)) => select.insert_reader(read),
            };
        }
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
    sys::close(err_write.fd).unwrap();

    // FIXME: Clean up fds as they close, rather than all at once.
    // FIXME: Merge select loop and wait loop, by handling SIGCHLD.
    while select.any() {
        match select.select(None) {
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

    for (_pid_t, proc) in &mut procs {
        for fd in &mut proc.fds {
            match (*fd).clean_up_in_parent() {
                Ok(Some(fd_result)) => {
                    proc.fd_res.insert(ir::fd::get_fd_name(fd.get_fd()), fd_result);
                }
                Ok(None) => {
                },
                Err(err) => {
                    proc.fd_res.insert(ir::fd::get_fd_name(fd.get_fd()), res::FdRes::Error {});
                    result.errors.push(format!("failed to clean up fd {}: {}", fd.get_fd(), err));
                },
            }
        }
    }

    let mut num_running = procs.len();
    while num_running > 0 {
        let (wait_pid, status, rusage) = match sys::wait4(-1, true) {
            Ok(Some(r)) => r,
            Ok(None) => panic!("wait4 empty result"),
            Err(ref err) if err.kind() == std::io::ErrorKind::Interrupted => {
                // wait4 interrupted, possibly by SIGCHLD.  Keep going.
                continue;
            },
            Err(err) => panic!("wait4 failed: {}", err),
        };

        let proc = match procs.get_mut(&wait_pid) {
            Some(p) => p,
            None => {
                // FIXME: Nothing wrong with this.
                eprintln!("wait4 returned unexpected pid: {}", wait_pid);
                continue;
            }
        };
        debug_assert!(proc.wait.is_none(), "proc already waited");
        proc.wait = Some((status, rusage));
        num_running -= 1;
    }

    // Collect proc results.
    result.procs = procs.into_iter().map(|(_, proc)| {
        let (status, rusage) = proc.wait.unwrap();
        let mut proc_res = res::ProcRes::new(proc.pid, status, rusage);
        proc_res.fds = proc.fd_res;
        proc_res
    }).collect();

    // Transfer errors retrieved from the error pipe buffer into results.
    result.errors.append(&mut err_read.errs);

    res::print(&result);
    println!("");

    std::process::exit(if result.errors.len() > 0 { 1 } else { exitcode::OK });
}

