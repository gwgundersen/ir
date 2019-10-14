use crate::sys::fd_t;
use libc;
use std::box::Box;
use std::vec::Vec;

//------------------------------------------------------------------------------

pub enum SelectMode {
    Read,
    Write,
    Error,
}

pub trait Selectable {
    fn ready() -> ();
}

pub struct Selecter {
    selables: Vec<(fd_t, mode, Box<dyn Selectable>),
}

impl Selecter {
    pub fn add(fd: fd_t, mode: SelectMode, selable: Box<dyn Selectable>);
    pub fn remove(fd: fd_t, mode: SelectMode);
    pub fn select(timeout: f64) -> i32;
}

