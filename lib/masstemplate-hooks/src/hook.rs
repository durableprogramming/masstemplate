use async_trait::async_trait;
use tokio::process::Command;

use crate::config::HookConfig;
use crate::config::HookTiming;
use crate::context::HookContext;
use crate::error::{HookError, Result};

/// Trait that defines what a hook can do
#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    fn timing(&self) -> HookTiming;
    async fn execute(&self, context: &HookContext) -> Result<()>;
}

/// A hook that executes a shell command
pub struct CommandHook {
    config: HookConfig,
}

impl CommandHook {
    pub fn new(config: HookConfig) -> Self {
        Self { config }
    }
}

/// A hook that does nothing (useful for testing)
pub struct NoOpHook {
    name: String,
    timing: HookTiming,
}

impl NoOpHook {
    pub fn new(name: String, timing: HookTiming) -> Self {
        Self { name, timing }
    }
}

#[async_trait]
impl Hook for NoOpHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn timing(&self) -> HookTiming {
        self.timing
    }

    async fn execute(&self, _context: &HookContext) -> Result<()> {
        println!("Executing no-op hook '{}'", self.name);
        Ok(())
    }
}

/// A hook that prints a message
pub struct PrintHook {
    name: String,
    timing: HookTiming,
    message: String,
}

impl PrintHook {
    pub fn new(name: String, timing: HookTiming, message: String) -> Self {
        Self { name, timing, message }
    }
}

#[async_trait]
impl Hook for PrintHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn timing(&self) -> HookTiming {
        self.timing
    }

    async fn execute(&self, _context: &HookContext) -> Result<()> {
        println!("Hook '{}': {}", self.name, self.message);
        Ok(())
    }
}

#[async_trait]
impl Hook for CommandHook {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn timing(&self) -> HookTiming {
        self.config.timing
    }

    async fn execute(&self, context: &HookContext) -> Result<()> {
        let mut cmd = Command::new(&self.config.command);

        // Add arguments if provided
        if let Some(args) = &self.config.args {
            cmd.args(args);
        }

        // Set working directory
        let working_dir = context.resolve_working_directory(self.config.working_directory.as_deref());
        cmd.current_dir(working_dir);

        // Set environment variables
        // First, add context-specific environment variables
        cmd.envs(context.get_environment_variables());

        // Then add hook-specific environment variables (these can override context ones)
        if let Some(env) = &self.config.environment {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }

        println!("Executing hook '{}' with command: {}", self.config.name, self.config.command);

        let output = cmd.output().await
            .map_err(|e| HookError::CommandSpawnFailed {
                command: self.config.command.clone(),
                reason: e.to_string(),
            })?;

        if output.status.success() {
            println!("Hook '{}' executed successfully", self.config.name);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(1);
            Err(HookError::ExecutionFailed {
                command: self.config.command.clone(),
                exit_code,
                stderr,
            })
        }
    }
}