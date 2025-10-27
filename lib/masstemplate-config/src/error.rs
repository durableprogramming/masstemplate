use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("Could not find home directory")]
    NoHomeDir,
    #[error("Could not find config directory")]
    NoConfigDir,
    #[error("Template directory not found: {0}")]
    TemplateDirNotFound(PathBuf),
    #[error("Invalid template name: {0}")]
    InvalidTemplateName(String),
}