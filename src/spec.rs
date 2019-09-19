use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::string::String;
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum EnvInheritSpec {
    None,
    All,
    Some {
        vars: Vec<String>,
    },
}

impl Default for EnvInheritSpec {
    fn default() -> Self { EnvInheritSpec::None }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct EnvSpec {
    pub inherit: EnvInheritSpec,
    pub vars: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Spec {
    pub argv: Vec<String>,
    pub env: EnvSpec,
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

