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

fn empty_sigaction() -> libc::sigaction
{
    libc::sigaction {
        sa_sigaction: libc::SIG_DFL,
        sa_mask: 0,
        sa_flags: 0,
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

