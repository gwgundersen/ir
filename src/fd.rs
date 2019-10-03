pub mod spec {

    use crate::sys::fd_t;
    use libc::c_int;
    use serde::{Serialize, Deserialize};
    use std::path::PathBuf;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub enum OpenFlag {
        // FIXME: Generalize.

        /// Equivalent to `Read` for stdin, `Write` for stdout/stderr,
        /// `ReadWrite` for others.
        Default,  

        /// Open existing file for reading.
        Read,
        /// Create or open exsting file for writing.
        Write,
        /// Create a new file for writing; file may not exist.
        Create,
        /// Overwrite an existing file for writing; file must exist.
        Replace,
        /// Create or open an existing file for appending.
        CreateAppend,
        /// Open an existing file for appending.
        Append,
        /// Create or open existing file for reading and writing.
        ReadWrite,
    }

    impl Default for OpenFlag {
        fn default() -> Self { Self::Default }
    }

    fn get_default_mode() -> c_int {
        0o666
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all="lowercase")]
    pub enum Fd {
        /// Inherit this fd from the parent process, if any.
        Inherit,

        /// Close this fd, if it's open.
        Close,

        /// Open this fd to /dev/null.
        Null {
            #[serde(default)]
            flags: OpenFlag,
        },

        /// Open this fd to a file.
        File { 
            path: PathBuf,
            #[serde(default)]
            flags: OpenFlag,
            #[serde(default = "get_default_mode")]
            mode: c_int,
            // format
        },

        /// Duplicate another existing fd to this one.
        Dup {
            fd: fd_t
        },
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
            0           => libc::O_RDONLY,
            1 | 2       => libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            _           => libc::O_RDWR   | libc::O_CREAT | libc::O_TRUNC,
        },
        Read            => libc::O_RDONLY,
        Write           => libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
        Create          => libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
        Replace         => libc::O_WRONLY                 | libc::O_TRUNC,
        Append          => libc::O_WRONLY                 | libc::O_APPEND,
        CreateAppend    => libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND,
        ReadWrite       => libc::O_RDWR   | libc::O_CREAT | libc::O_TRUNC,
    }
}

pub trait Fd {
}

//------------------------------------------------------------------------------

struct Inherit {
}

impl Fd for Inherit {
}

impl Inherit {
    fn new(_fd: fd_t) -> io::Result<Inherit> { Ok(Inherit {}) }
}

//------------------------------------------------------------------------------

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

//------------------------------------------------------------------------------

struct File {
}

impl File {
    fn new(fd: fd_t, path: &Path, flags: &spec::OpenFlag, mode: libc::c_int)
           -> io::Result<File>
    {
        let file_fd = sys::open(path, get_oflags(flags, fd), mode)?;
        sys::dup2(file_fd, fd)?;
        sys::close(file_fd)?;
        Ok(File {})
    }
}

impl Fd for File {
}

//------------------------------------------------------------------------------

struct Dup {
}

impl Dup {
    fn new(fd: fd_t, other_fd: fd_t) -> io::Result<Dup> {
        sys::dup2(other_fd, fd)?;
        Ok(Dup {})
    }
}

impl Fd for Dup {
}

//------------------------------------------------------------------------------

pub fn create_fd(fd: fd_t, fd_spec: &spec::Fd) -> io::Result<Box<dyn Fd>> {
    Ok(match fd_spec {
        spec::Fd::Inherit
            => Box::new(Inherit::new(fd)?),
        spec::Fd::Close
            => Box::new(Close::new(fd)?),
        spec::Fd::Null { flags }
            => Box::new(File::new(fd, Path::new("/dev/null"), flags, 0)?),
        spec::Fd::File { path, flags, mode }
            => Box::new(File::new(fd, path, flags, *mode)?),
        spec::Fd::Dup { fd: other_fd }
            => Box::new(Dup::new(fd, *other_fd)?),
    })
}

