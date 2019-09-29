use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OpenFlagSpec {
    // FIXME: Generalize.

    /// Read for stdin, Write for stdout/stderr, ReadWrite for others.
    Default,  

    Read,
    Write,
    Append,
    ReadWrite,
}

impl Default for OpenFlagSpec {
    fn default() -> Self { Self::Default }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct FileSpec {
    path: PathBuf,
    flags: OpenFlagSpec,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all="lowercase")]
pub enum FdSpec {
    Inherit,
    Close,
    Null,
    File(FileSpec),
}

impl Default for FdSpec {
    fn default() -> Self { Self::Inherit }
}

