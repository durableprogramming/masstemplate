pub mod error;
pub mod context;
pub mod config;
pub mod hook;
pub mod manager;

// Re-exports for public API
pub use error::*;
pub use context::*;
pub use config::*;
pub use hook::*;
pub use manager::*;

// Imports for tests
#[cfg(test)]
use std::fs;
#[cfg(test)]
use tempfile::TempDir;
#[cfg(test)]
use toml;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_template_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        temp_dir
    }



    #[tokio::test]
    async fn test_hook_manager_load_no_hooks_file() {
        let temp_dir = create_test_template_dir();
        let manager = HookManager::load_from_template(temp_dir.path()).await.unwrap();
        assert!(!manager.has_hooks());
    }

    #[tokio::test]
    async fn test_hook_manager_load_with_hooks() {
        let temp_dir = create_test_template_dir();
        let hooks_toml = r#"
[[hooks]]
name = "test_hook"
command = "echo"
args = ["hello", "world"]
timing = "pre_copy"

[[hooks]]
name = "post_hook"
command = "echo"
timing = "post_copy"
"#;
        fs::write(temp_dir.path().join("hooks.toml"), hooks_toml).unwrap();

        let manager = HookManager::load_from_template(temp_dir.path()).await.unwrap();
        assert!(manager.has_hooks());
        // We can't easily check the exact count without exposing internals
    }

    #[tokio::test]
    async fn test_hook_manager_load_invalid_toml() {
        let temp_dir = create_test_template_dir();
        fs::write(temp_dir.path().join("hooks.toml"), "invalid toml content").unwrap();

        let result = HookManager::load_from_template(temp_dir.path()).await;
        assert!(matches!(result, Err(HookError::Toml(_))));
    }

    #[tokio::test]
    async fn test_hook_manager_load_invalid_hook_config_empty_name() {
        let temp_dir = create_test_template_dir();
        let hooks_toml = r#"
[[hooks]]
name = ""
command = "echo"
timing = "pre_copy"
"#;
        fs::write(temp_dir.path().join("hooks.toml"), hooks_toml).unwrap();

        let result = HookManager::load_from_template(temp_dir.path()).await;
        assert!(matches!(result, Err(HookError::InvalidHookName(_))));
    }

    #[tokio::test]
    async fn test_hook_manager_load_invalid_hook_config_empty_command() {
        let temp_dir = create_test_template_dir();
        let hooks_toml = r#"
[[hooks]]
name = "test"
command = ""
timing = "pre_copy"
"#;
        fs::write(temp_dir.path().join("hooks.toml"), hooks_toml).unwrap();

        let result = HookManager::load_from_template(temp_dir.path()).await;
        assert!(matches!(result, Err(HookError::InvalidHookCommand(_))));
    }

    #[tokio::test]
    async fn test_hook_manager_hook_info() {
        let temp_dir = create_test_template_dir();
        let hooks_toml = r#"
[[hooks]]
name = "hook1"
command = "echo"
timing = "pre_copy"

[[hooks]]
name = "hook2"
command = "ls"
timing = "post_copy"
"#;
        fs::write(temp_dir.path().join("hooks.toml"), hooks_toml).unwrap();

        let manager = HookManager::load_from_template(temp_dir.path()).await.unwrap();
        let info = manager.hook_info();
        assert_eq!(info.len(), 2);
        assert_eq!(info[0], ("hook1", HookTiming::PreCopy));
        assert_eq!(info[1], ("hook2", HookTiming::PostCopy));
    }

    #[tokio::test]
    async fn test_command_hook_execution_success() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("dest")).unwrap();

        let config = HookConfig {
            name: "test_hook".to_string(),
            command: "echo".to_string(),
            args: Some(vec!["hello".to_string()]),
            timing: HookTiming::PreCopy,
            working_directory: Some(temp_dir.path().join("dest").to_string_lossy().to_string()),
            environment: None,
        };

        let hook = CommandHook::new(config);
        let context = HookContext::new(
            "test_template".to_string(),
            temp_dir.path().join("template"),
            temp_dir.path().join("dest"),
            masstemplate_fileops::CollisionStrategy::Skip,
        );

        let result = hook.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_command_hook_execution_failure() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("dest")).unwrap();

        let config = HookConfig {
            name: "test_hook".to_string(),
            command: "false".to_string(), // Command that always fails
            args: None,
            timing: HookTiming::PreCopy,
            working_directory: Some(temp_dir.path().join("dest").to_string_lossy().to_string()),
            environment: None,
        };

        let hook = CommandHook::new(config);
        let context = HookContext::new(
            "test_template".to_string(),
            temp_dir.path().join("template"),
            temp_dir.path().join("dest"),
            masstemplate_fileops::CollisionStrategy::Skip,
        );

        let result = hook.execute(&context).await;
        assert!(matches!(result, Err(HookError::ExecutionFailed { .. })));
    }

    #[tokio::test]
    async fn test_command_hook_timing() {
        let config = HookConfig {
            name: "test_hook".to_string(),
            command: "echo".to_string(),
            args: None,
            timing: HookTiming::PostCopy,
            working_directory: None,
            environment: None,
        };

        let hook = CommandHook::new(config);
        assert_eq!(hook.timing(), HookTiming::PostCopy);
    }

    #[tokio::test]
    async fn test_command_hook_command_not_found() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("dest")).unwrap();

        let config = HookConfig {
            name: "test_hook".to_string(),
            command: "nonexistent_command_12345".to_string(), // Command that doesn't exist
            args: None,
            timing: HookTiming::PreCopy,
            working_directory: Some(temp_dir.path().join("dest").to_string_lossy().to_string()),
            environment: None,
        };

        let hook = CommandHook::new(config);
        let context = HookContext::new(
            "test_template".to_string(),
            temp_dir.path().join("template"),
            temp_dir.path().join("dest"),
            masstemplate_fileops::CollisionStrategy::Skip,
        );

        let result = hook.execute(&context).await;
        assert!(matches!(result, Err(HookError::CommandSpawnFailed { .. })));
    }

    #[tokio::test]
    async fn test_command_hook_invalid_working_directory() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("dest")).unwrap();

        let config = HookConfig {
            name: "test_hook".to_string(),
            command: "echo".to_string(),
            args: Some(vec!["test".to_string()]),
            timing: HookTiming::PreCopy,
            working_directory: Some("/nonexistent/directory/that/does/not/exist".to_string()),
            environment: None,
        };

        let hook = CommandHook::new(config);
        let context = HookContext::new(
            "test_template".to_string(),
            temp_dir.path().join("template"),
            temp_dir.path().join("dest"),
            masstemplate_fileops::CollisionStrategy::Skip,
        );

        let result = hook.execute(&context).await;
        // This should fail because the working directory doesn't exist
        // The error will be CommandSpawnFailed since tokio::process::Command::current_dir fails
        assert!(matches!(result, Err(HookError::CommandSpawnFailed { .. })));
    }

    #[tokio::test]
    async fn test_noop_hook_execution() {
        let hook = NoOpHook::new("noop_hook".to_string(), HookTiming::PreCopy);
        let context = HookContext::new(
            "test_template".to_string(),
            std::path::PathBuf::from("/tmp/template"),
            std::path::PathBuf::from("/tmp/dest"),
            masstemplate_fileops::CollisionStrategy::Skip,
        );

        let result = hook.execute(&context).await;
        assert!(result.is_ok());
        assert_eq!(hook.name(), "noop_hook");
        assert_eq!(hook.timing(), HookTiming::PreCopy);
    }

    #[tokio::test]
    async fn test_print_hook_execution() {
        let hook = PrintHook::new(
            "print_hook".to_string(),
            HookTiming::PostCopy,
            "Hello from hook!".to_string(),
        );
        let context = HookContext::new(
            "test_template".to_string(),
            std::path::PathBuf::from("/tmp/template"),
            std::path::PathBuf::from("/tmp/dest"),
            masstemplate_fileops::CollisionStrategy::Skip,
        );

        let result = hook.execute(&context).await;
        assert!(result.is_ok());
        assert_eq!(hook.name(), "print_hook");
        assert_eq!(hook.timing(), HookTiming::PostCopy);
    }

    #[tokio::test]
    async fn test_hook_manager_execute_hooks_filtering() {
        let temp_dir = create_test_template_dir();
        let dest_dir = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_dir).unwrap();

        let hooks_toml = r#"
