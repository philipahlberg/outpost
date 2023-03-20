use std::{
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub stdout: Option<PathBuf>,
    pub stderr: Option<PathBuf>,
    pub on_update: String,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Toml(toml::de::Error),
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl Config {
    pub fn from_path(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let base = path.parent().unwrap_or(Path::new("."));
        Self::from_toml(&content)?.with_absolute_paths(base)
    }

    fn from_toml(content: &str) -> Result<Self, ConfigError> {
        toml::from_str(content).map_err(ConfigError::Toml)
    }

    fn with_absolute_paths(self, base: &Path) -> Result<Self, ConfigError> {
        let stdout = self
            .stdout
            .map(|path| normalize_path(base, path))
            .transpose()?;
        let stderr = self
            .stderr
            .map(|path| normalize_path(base, path))
            .transpose()?;

        Ok(Self {
            stdout,
            stderr,
            on_update: self.on_update,
        })
    }
}

fn normalize_path(base: &Path, path: PathBuf) -> Result<PathBuf, io::Error> {
    let path = relative_to(base, path);

    if let (Some(parent), Some(file_name)) = (path.parent(), path.file_name()) {
        let parent = parent.canonicalize()?;
        Ok(parent.join(file_name))
    } else {
        Ok(path)
    }
}

fn relative_to(base: &Path, path: PathBuf) -> PathBuf {
    if path.is_relative() {
        let mut base = base.to_path_buf();
        base.extend(path.iter());
        base
    } else {
        path
    }
}
