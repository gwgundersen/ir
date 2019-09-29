pub mod spec {

    use serde::{Serialize, Deserialize};
    use std::path::PathBuf;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub enum OpenFlag {
        // FIXME: Generalize.

        /// Read for stdin, Write for stdout/stderr, ReadWrite for others.
        Default,  

        Read,
        Write,
        Append,
        ReadWrite,
    }

    impl Default for OpenFlag {
        fn default() -> Self { Self::Default }
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(default)]
    pub struct File {
        path: PathBuf,
        flags: OpenFlag,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all="lowercase")]
    pub enum Fd {
        Inherit,
        Close,
        Null,
        File(File),
    }

    impl Default for Fd {
        fn default() -> Self { Self::Inherit }
    }

}

