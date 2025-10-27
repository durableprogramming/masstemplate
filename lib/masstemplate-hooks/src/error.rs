use thiserror::Error;

#[derive(Error, Debug)]
pub enum HookError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("Hook execution failed: {command} (exit code: {exit_code}): {stderr}")]
    ExecutionFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },
    #[error("Failed to spawn command '{command}': {reason}")]
    CommandSpawnFailed {
        command: String,
        reason: String,
    },
    #[error("Invalid hook name: {0}")]
    InvalidHookName(String),
    #[error("Invalid hook command: {0}")]
    InvalidHookCommand(String),
    #[error("Invalid working directory: {0}")]
    InvalidWorkingDirectory(String),
    #[error("Invalid environment variable: {0}")]
    InvalidEnvironmentVariable(String),
    #[error("Invalid hook arguments: {0}")]
    InvalidHookArgs(String),
}

pub type Result<T> = std::result::Result<T, HookError>;