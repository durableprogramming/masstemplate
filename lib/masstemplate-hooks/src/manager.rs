use std::path::Path;

use crate::config::{HookTiming, HooksConfig};
use crate::context::HookContext;
use crate::error::Result;
use crate::hook::{Hook, CommandHook};

/// Manages hooks for a template
pub struct HookManager {
    hooks: Vec<Box<dyn Hook>>,
}

impl HookManager {
    /// Load hooks from a template directory
    pub async fn load_from_template(template_path: &Path) -> Result<Self> {
        let hooks_path = template_path.join("hooks.toml");

        if !hooks_path.exists() {
            // No hooks configuration, return empty manager
            return Ok(Self { hooks: Vec::new() });
        }

        let content = tokio::fs::read_to_string(&hooks_path).await?;
        let config: HooksConfig = toml::from_str(&content)?;

        let mut hooks = Vec::new();
        for hook_config in config.hooks {
            // Validate hook configuration
            hook_config.validate()?;
            let hook: Box<dyn Hook> = Box::new(CommandHook::new(hook_config));
            hooks.push(hook);
        }

        Ok(Self { hooks })
    }



    /// Execute all hooks for the given timing
    pub async fn execute_hooks(&self, timing: HookTiming, context: &HookContext) -> Result<()> {
        for hook in &self.hooks {
            if hook.timing() == timing {
                hook.execute(context).await?;
            }
        }
        Ok(())
    }

    /// Execute pre-copy hooks
    pub async fn execute_pre_copy_hooks(&self, context: &HookContext) -> Result<()> {
        self.execute_hooks(HookTiming::PreCopy, context).await
    }

    /// Execute post-copy hooks
    pub async fn execute_post_copy_hooks(&self, context: &HookContext) -> Result<()> {
        self.execute_hooks(HookTiming::PostCopy, context).await
    }

    /// Check if there are any hooks configured
    pub fn has_hooks(&self) -> bool {
        !self.hooks.is_empty()
    }

    /// Get the number of hooks configured
    pub fn hook_count(&self) -> usize {
        self.hooks.len()
    }

    /// Get information about configured hooks
    pub fn hook_info(&self) -> Vec<(&str, HookTiming)> {
        self.hooks.iter().map(|h| (h.name(), h.timing())).collect()
    }
}