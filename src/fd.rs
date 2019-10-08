pub mod spec {

    use crate::sys::fd_t;
    use libc::c_int;
    use serde::{Serialize, Deserialize};
    use std::path::PathBuf;

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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

        /// Capture output from fd; include in results.
        Capture {
            // format: raw or text (encoding?)
        },

    }

    impl Default for Fd {
        fn default() -> Self { Self::Inherit }
    }

}

//------------------------------------------------------------------------------

use std::boxed::Box;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::os::unix::io::FromRawFd;
use std::path::PathBuf;
use crate::result::FdResult;
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
    fn get_fd(&self) -> fd_t;

    /// Called before fork().
    fn set_up_in_parent(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Called after fork(), in child process.
    fn set_up_in_child(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Called in parent process after wait().
    fn clean_up_in_parent(&mut self) -> io::Result<(Option<FdResult>)> {
        Ok(None)
    }

    /// Called only if exec() fails.
    fn clean_up_in_child(&mut self) -> io::Result<()> {
        Ok(())
    }
}

//------------------------------------------------------------------------------

struct Inherit {
    fd: fd_t,
}

impl Fd for Inherit {
    fn get_fd(&self) -> fd_t { self.fd }
}

impl Inherit {
    fn new(fd: fd_t) -> Inherit { Inherit { fd } }
}

//------------------------------------------------------------------------------

struct Close {
    fd: fd_t,
}

impl Close {
    fn new(fd: fd_t) -> Close {
        Close { fd }
    }
}

impl Fd for Close {
    fn get_fd(&self) -> fd_t { self.fd }

    fn set_up_in_child(&mut self) -> io::Result<()> {
        sys::close(self.fd)?;
        Ok(())
    }
}

//------------------------------------------------------------------------------

struct File {
    fd: fd_t,
    path: PathBuf,
    oflags: libc::c_int,
    mode: libc::c_int,
}

impl File {
    fn new(fd: fd_t, path: PathBuf, flags: spec::OpenFlag, mode: libc::c_int)
        -> File
    {
        File { fd, path, oflags: get_oflags(&flags, fd), mode }
    }
}
        

impl Fd for File {
    fn get_fd(&self) -> fd_t { self.fd }

    fn set_up_in_child(&mut self) -> io::Result<()>
    {
        let file_fd = sys::open(&self.path, self.oflags, self.mode)?;
        sys::dup2(file_fd, self.fd)?;
        sys::close(file_fd)?;
        Ok(())
    }
}

//------------------------------------------------------------------------------

struct Dup {
    fd: fd_t,
    /// File descriptor that will be duplicated.
    dup_fd: fd_t,
}

impl Dup {
    fn new(fd: fd_t, dup_fd: fd_t) -> Dup {
        Dup { fd, dup_fd }
    }
}

impl Fd for Dup {
    fn get_fd(&self) -> fd_t { self.fd }

    fn set_up_in_child(&mut self) -> io::Result<()> {
        sys::dup2(self.dup_fd, self.fd)?;
        Ok(())
    }
}

//------------------------------------------------------------------------------

struct TempFileCapture {
    fd: fd_t,
    tmp_fd: fd_t,
}

// FIXME: Template.
const TMP_TEMPLATE: &str = "/tmp/ir-capture-XXXXXXXXXXXX";

impl TempFileCapture {
    fn new(fd: fd_t) -> TempFileCapture {
        TempFileCapture { fd, tmp_fd: -1 }
    }
}

impl Fd for TempFileCapture {
    fn get_fd(&self) -> fd_t { self.fd }

    fn set_up_in_parent(&mut self) -> io::Result<()> {
        let (tmp_path, tmp_fd) = sys::mkstemp(TMP_TEMPLATE)?;
        eprintln!("capturing {} to {} (unlinked)", self.fd, tmp_path.to_str().unwrap());
        std::fs::remove_file(tmp_path)?;
        self.tmp_fd = tmp_fd;
        Ok(())
    }

    fn set_up_in_child(&mut self) -> io::Result<()> {
        eprintln!("duping {} from tmp fd", self.fd);
        sys::dup2(self.tmp_fd, self.fd)?;
        sys::close(self.tmp_fd)?;
        self.tmp_fd = -1;
        Ok(())
    }

    fn clean_up_in_parent(&mut self) -> io::Result<(Option<FdResult>)> {
        let mut file = unsafe {
            let file = std::fs::File::from_raw_fd(self.tmp_fd);
            self.tmp_fd = -1;
            file
        };
        file.seek(std::io::SeekFrom::Start(0))?;
        let mut reader = std::io::BufReader::new(file);

        let mut bytes: Vec<u8> = Vec::new();
        let size = reader.read_to_end(&mut bytes)?;
        let string = String::from_utf8_lossy(&bytes).into_owned();
        eprintln!("read {} bytes from temp file", size);

        Ok(Some(FdResult::Capture { string }))
    }
}

//------------------------------------------------------------------------------

pub fn create_fd(fd: fd_t, fd_spec: &spec::Fd) -> io::Result<Box<dyn Fd>> {
    Ok(match fd_spec {
        spec::Fd::Inherit
            => Box::new(Inherit::new(fd)),
        spec::Fd::Close
            => Box::new(Close::new(fd)),
        spec::Fd::Null { flags }
            => Box::new(File::new(fd, PathBuf::from("/dev/null"), *flags, 0)),
        spec::Fd::File { path, flags, mode }
            => Box::new(File::new(fd, path.to_path_buf(), *flags, *mode)),
        spec::Fd::Dup { fd: other_fd }
            => Box::new(Dup::new(fd, *other_fd)),
        spec::Fd::Capture {}
            => Box::new(TempFileCapture::new(fd)),
    })
}

