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

//------------------------------------------------------------------------------

// FIXME: Elsewhere.
fn wait(block: bool) -> Option<sys::WaitInfo> {
    loop {
        match sys::wait4(-1, block) {
            Ok(Some(ti)) =>
                return Some(ti),
            Ok(None) =>
                if block {
                    panic!("wait4 empty result");
                }
                else {
                    return None;
                },
            Err(ref err)if err.kind() == std::io::ErrorKind::Interrupted =>
                // wait4 interrupted, possibly by SIGCHLD.
                if block {
                    // Keep going.
                    continue;
                }
                else {
                    // Return, as the caller might want to do something.
                    return None;
                },
            Err(err) => panic!("wait4 failed: {}", err),
        };
    }
}

//------------------------------------------------------------------------------

/// A proc we're running, or that has terminated.
struct Proc {
    pub pid: pid_t,

    /// None while the proc is running; the result of wait4() once the proc has
    /// terminated and been cleaned up.
    pub wait_info: Option<sys::WaitInfo>,
}

struct Procs {
    procs: Vec<Proc>,
    num_running: usize,
}

impl Procs {
    pub fn new() -> Self {
        Self { procs: Vec::new(), num_running: 0 }
    }

    pub fn push(&mut self, pid: pid_t) {
        self.procs.push(Proc { pid, wait_info: None });
        self.num_running += 1;
    }

    /// Waits for procs to finish.  Waits all completed `procs` and stores their
    /// termination info.  If `block` is true and no procs are read to wait,
    /// blocks until one is.
    fn wait(&mut self, block: bool) {
        while self.num_running > 0 {
            if let Some(wait_info) = wait(block) {
                let pid = wait_info.0;
                let mut pid_found = false;
                for proc in &mut self.procs {
                    if proc.pid == pid {
                        assert!(proc.wait_info.replace(wait_info).is_none());
                        self.num_running -= 1;
                        pid_found = true;
                        break
                    }
                }
                assert!(pid_found, "wait returned unexpected pid: {}", pid);
            }
            else {
                assert!(! block, "blocking wait returned no pid");
                break
            }
        }
    }

    pub fn wait_any(&mut self) { self.wait(false); }
    pub fn wait_all(&mut self) { self.wait(true); }

    pub fn into_iter(self) -> std::vec::IntoIter<Proc> { self.procs.into_iter() }
}

//------------------------------------------------------------------------------

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

    // Build the objects presenting each of the file descriptors in each proc.
    let mut fds = input.procs.iter().map(|spec| {
        spec.fds.iter().map(|(fd_str, fd_spec)| {
            // FIXME: Parse when deserializing, rather than here.
            let fd_num = parse_fd(fd_str).unwrap_or_else(|err| {
                eprintln!("failed to parse fd {}: {}", fd_str, err);
                std::process::exit(exitcode::OSERR);
            });

            // FIXME: Errors.
            ir::fd::create_fd(fd_num, &fd_spec).unwrap()
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let mut procs = Procs::new();
    for (spec, proc_fds) in input.procs.into_iter().zip(fds.iter_mut()) {
        let env = environ::build(std::env::vars(), &spec.env);

        // Fork the child process.
        let child_pid = sys::fork().unwrap_or_else(|err| {
            panic!("failed to fork: {}", err);
        });

        if child_pid == 0 {
            // Child process.

            // Close the read end of the error pipe.
            sys::close(err_read.fd).unwrap();
            let mut ok = true;

            for fd in proc_fds.iter_mut() {
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

            for fd in proc_fds {
                (*fd).clean_up_in_child().unwrap_or_else(|err| {
                    err_write.send(&format!("failed to clean up fd {}: {}", fd.get_fd(), err));
                    ok = false;
                });
            }

            std::process::exit(if ok { exitcode::OK } else { exitcode::OSERR });
        }

        else {
            // Parent process.  Construct the record of this running proc.
            procs.push(child_pid);
        }
    }

    // FIXME: Abstract this away.
    static mut SIGCHLD_FLAG: bool = false;

    extern "system" fn sigchld_handler(signum: libc::c_int) {
        eprintln!("sigchld handler: {}", signum);
        // Accessing a static global is in general not threadsafe, but this
        // signal handler will only ever be called on the main thread.
        unsafe { SIGCHLD_FLAG = true; }
    }

    // Set up SIGCHLD handler.
    sig::sigaction(libc::SIGCHLD, Some(sig::Sigaction {
        disposition: sig::Sigdisposition::Handler(sigchld_handler),
        mask: sig::empty_sigset(),
        flags: libc::SA_NOCLDSTOP,
    })).unwrap_or_else(|err| {
        eprintln!("sigaction failed: {}", err);
        std::process::exit(exitcode::OSERR);
    });

    // Close the write end of the error pipe.
    sys::close(err_write.fd).unwrap();

    // Now we wait for the procs to run.

    // Clean up procs that might have completed already.
    procs.wait_any();

    // Finish setting up all file descriptors for all procs.
    for proc_fds in &mut fds {
        for fd in proc_fds {
            let f = fd.get_fd();
            match (*fd).set_up_in_parent() {
                Err(err) => result.errors.push(format!("failed to set up fd {}: {}", f, err)),
                Ok(None) => (),
                Ok(Some(read)) => select.insert_reader(read),
            };
        }
    }

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
        if unsafe { SIGCHLD_FLAG } {
            procs.wait_any();
        }
    };
    std::mem::drop(select);

    // Wait for all remaining procs to terminate.
    procs.wait_all();

    // Collect fd results for each proc.
    
    fn get_fd_res(fds: Vec<Box<dyn Fd>>) -> BTreeMap<String, res::FdRes> {
        let mut fd_res = BTreeMap::<String, res::FdRes>::new();
        for mut fd in fds {
            match fd.clean_up_in_parent() {
                Ok(Some(fd_result)) => {
                    fd_res.insert(ir::fd::get_fd_name(fd.get_fd()), fd_result);
                }
                Ok(None) => {
                },
                Err(err) => {
                    fd_res.insert(ir::fd::get_fd_name(fd.get_fd()), res::FdRes::Error {});
                    // FIXME: Restore this.
                    // result.errors.push(format!("failed to clean up fd {}: {}", fd.get_fd(), err));
                },
            };
        }
        fd_res
    }        

    // Collect proc results.
    result.procs = procs.into_iter()
        .zip(fds.into_iter())
        .map(|(proc, fds)| {
            let (_, status, rusage) = proc.wait_info.unwrap();
            let mut proc_res = res::ProcRes::new(proc.pid, status, rusage);
            proc_res.fds = get_fd_res(fds);
            proc_res
        }).collect();

    // Transfer errors retrieved from the error pipe buffer into results.
    result.errors.append(&mut err_read.errs);

    res::print(&result);
    println!("");

    std::process::exit(if result.errors.len() > 0 { 1 } else { exitcode::OK });
}

