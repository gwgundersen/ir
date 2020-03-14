use crate::err::Error;
use crate::fdio;
use crate::sys::{FdSet, fd_t, select};
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
    // FIXME: Rewrite this to return a Result, and handle Error::Eof in caller.
    fn ready(&mut self, fd: fd_t) -> bool {
        let size = 1024;

        match self { 
            Reader::Errors { errs } => {
                errs.push(
                    match fdio::read_str(fd) {
                        Ok(str) => str,
                        Err(Error::Eof) => return false,
                        Err(err) => panic!("error: {}", err),
                    }
                );
                true
            },

            Reader::Capture { buf } => {
                match fdio::read_into_vec(fd, buf, size) {
                    Ok(_) => true,
                    Err(Error::Eof) => false,
                    Err(err) => panic!("error: {}", err),
                }
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
        // FIXME: Pass through EINTR.
        select(&mut read_set, &mut write_set, &mut error_set, timeout)?;

        for (fd, reader) in self.readers.iter_mut() {
            if read_set.is_set(*fd) && ! reader.ready(*fd) {
                self.read_fds.remove(fd);
            }
        }

        Ok(())
    }
}

//------------------------------------------------------------------------------

// This is the new dyn trait-based select().  Implement `Read` for the fds
// that need it.

pub trait Read {
    /// Reads from `fd`, when a read is ready.  Returns true if the fd is
    /// complete and should no longer be selected.
    fn read(&mut self, fd: fd_t) -> bool;
}

pub struct Select<'a> {
    // We use a hash set rather than maintaining an FdSet directly because
    // different OSes have different semantics for how select() modifies its
    // fd_set and whether an fd_set can be copied.
    read_fds: HashSet<fd_t>,

    readers: BTreeMap<fd_t, &'a mut dyn Read>,
}

/// FIXME: Write, error not implememented.
impl<'a> Select<'a> {
    pub fn new() -> Self {
        Select {
            readers: BTreeMap::new(),
            read_fds: HashSet::new(),
        }
    }

    pub fn any(&self) -> bool {
        ! self.read_fds.is_empty()
    }

    pub fn insert_reader(&mut self, fd: fd_t, read: &'a mut dyn Read) {
        self.read_fds.insert(fd);
        self.readers.insert(fd, read);
    }

    pub fn remove_reader(&mut self, fd: fd_t) -> &'a mut dyn Read {
        self.read_fds.remove(&fd);
        self.readers.remove(&fd).unwrap()
    }

    /// Blocks until a file descriptor is ready, and processes any ready file
    /// descriptors.
    pub fn select(&mut self, timeout: Option<f64>) -> io::Result<()> {
        let mut read_set  = FdSet::from_fds(self.read_fds.iter().copied());
        let mut write_set = FdSet::new();
        let mut error_set = FdSet::new();
        select(&mut read_set, &mut write_set, &mut error_set, timeout)?;

        // Process read-ready fds.  Collect those that are done, collect them to
        // avoid inalidating the iter, then remove them.
        self.readers.iter_mut().filter_map(
            |(fd, reader)| {
                if read_set.is_set(*fd) && reader.read(*fd) {
                    Some(*fd)
                }
                else {
                    None
                }
            }
        ).collect::<Vec<_>>().into_iter().for_each(
            |fd| { self.remove_reader(fd); }
        );

        Ok(())
    }
}

