use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::string::String;

use crate::environ;
use crate::fd;

#[derive(Debug)]
pub enum SpecError {
    Io(std::io::Error),
    Json(serde_json::error::Error),
}

impl std::fmt::Display for SpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SpecError::Io(ref err) => err.fmt(f),
            SpecError::Json(ref err) => err.fmt(f),
        }
    }
}

impl std::error::Error for SpecError {
    fn description(&self) -> &str {
        match *self {
            SpecError::Io(ref err) => err.description(),
            SpecError::Json(ref err) => err.description(),
        }
    }
}

impl From<std::io::Error> for SpecError {
    fn from(err: std::io::Error) -> SpecError {
        SpecError::Io(err)
    }
}

impl From<serde_json::error::Error> for SpecError {
    fn from(err: serde_json::error::Error) -> SpecError {
        SpecError::Json(err)
    }
}

type Result<T> = std::result::Result<T, SpecError>;

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Spec {
    pub argv: Vec<String>,
    pub env: environ::spec::Env,
    pub fds: Vec<(String, fd::spec::Fd)>,
}

pub fn load_spec_file<P: AsRef<Path>>(path: P) -> Result<Spec> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Spec`.
    let spec = serde_json::from_reader(reader)?;

    // Return the spec.
    Ok(spec)
}

