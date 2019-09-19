use serde::{Serialize, Deserialize};
use std::collections::HashMap;
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

