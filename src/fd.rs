pub mod spec {

    use serde::{Serialize, Deserialize};
    use std::path::PathBuf;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub enum OpenFlag {
        // FIXME: Generalize.

        /// Read for stdin, Write for stdout/stderr, ReadWrite for others.
        Default,  

        Read,
        Write,
        Append,
        ReadWrite,
    }

    impl Default for OpenFlag {
        fn default() -> Self { Self::Default }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all="lowercase")]
    pub enum Fd {
        Inherit,
        Close,
        Null { flags: OpenFlag },
        File { path: PathBuf, flags: OpenFlag },
    }

    impl Default for Fd {
        fn default() -> Self { Self::Inherit }
    }

}

//------------------------------------------------------------------------------

use std::boxed::Box;
use std::io;
use std::path::Path;
use crate::sys;
use crate::sys::fd_t;
use libc;

// FIXME: Generalize.
fn get_oflags(flags: &spec::OpenFlag, fd: fd_t) -> libc::c_int {
    use spec::OpenFlag::*;
    match flags {
        Default => match fd {
            0       => libc::O_RDONLY,
            1 | 2   => libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            _       => libc::O_RDWR   | libc::O_CREAT | libc::O_TRUNC,
        },
        Read        => libc::O_RDONLY,
        Write       => libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
        Append      => libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND,
        ReadWrite   => libc::O_RDWR   | libc::O_CREAT | libc::O_TRUNC,
    }
}

pub trait Fd {
}

struct Inherit {
}

impl Fd for Inherit {
}

impl Inherit {
    fn new(_fd: fd_t) -> io::Result<Inherit> { Ok(Inherit {}) }
}

struct Close {
}

impl Close {
    fn new(fd: fd_t) -> io::Result<Close> {
        sys::close(fd)?;
        Ok(Close {})
    }
}

impl Fd for Close {
}

struct File {
}

impl File {
    fn new(fd: fd_t, path: &Path, flags: &spec::OpenFlag) -> io::Result<File> {
        let file_fd = sys::open(path, get_oflags(flags, fd))?;
        sys::dup2(file_fd, fd)?;
        sys::close(file_fd)?;
        Ok(File {})
    }
}

impl Fd for File {
}

pub fn create_fd(fd: fd_t, fd_spec: &spec::Fd) -> io::Result<Box<dyn Fd>> {
    Ok(match fd_spec {
        spec::Fd::Inherit
            => Box::new(Inherit::new(fd)?),
        spec::Fd::Close
            => Box::new(Close::new(fd)?),
        spec::Fd::Null { flags }
            => Box::new(File::new(fd, Path::new("/dev/null"), flags)?),
        spec::Fd::File { path, flags }
            => Box::new(File::new(fd, path, flags)?),
    })
}

