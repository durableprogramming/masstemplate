use std::path::PathBuf;
use crate::{ConfigError, GlobalConfig};

/// Get the path to the global configuration file
pub fn get_global_config_path() -> Result<PathBuf, ConfigError> {
    let config_dir = dirs::config_dir()
        .ok_or(ConfigError::NoConfigDir)?
        .join("masstemplate");

    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("config.toml"))
}

/// Get the path to the templates directory
pub fn get_templates_directory(config: &GlobalConfig) -> Result<PathBuf, ConfigError> {
    if let Some(custom_dir) = &config.template_directory {
        Ok(PathBuf::from(custom_dir))
    } else {
        let home_dir = dirs::home_dir().ok_or(ConfigError::NoHomeDir)?;
        Ok(home_dir.join(".local/masstemplate"))
    }
}

/// Get the path to a specific template
pub fn get_template_path(config: &GlobalConfig, template_name: &str) -> Result<PathBuf, ConfigError> {
    if template_name.is_empty() || template_name.contains('/') || template_name.contains('\\') {
        return Err(ConfigError::InvalidTemplateName(template_name.to_string()));
    }

    // Check primary template directory first
    let templates_dir = get_templates_directory(config)?;
    let primary_path = templates_dir.join(template_name);
    if primary_path.exists() && primary_path.is_dir() {
        return Ok(primary_path);
    }

    // Check additional template source directories
    if let Some(sources) = &config.template_sources {
        for source_dir in sources {
            let source_path = PathBuf::from(source_dir).join(template_name);
            if source_path.exists() && source_path.is_dir() {
                return Ok(source_path);
            }
        }
    }

    // If not found anywhere, return the primary path (for backwards compatibility with error messages)
    Ok(primary_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_template_path_invalid_name() {
        let config = GlobalConfig::default();
        // Test with invalid template names
        assert!(matches!(get_template_path(&config, ""), Err(ConfigError::InvalidTemplateName(_))));
        assert!(matches!(get_template_path(&config, "template/with/slash"), Err(ConfigError::InvalidTemplateName(_))));
        assert!(matches!(get_template_path(&config, "template\\with\\backslash"), Err(ConfigError::InvalidTemplateName(_))));
    }

    #[test]
    fn test_get_template_path_valid_name() {
        let config = GlobalConfig::default();
        // Test that it constructs the path correctly (without checking if it exists)
        let path = get_template_path(&config, "my_template").unwrap();
        let expected_suffix = ".local/masstemplate/my_template";
        assert!(path.to_string_lossy().ends_with(expected_suffix));
    }

    #[test]
    fn test_get_template_path_custom_directory() {
        let mut config = GlobalConfig::default();
        config.template_directory = Some("/custom/templates".to_string());
        let path = get_template_path(&config, "my_template").unwrap();
        assert_eq!(path, PathBuf::from("/custom/templates/my_template"));
    }

    #[test]
    fn test_get_templates_directory_custom() {
        let mut config = GlobalConfig::default();
        config.template_directory = Some("/custom/templates".to_string());
        let path = get_templates_directory(&config).unwrap();
        assert_eq!(path, PathBuf::from("/custom/templates"));
    }

    #[test]
    fn test_get_templates_directory_default() {
        let config = GlobalConfig::default();
        let path = get_templates_directory(&config).unwrap();
        let expected_suffix = ".local/masstemplate";
        assert!(path.to_string_lossy().ends_with(expected_suffix));
    }
}