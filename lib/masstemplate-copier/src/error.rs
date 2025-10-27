use thiserror::Error;

#[derive(Error, Debug)]
pub enum CopierError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Template rendering error: {0}")]
    Template(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Task execution failed: {0}")]
    TaskFailed(String),

    #[error("Variable prompt error: {0}")]
    PromptError(String),
}

pub type Result<T> = std::result::Result<T, CopierError>;
