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

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "lowercase")]
    pub enum CaptureMode {
        TempFile,
        Memory,
    }

    impl Default for CaptureMode {
        fn default() -> Self { Self::TempFile }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "lowercase")]
    pub enum CaptureFormat {
        Text,
        // FIXME: Raw... base64?
    }

    impl Default for CaptureFormat {
        fn default() -> Self { Self::Text }
    }

    fn get_default_mode() -> c_int { 0o666 }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "lowercase")]
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
            #[serde(default)]
            mode: CaptureMode,

            #[serde(default)]
            format: CaptureFormat,
        },

    }

    impl Default for Fd {
        fn default() -> Self { Self::Inherit }
    }

}

//------------------------------------------------------------------------------

use crate::res::FdRes;
use crate::sel::{Reader, Selecter};
use crate::sys;
use crate::sys::fd_t;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::os::unix::io::FromRawFd;
use std::path::PathBuf;
use libc;

//------------------------------------------------------------------------------

// FIXME: Hoist this into a top-level union-of-all Error type?

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    ParseInt(std::num::ParseIntError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::ParseInt(ref err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::ParseInt(ref err) => err.description(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error::ParseInt(err)
    }
}

type Result<T> = std::result::Result<T, Error>;

//------------------------------------------------------------------------------

pub fn parse_fd(fd: &str) -> std::result::Result<fd_t, std::num::ParseIntError> {
    match fd {
        "stdin" => Ok(0),
        "stdout" => Ok(1),
        "stderr" => Ok(2),
        _ => fd.parse::<fd_t>(),
    }
}

pub fn get_fd_name(fd: fd_t) -> String {
    match fd {
        0 => "stdin".to_string(),
        1 => "stdout".to_string(),
        2 => "stderr".to_string(),
        _ => fd.to_string(),
    }
}

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
    fn set_up(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Called after fork(), in parent process.
    fn set_up_in_parent(&mut self, _: &mut Selecter) -> io::Result<()> {
        Ok(())
    }

    /// Called after fork(), in child process.
    fn set_up_in_child(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Called in parent process after wait().
    fn clean_up_in_parent(&mut self, _: &mut Selecter) -> io::Result<(Option<FdRes>)> {
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

    fn clean_up_in_parent(&mut self, _: &mut Selecter) -> io::Result<(Option<FdRes>)> {
        Ok(Some(FdRes::File { path: self.path.clone() }))
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

    // FIXME: Insert path into result.
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

    fn set_up(&mut self) -> io::Result<()> {
        let (tmp_path, tmp_fd) = sys::mkstemp(TMP_TEMPLATE)?;
        std::fs::remove_file(tmp_path)?;
        self.tmp_fd = tmp_fd;
        Ok(())
    }

    fn set_up_in_child(&mut self) -> io::Result<()> {
        sys::dup2(self.tmp_fd, self.fd)?;
        sys::close(self.tmp_fd)?;
        self.tmp_fd = -1;
        Ok(())
    }

    fn clean_up_in_parent(&mut self, _: &mut Selecter) -> io::Result<(Option<FdRes>)> {
        let mut file = unsafe {
            let file = std::fs::File::from_raw_fd(self.tmp_fd);
            self.tmp_fd = -1;
            file
        };
        file.seek(std::io::SeekFrom::Start(0))?;
        let mut reader = std::io::BufReader::new(file);

        let mut bytes: Vec<u8> = Vec::new();
        let _size = reader.read_to_end(&mut bytes)?;
        let text = String::from_utf8_lossy(&bytes).into_owned();

        Ok(Some(FdRes::Capture { text }))
    }
}

//------------------------------------------------------------------------------

pub struct MemoryCapture {
    /// Proc-visible fd.
    fd: fd_t,

    /// Read end of the pipe.
    read_fd: fd_t,

    /// Write end of the pipe.
    write_fd: fd_t,
}

impl MemoryCapture {
    fn new(fd: fd_t) -> MemoryCapture {
        MemoryCapture {
            fd: fd,
            read_fd: -1,
            write_fd: -1
        }
    }
}

impl Fd for MemoryCapture {
    fn get_fd(&self) -> fd_t {
        self.fd
    }

    fn set_up(&mut self) -> io::Result<()> {
        let (read_fd, write_fd) = sys::pipe()?;
        self.read_fd = read_fd;
        self.write_fd = write_fd;
        Ok(())
    }

    fn set_up_in_parent(&mut self, selecter: &mut Selecter) -> io::Result<()> {
        // Close the write end of the pipe.  Only the child writes.
        sys::close(self.write_fd)?;

        // Set up to read from the pipe.
        selecter.insert_reader(
            self.read_fd, 
            Reader::Capture { buf: Vec::new() },
        );

        Ok(())
    }

    fn set_up_in_child(&mut self) -> io::Result<()> {
        sys::close(self.read_fd)?;
        sys::dup2(self.write_fd, self.fd)?;
        Ok(())
    }

    /// Called in parent process after wait().
    fn clean_up_in_parent(&mut self, selecter: &mut Selecter) -> io::Result<(Option<FdRes>)> {
        match selecter.remove_reader(self.read_fd) {
            Reader::Capture { mut buf } => {
                let mut buffer = Vec::new();
                std::mem::swap(&mut buffer, &mut buf);
                let text = String::from_utf8(buffer).unwrap();  // FIXME
                Ok(Some(FdRes::Capture { text }))
            },
        }
    }
}

//------------------------------------------------------------------------------

pub fn create_fd(fd: fd_t, fd_spec: &spec::Fd) -> Result<Box<dyn Fd>> {
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
        spec::Fd::Capture { mode, format: _format }
            => match mode {
                spec::CaptureMode::TempFile
                    => Box::new(TempFileCapture::new(fd)),
                spec::CaptureMode::Memory
                    => Box::new(MemoryCapture::new(fd)),
            },
    })
}

