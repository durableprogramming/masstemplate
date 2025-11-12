use crate::{ConfigError, GlobalConfig};
use crate::paths::get_global_config_path;

/// Load global configuration
pub async fn load_global_config() -> Result<GlobalConfig, ConfigError> {
    let config_path = get_global_config_path()?;

    if !config_path.exists() {
        return Ok(GlobalConfig::default());
    }

    let content = tokio::fs::read_to_string(&config_path).await?;
    let config: GlobalConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Save global configuration
pub async fn save_global_config(config: &GlobalConfig) -> Result<(), ConfigError> {
    let config_path = get_global_config_path()?;
    let content = toml::to_string_pretty(config)?;
    tokio::fs::write(&config_path, content).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_global_config_load_save() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.toml");

        let config = GlobalConfig {
            default_collision_strategy: Some("overwrite".to_string()),
            verbose: Some(true),
            template_directory: Some("/custom/path".to_string()),
            template_sources: Some(Vec::new()),
        };

        // Save config to temp file
        let content = toml::to_string_pretty(&config).unwrap();
        tokio::fs::write(&config_file, content).await.unwrap();

        // Load config from temp file
        let content = tokio::fs::read_to_string(&config_file).await.unwrap();
        let loaded_config: GlobalConfig = toml::from_str(&content).unwrap();

        assert_eq!(loaded_config.default_collision_strategy, config.default_collision_strategy);
        assert_eq!(loaded_config.verbose, config.verbose);
        assert_eq!(loaded_config.template_directory, config.template_directory);
    }

    #[test]
    fn test_global_config_default() {
        let config = GlobalConfig::default();
        assert_eq!(config.default_collision_strategy, Some("skip".to_string()));
        assert_eq!(config.verbose, Some(false));
        assert!(config.template_directory.is_none());
    }
}