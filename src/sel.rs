use crate::sys;
use crate::sys::{FdSet, fd_t};
use std::collections::{BTreeMap, HashSet};
use std::io;
use std::vec::Vec;

//------------------------------------------------------------------------------

// In C++, I would provide different selectables with dynamic dispatch.  Here,
// we're not writing a library, and the number of select behaviors will be
// small, so let's see how it works to use an enum instead.

#[derive(Debug)]
pub enum Reader {
    Errors { errs: Vec<String> },
    Capture { buf: Vec<u8> },
}

impl Reader {
    fn ready(&mut self, fd: fd_t) -> bool {
        let size = 1024;

        match self { 
            Reader::Errors { errs } => {
                let mut err = Vec::new();
                let nread = sys::read(fd, &mut err, 65536)
                    .expect("read err from fd");
                if nread > 0 {
                    errs.push(String::from_utf8_lossy(&err).to_string());
                }
                eprintln!("Reader::Errors read {}", nread);
                nread > 0
            },

            Reader::Capture { buf } => {
                let nread = sys::read(fd, buf, size).expect("read from fd");
                nread > 0
            },
        }
    }
}

//------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Selecter {
    readers: BTreeMap<fd_t, Reader>,  // FIXME: Vec<(fd_t, Reader)> ?
    read_fds: HashSet<fd_t>,
}

impl Selecter {
    pub fn new() -> Selecter {
        Selecter {
            ..Default::default()
        }
    }

    pub fn any(&self) -> bool {
        self.read_fds.len() > 0
    }

    pub fn insert_reader(&mut self, fd: fd_t, reader: Reader) {
        self.readers.insert(fd, reader);
        self.read_fds.insert(fd);
    }

    pub fn remove_reader(&mut self, fd: fd_t) -> Reader {
        self.read_fds.remove(&fd);
        self.readers.remove(&fd).unwrap()
    }

    pub fn select(&mut self, timeout: Option<f64>) -> io::Result<()> {
        let mut read_set = FdSet::new();
        for fd in self.read_fds.iter() {
            read_set.set(*fd);
        }
        let mut write_set = FdSet::new();
        let mut error_set = FdSet::new();
        sys::select(&mut read_set, &mut write_set, &mut error_set, timeout)?;

        for (fd, reader) in self.readers.iter_mut() {
            if read_set.is_set(*fd) && ! reader.ready(*fd) {
                self.read_fds.remove(fd);
            }
        }

        Ok(())
    }
}

