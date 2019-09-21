use serde::{Serialize, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::string::String;
use std::vec::Vec;

//------------------------------------------------------------------------------

#[derive(Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum EnvInheritSpec {
    None,
    All,
    Vars(Vec<String>),
}

impl Default for EnvInheritSpec {
    fn default() -> Self { Self::None }
}

impl<'de> Deserialize<'de> for EnvInheritSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = EnvInheritSpec;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("true, false, or seq of env var names")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(if v { Self::Value::All } else { Self::Value::None })
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error> 
            where
                S: serde::de::SeqAccess<'de>
            {
                let mut vars = Vec::new();
                while let Some(var) = seq.next_element()? {
                    vars.push(var);
                }                    
                Ok(Self::Value::Vars(vars))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

//------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct EnvSpec {
    pub inherit: EnvInheritSpec,
    pub vars: HashMap<String, String>,
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::EnvInheritSpec::*;

    fn assert_json(json: &'static str, expected: EnvSpec) {
        assert_eq!(
            serde_json::from_str::<'static, EnvSpec>(json).unwrap(),
            expected);
    }

    #[test]
    fn empty() {
        assert_json(
            r#" {} "#, 
            EnvSpec { inherit: None, ..Default::default() }
        );
    }

    #[test]
    fn inherit_all() {
        assert_json(
            r#" {"inherit": true} "#,
            EnvSpec { inherit: All, ..Default::default() }
        );
    }

    #[test]
    fn inherit_vars() {
        assert_json(
            r#" {"inherit": ["HOME", "USER", "PATH"]} "#,
            EnvSpec {
                inherit: Vars(vec!(
                    "HOME".to_string(),
                    "USER".to_string(),
                    "PATH".to_string())
                ),
                ..Default::default()
            }
        );
    }
}

