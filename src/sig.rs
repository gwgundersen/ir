extern crate libc;

use std::io;

//------------------------------------------------------------------------------

// FIXME: ucontext_t?
// FIXME: Handler type has to be predicated on flags & SA_SIGINFO.
// void handler(int sig, siginfo_t *info, void *ucontext)
// type Sighandler = extern "system" fn(libc::c_int, *const libc::siginfo_t, *const libc::c_void);
type Sighandler = extern "system" fn(libc::c_int) -> ();

pub enum Sigdisposition {
    Default,
    Ignore,
    Handler(Sighandler),
}

pub struct Sigaction {
    pub disposition: Sigdisposition,
    pub mask: libc::sigset_t,
    pub flags: libc::c_int,
}

impl std::convert::Into<libc::sigaction> for Sigaction {
    fn into(self) -> libc::sigaction {
        libc::sigaction {
            sa_sigaction: match self.disposition {
                Sigdisposition::Default => libc::SIG_DFL,
                Sigdisposition::Ignore => libc::SIG_IGN,
                Sigdisposition::Handler(h) => h as libc::sighandler_t,
            },
            sa_mask: self.mask,
            sa_flags: self.flags,
#[cfg(target_os = "linux")]
            sa_restorer: None,
        }
    }
}

impl std::convert::From<libc::sigaction> for Sigaction {
    fn from(sa: libc::sigaction) -> Self {
        Self {
            disposition: match sa.sa_sigaction {
                libc::SIG_DFL => Sigdisposition::Default,
                libc::SIG_IGN => Sigdisposition::Ignore,
                h => Sigdisposition::Handler(unsafe {
                    std::mem::transmute::<libc::sighandler_t, Sighandler>(h)
                }),
            },
            mask: sa.sa_mask,
            flags: sa.sa_flags,
        }
    }
}

pub fn empty_sigset() -> libc::sigset_t
{
    unsafe { std::mem::zeroed() }
}

fn empty_sigaction() -> libc::sigaction
{
    libc::sigaction {
        sa_sigaction: libc::SIG_DFL,
        sa_mask: empty_sigset(),
        sa_flags: 0,
        sa_restorer: None,
    }
}

/// Sets and/or retrieves the signal action of `signum`.
///
/// If sigaction is `Some`, sets the signal action and returns the previous.  If
/// sigaction is `None`, retrieves the signal action without changing it.
pub fn sigaction(signum: libc::c_int, sigaction: Option<Sigaction>) -> io::Result<Sigaction>
{
    let act: libc::sigaction;
    let act_ptr = match sigaction {
        Some(sa) => {
            act = sa.into();
            &act
        },
        None => std::ptr::null(),
    };
    let mut old = empty_sigaction();
    match unsafe { libc::sigaction(signum, act_ptr, &mut old) } {
        -1 => Err(io::Error::last_os_error()),
        0 => Ok(Sigaction::from(old)),
        ret => panic!("sigaction returned {}", ret),
    }
}

//------------------------------------------------------------------------------

// FIXME: NSIG is not reliably available in libc.  I hope this is enough.
const NSIG: usize = 256;

pub struct SignalFlag {
    signum: usize,
}

static mut SIGNAL_FLAGS: [bool; NSIG] = [false; NSIG];

/// Hacky unsafe boolean flag for a signal.  Installs a signal handler that sets
/// the flag when the signal is received.
impl SignalFlag {
    pub fn new(signum: libc::c_int) -> Self {
        assert!(signum > 0);
        assert!(signum < NSIG as libc::c_int);
        
        extern "system" fn handler(signum: libc::c_int) {
            eprintln!("signal handler: {}", signum);
            // Accessing a static global is in general not threadsafe, but this
            // signal handler will only ever be called on the main thread.
            unsafe { SIGNAL_FLAGS[signum as usize] = true; }
        }

        // Set up the handler.
        // FIXME: Check that we're not colliding with an existing handler.
        sigaction(signum, Some(Sigaction {
            disposition: Sigdisposition::Handler(handler),
            mask: empty_sigset(),
            flags: libc::SA_NOCLDSTOP,
        })).unwrap_or_else(|err| {
            eprintln!("sigaction failed: {}", err);
            std::process::exit(exitcode::OSERR);
        });

        Self { signum: signum as usize }
    }

    /// Retrieves the flag value, and clears it.
    pub fn get(&self) -> bool {
        unsafe {
            let val = SIGNAL_FLAGS[self.signum];
            SIGNAL_FLAGS[self.signum] = false;
            val
        }
    }
}

