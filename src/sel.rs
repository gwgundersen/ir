use crate::sys::{FdSet, fd_t};
use std::vec::Vec;

//------------------------------------------------------------------------------

// In C++, I would provide different selectables with dynamic dispatch.  Here,
// we're not writign a library, and the number of select behaviors will be
// small, so let's see how it works to use an enum instead.

#[derive(Debug)]
pub enum Reader {
    Capture { buf: Vec<u8> },
}

impl Reader {
    fn ready(&mut self) {
    }
}

//------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Selecter {
    readers: Vec<(fd_t, Reader)>,
}

impl Selecter {
    pub fn new() -> Selecter {
        Selecter {
            ..Default::default()
        }
    }

    pub fn add_read(&mut self, fd: fd_t, reader: Reader) {
        self.readers.push((fd, reader));
    }

    pub fn select(&self, timeout: f64) {
        let mut read_fds = FdSet::new();
        for (fd, _) in self.readers.iter() {
            read_fds.set(*fd);
        }
    }
}