[[hooks]]
name = "pre_hook"
command = "echo"
args = ["pre"]
timing = "pre_copy"

[[hooks]]
name = "post_hook"
command = "echo"
args = ["post"]
timing = "post_copy"
"#;
        fs::write(temp_dir.path().join("hooks.toml"), hooks_toml).unwrap();

        let manager = HookManager::load_from_template(temp_dir.path()).await.unwrap();
        let context = HookContext::new(
            "test_template".to_string(),
            temp_dir.path().to_path_buf(),
            dest_dir,
            masstemplate_fileops::CollisionStrategy::Skip,
        );

        // Should execute without error (both hooks should run based on their timing)
        let result = manager.execute_pre_copy_hooks(&context).await;
        assert!(result.is_ok());

        let result = manager.execute_post_copy_hooks(&context).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_hook_config_deserialization() {
        let toml_content = r#"
name = "test_hook"
command = "echo"
args = ["hello", "world"]
timing = "pre_copy"
working_directory = "/tmp"
[environment]
KEY1 = "value1"
KEY2 = "value2"
"#;

        let config: HookConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.name, "test_hook");
        assert_eq!(config.command, "echo");
        assert_eq!(config.args.as_ref().unwrap(), &vec!["hello".to_string(), "world".to_string()]);
        assert_eq!(config.timing, HookTiming::PreCopy);
        assert_eq!(config.working_directory, Some("/tmp".to_string()));
        assert_eq!(config.environment.as_ref().unwrap().get("KEY1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_hooks_config_deserialization() {
        let toml_content = r#"
[[hooks]]
name = "hook1"
command = "echo"
timing = "pre_copy"

[[hooks]]
name = "hook2"
command = "ls"
timing = "post_copy"
"#;

        let config: HooksConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.hooks.len(), 2);
        assert_eq!(config.hooks[0].name, "hook1");
        assert_eq!(config.hooks[1].name, "hook2");
    }

    #[test]
    fn test_hook_config_validation_valid() {
        let config = HookConfig {
            name: "test_hook".to_string(),
            command: "echo".to_string(),
            args: Some(vec!["hello".to_string()]),
            timing: HookTiming::PreCopy,
            working_directory: Some("/tmp".to_string()),
            environment: Some([("KEY".to_string(), "value".to_string())].into()),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_hook_config_validation_empty_name() {
        let config = HookConfig {
            name: "".to_string(),
            command: "echo".to_string(),
            args: None,
            timing: HookTiming::PreCopy,
            working_directory: None,
            environment: None,
        };

        assert!(matches!(config.validate(), Err(HookError::InvalidHookName(_))));
    }

    #[test]
    fn test_hook_config_validation_empty_command() {
        let config = HookConfig {
            name: "test".to_string(),
            command: "".to_string(),
            args: None,
            timing: HookTiming::PreCopy,
            working_directory: None,
            environment: None,
        };

        assert!(matches!(config.validate(), Err(HookError::InvalidHookCommand(_))));
    }

    #[test]
    fn test_hook_config_validation_invalid_working_directory() {
        let config = HookConfig {
            name: "test".to_string(),
            command: "echo".to_string(),
            args: None,
            timing: HookTiming::PreCopy,
            working_directory: Some("".to_string()),
            environment: None,
        };

        assert!(matches!(config.validate(), Err(HookError::InvalidWorkingDirectory(_))));
    }

    #[test]
    fn test_hook_config_validation_invalid_environment_key() {
        let config = HookConfig {
            name: "test".to_string(),
            command: "echo".to_string(),
            args: None,
            timing: HookTiming::PreCopy,
            working_directory: None,
            environment: Some([("".to_string(), "value".to_string())].into()),
        };

        assert!(matches!(config.validate(), Err(HookError::InvalidEnvironmentVariable(_))));
    }

    #[test]
    fn test_hook_config_validation_invalid_args() {
        let config = HookConfig {
            name: "test".to_string(),
            command: "echo".to_string(),
            args: Some(vec!["hello\x00world".to_string()]),
            timing: HookTiming::PreCopy,
            working_directory: None,
            environment: None,
        };

        assert!(matches!(config.validate(), Err(HookError::InvalidHookArgs(_))));
    }
}