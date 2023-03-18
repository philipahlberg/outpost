use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Key(Vec<u8>);

impl From<String> for Key {
    fn from(value: String) -> Self {
        Self(value.into_bytes())
    }
}

impl<'a> From<&'a str> for Key {
    fn from(value: &'a str) -> Self {
        Self(value.as_bytes().to_vec())
    }
}

impl From<Key> for String {
    fn from(value: Key) -> Self {
        String::from_utf8(value.0).expect("invalid UTF-8")
    }
}

pub const PROCESSES: &str = "processes";

#[derive(Debug, Serialize, Deserialize)]
pub enum Process {
    V1(v1::Process),
}

pub mod v1 {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Process {
        pub directory: String,
        pub stdout: String,
        pub stderr: String,
        pub process_id: Option<u32>,
    }
}

impl Process {
    #[allow(unused)]
    pub fn directory(&self) -> &str {
        match self {
            Process::V1(v) => &v.directory,
        }
    }

    #[allow(unused)]
    pub fn stdout(&self) -> &str {
        match self {
            Process::V1(v) => &v.stdout,
        }
    }

    #[allow(unused)]
    pub fn stderr(&self) -> &str {
        match self {
            Process::V1(v) => &v.stderr,
        }
    }

    #[allow(unused)]
    pub fn active(&self) -> bool {
        match self {
            Process::V1(v) => v.process_id.is_some(),
        }
    }

    #[allow(unused)]
    pub fn process_id(&self) -> Option<u32> {
        match self {
            Process::V1(v) => v.process_id,
        }
    }
}
