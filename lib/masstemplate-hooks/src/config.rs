/// Defines the timing when a hook should execute
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
pub enum HookTiming {
    #[serde(rename = "pre_copy")]
    #[default]
    PreCopy,
    #[serde(rename = "post_copy")]
    PostCopy,
}

/// Configuration for a single hook
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct HookConfig {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub timing: HookTiming,
    pub working_directory: Option<String>,
    pub environment: Option<std::collections::HashMap<String, String>>,
}

impl HookConfig {
    /// Validate the hook configuration
    pub fn validate(&self) -> Result<(), crate::error::HookError> {
        // Validate name
        if self.name.trim().is_empty() {
            return Err(crate::error::HookError::InvalidHookName(self.name.clone()));
        }

        // Validate command
        if self.command.trim().is_empty() {
            return Err(crate::error::HookError::InvalidHookCommand(self.command.clone()));
        }

        // Validate working directory if provided
        if let Some(ref wd) = self.working_directory {
            if wd.trim().is_empty() {
                return Err(crate::error::HookError::InvalidWorkingDirectory(wd.clone()));
            }
            // Check for invalid path characters (basic check)
            if wd.contains('\0') {
                return Err(crate::error::HookError::InvalidWorkingDirectory(wd.clone()));
            }
        }

        // Validate environment variables
        if let Some(ref env) = self.environment {
            for (key, value) in env {
                if key.trim().is_empty() {
                    return Err(crate::error::HookError::InvalidEnvironmentVariable(format!("empty key for value '{}'", value)));
                }
                if key.contains('=') || key.contains('\0') {
                    return Err(crate::error::HookError::InvalidEnvironmentVariable(format!("invalid key '{}'", key)));
                }
            }
        }

        // Validate args if provided
        if let Some(ref args) = self.args {
            for arg in args {
                if arg.contains('\0') {
                    return Err(crate::error::HookError::InvalidHookArgs(format!("argument contains null byte: '{}'", arg)));
                }
            }
        }

        Ok(())
    }
}

/// Configuration for all hooks in a template
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct HooksConfig {
    pub hooks: Vec<HookConfig>,
}