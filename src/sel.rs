use crate::sys;
use crate::sys::{FdSet, fd_t};
use std::collections::BTreeMap;
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
    fn ready(&mut self, fd: fd_t) {
        let size = 8;

        match self { 
            Reader::Capture { buf } => {
                sys::read(fd, buf, size).expect("read from df");
            }
        }
    }
}

//------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Selecter {
    readers: BTreeMap<fd_t, Reader>,
}

impl Selecter {
    pub fn new() -> Selecter {
        Selecter {
            ..Default::default()
        }
    }

    pub fn insert_reader(&mut self, fd: fd_t, reader: Reader) {
        self.readers.insert(fd, reader);
    }

    pub fn remove_reader(&mut self, fd: fd_t) -> Reader {
        self.readers.remove(&fd).unwrap()
    }

    pub fn select(&mut self, timeout: Option<f64>) -> io::Result<()> {
        // FIXME: Don't rebuild fd sets every time.
        let mut read_fds = FdSet::new();
        for (fd, _) in self.readers.iter() {
            eprintln!("select read fd: {}", fd);
            read_fds.set(*fd);
        }
        let mut write_fds = FdSet::new();
        let mut error_fds = FdSet::new();
        sys::select(&mut read_fds, &mut write_fds, &mut error_fds, timeout)?;

        for (fd, reader) in self.readers.iter_mut() {
            eprintln!("checking read ready: {}", fd);
            if read_fds.is_set(*fd) {
                eprintln!("fd is read ready: {}", fd);
                reader.ready(*fd);
            }
        }

        Ok(())
    }
}

