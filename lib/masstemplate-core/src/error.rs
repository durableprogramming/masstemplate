use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Invalid template path: {0}")]
    InvalidTemplatePath(PathBuf),
    #[error("Script execution failed: {0}")]
    ScriptExecutionFailed(String),
    #[error("Configuration error: {0}")]
    Config(#[from] masstemplate_config::ConfigError),
    #[error("File operation error: {0}")]
    FileOp(#[from] masstemplate_fileops::FileOpsError),
    #[error("Hook error: {0}")]
    Hook(#[from] masstemplate_hooks::HookError),

    #[error("VCS error: {0}")]
    Vcs(#[from] masstemplate_vcs::VcsError),
    #[error("Generic error: {0}")]
    Generic(String),
}
