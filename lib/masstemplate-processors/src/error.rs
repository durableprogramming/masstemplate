#[derive(Debug, thiserror::Error)]
pub enum ProcessorError {
    #[error("Invalid environment variable format")]
    InvalidEnvFormat,
    #[error("Processing error: {0}")]
    ProcessingError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}