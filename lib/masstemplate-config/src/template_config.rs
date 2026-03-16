use std::path::Path;
use std::collections::HashSet;
use crate::{ConfigError, GlobalConfig, TemplateConfig, TemplateInfo};
use crate::paths::{get_templates_directory, get_template_path, get_template_path_async};

/// Discover all available templates
pub async fn discover_templates(config: &GlobalConfig) -> Result<Vec<TemplateInfo>, ConfigError> {
    let mut templates = Vec::new();
    let mut seen_names = HashSet::new();

    // Check the primary template directory
    let templates_dir = get_templates_directory(config)?;
    if templates_dir.exists() {
        let mut entries = tokio::fs::read_dir(&templates_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir()
                && let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if seen_names.insert(name.to_string()) {
                    let config = load_template_config(&path).await.ok();
                    templates.push(TemplateInfo {
                        name: name.to_string(),
                        path,
                        config,
                    });
                }
            }
        }
    }

    // Check additional template source directories
    if let Some(sources) = &config.template_sources {
        for source_dir in sources {
            let source_path = std::path::PathBuf::from(source_dir);
            if source_path.exists() {
                let mut entries = tokio::fs::read_dir(&source_path).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.is_dir()
                        && let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if seen_names.insert(name.to_string()) {
                            let config = load_template_config(&path).await.ok();
                            templates.push(TemplateInfo {
                                name: name.to_string(),
                                path,
                                config,
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by name for consistent ordering
    templates.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(templates)
}

/// Load template-specific configuration
pub async fn load_template_config(template_path: &Path) -> Result<TemplateConfig, ConfigError> {
    let config_path = template_path.join("template.toml");

    if !config_path.exists() {
        return Ok(TemplateConfig {
            name: None,
            description: None,
            collision_strategy: None,
            tags: None,
            version: None,
        });
    }

    let content = tokio::fs::read_to_string(&config_path).await?;
    let config: TemplateConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Save template-specific configuration
pub async fn save_template_config(template_path: &Path, config: &TemplateConfig) -> Result<(), ConfigError> {
    let config_path = template_path.join("template.toml");
    let content = toml::to_string_pretty(config)?;
    tokio::fs::write(&config_path, content).await?;
    Ok(())
}

/// Check if a template exists
pub async fn template_exists(config: &GlobalConfig, template_name: &str) -> Result<bool, ConfigError> {
    let template_path = get_template_path(config, template_name)?;
    Ok(template_path.is_dir())
}

/// Get information about a specific template
pub async fn get_template_info(config: &GlobalConfig, template_name: &str) -> Result<TemplateInfo, ConfigError> {
    let path = get_template_path_async(config, template_name).await?;

    if !path.is_dir() {
        return Err(ConfigError::TemplateDirNotFound(path));
    }

    let config = load_template_config(&path).await.ok();

    Ok(TemplateInfo {
        name: template_name.to_string(),
        path,
        config,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_template_config_load_save() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("test_template");
        fs::create_dir_all(&template_dir).unwrap();

        let config = TemplateConfig {
            name: Some("Test Template".to_string()),
            description: Some("A test template".to_string()),
            collision_strategy: Some("overwrite".to_string()),
            tags: Some(vec!["test".to_string(), "example".to_string()]),
            version: Some("1.0.0".to_string()),
        };

        // Save config
        save_template_config(&template_dir, &config).await.unwrap();

        // Load config
        let loaded_config = load_template_config(&template_dir).await.unwrap();

        assert_eq!(loaded_config.name, config.name);
        assert_eq!(loaded_config.description, config.description);
        assert_eq!(loaded_config.collision_strategy, config.collision_strategy);
        assert_eq!(loaded_config.tags, config.tags);
        assert_eq!(loaded_config.version, config.version);
    }

    #[tokio::test]
    async fn test_template_config_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("test_template");
        fs::create_dir_all(&template_dir).unwrap();

        let config = load_template_config(&template_dir).await.unwrap();
        assert!(config.name.is_none());
        assert!(config.description.is_none());
        assert!(config.collision_strategy.is_none());
        assert!(config.tags.is_none());
        assert!(config.version.is_none());
    }

    #[tokio::test]
    async fn test_template_exists_nonexistent() {
        let config = GlobalConfig::default();
        // Test with a template that definitely doesn't exist
        let exists = template_exists(&config, "definitely_nonexistent_template_12345").await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_get_template_info_not_found() {
        let config = GlobalConfig::default();
        let result = get_template_info(&config, "nonexistent_template_12345").await;
        assert!(matches!(result, Err(ConfigError::TemplateDirNotFound(_))));
    }
}