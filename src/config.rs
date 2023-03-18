use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub on_update: String,
}
