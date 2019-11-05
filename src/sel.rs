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
    Capture { buf: Vec<u8> },
}

impl Reader {
    fn ready(&mut self, fd: fd_t) -> bool {
        let size = 1024;

        match self { 
            Reader::Capture { buf } => {
                let nread = sys::read(fd, buf, size).expect("read from df");
                nread > 0
            }
        }
    }
}

//------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Selecter {
    readers: BTreeMap<fd_t, Reader>,
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
        // FIXME: Don't rebuild fd sets every time.
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

