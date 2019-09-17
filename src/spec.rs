use serde::{Serialize, Deserialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::string::String;
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug)]
pub struct Spec {
    argv: Vec<String>,
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

