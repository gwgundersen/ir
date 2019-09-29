use libc::c_int;
use serde::{Serialize, Deserialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::string::String;

use crate::environ;
use crate::fd;

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct FdSpec {
    pub fd: c_int,
    #[serde(flatten)]
    pub spec: fd::spec::Fd,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Spec {
    pub argv: Vec<String>,
    pub env: environ::spec::Env,
    pub fds: Vec<FdSpec>,
}

pub fn load_spec_file<P: AsRef<Path>>(path: P) -> Result<Spec, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Spec`.
    let spec = serde_json::from_reader(reader)?;

    // Return the spec.
    Ok(spec)
}

