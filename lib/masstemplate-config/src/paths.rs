use std::path::PathBuf;
use crate::{ConfigError, GlobalConfig};

/// Parse a GitHub URL in the format github:org/repo/path
/// Returns (org, repo, path) tuple
pub fn parse_github_url(template_name: &str) -> Option<(String, String, String)> {
    let template_name = template_name.strip_prefix("github:")?;
    let parts: Vec<&str> = template_name.splitn(3, '/').collect();

    if parts.len() < 2 {
        return None;
    }

    let org = parts[0].to_string();
    let repo = parts[1].to_string();
    let path = if parts.len() == 3 {
        parts[2].to_string()
    } else {
        String::new()
    };

    Some((org, repo, path))
}

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

/// Clone a GitHub repository to a temporary directory and return the path to the subdirectory
async fn clone_github_template(org: &str, repo: &str, path: &str) -> Result<PathBuf, ConfigError> {
    use std::process::Stdio;
    use tokio::process::Command;

    // Create a cache directory for GitHub templates
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find cache directory",
            )
        })?
        .join("masstemplate")
        .join("github");

    tokio::fs::create_dir_all(&cache_dir).await?;

    let repo_cache_dir = cache_dir.join(format!("{}-{}", org, repo));

    // Check if repo is already cached
    if repo_cache_dir.exists() {
        // Update the cached repository
        let output = Command::new("git")
            .args(["-C", repo_cache_dir.to_str().unwrap(), "pull"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .await?;

        if !output.status.success() {
            // If pull fails, remove and re-clone
            tokio::fs::remove_dir_all(&repo_cache_dir).await?;
        }
    }

    // Clone if not cached or if we just removed it
    if !repo_cache_dir.exists() {
        let repo_url = format!("https://github.com/{}/{}.git", org, repo);
        let output = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                &repo_url,
                repo_cache_dir.to_str().unwrap(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to clone repository: {}", error_msg),
            ).into());
        }
    }

    // Return the full path to the subdirectory
    let template_path = if path.is_empty() {
        repo_cache_dir
    } else {
        repo_cache_dir.join(path)
    };

    if !template_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path '{}' does not exist in repository {}/{}", path, org, repo),
        ).into());
    }

    Ok(template_path)
}

/// Get the path to a specific template
pub async fn get_template_path_async(config: &GlobalConfig, template_name: &str) -> Result<PathBuf, ConfigError> {
    // Check if it's a GitHub URL
    if let Some((org, repo, path)) = parse_github_url(template_name) {
        return clone_github_template(&org, &repo, &path).await;
    }

    // Synchronous path for local templates
    get_template_path_sync(config, template_name)
}

/// Synchronous version of get_template_path for backward compatibility
pub fn get_template_path(config: &GlobalConfig, template_name: &str) -> Result<PathBuf, ConfigError> {
    get_template_path_sync(config, template_name)
}

fn get_template_path_sync(config: &GlobalConfig, template_name: &str) -> Result<PathBuf, ConfigError> {
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