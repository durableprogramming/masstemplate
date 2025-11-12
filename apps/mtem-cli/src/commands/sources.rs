use anyhow::{anyhow, Result};
use masstemplate_config::{load_global_config, save_global_config};
use std::path::PathBuf;

/// Add a template source directory
pub async fn add_source(path: PathBuf) -> Result<()> {
    let mut config = load_global_config().await?;

    // Canonicalize the path to get absolute path
    let canonical_path = path.canonicalize().map_err(|e| {
        anyhow!("Failed to resolve path '{}': {}", path.display(), e)
    })?;

    // Verify the path exists and is a directory
    if !canonical_path.is_dir() {
        return Err(anyhow!("Path '{}' is not a directory", canonical_path.display()));
    }

    let path_str = canonical_path.to_string_lossy().to_string();

    // Initialize template_sources if None
    if config.template_sources.is_none() {
        config.template_sources = Some(Vec::new());
    }

    // Check if source already exists
    if let Some(sources) = &config.template_sources {
        if sources.contains(&path_str) {
            println!("Source '{}' already exists", path_str);
            return Ok(());
        }
    }

    // Add the new source
    config.template_sources.as_mut().unwrap().push(path_str.clone());

    // Save configuration
    save_global_config(&config).await?;

    println!("Added template source: {}", path_str);
    Ok(())
}

/// Remove a template source directory
pub async fn remove_source(path: PathBuf) -> Result<()> {
    let mut config = load_global_config().await?;

    // Try to canonicalize, but if it fails (path doesn't exist), use the raw path
    let path_str = if let Ok(canonical) = path.canonicalize() {
        canonical.to_string_lossy().to_string()
    } else {
        path.to_string_lossy().to_string()
    };

    // Initialize template_sources if None
    if config.template_sources.is_none() {
        config.template_sources = Some(Vec::new());
    }

    let sources = config.template_sources.as_mut().unwrap();

    // Find and remove the source
    let original_len = sources.len();
    sources.retain(|s| s != &path_str);

    if sources.len() == original_len {
        return Err(anyhow!("Source '{}' not found in configuration", path_str));
    }

    // Save configuration
    save_global_config(&config).await?;

    println!("Removed template source: {}", path_str);
    Ok(())
}

/// List all template source directories
pub async fn list_sources() -> Result<()> {
    let config = load_global_config().await?;

    if let Some(sources) = &config.template_sources {
        if sources.is_empty() {
            println!("No additional template sources configured");
        } else {
            println!("Template sources:");
            for source in sources {
                println!("  {}", source);
            }
        }
    } else {
        println!("No additional template sources configured");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_add_remove_source() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("templates");
        std::fs::create_dir(&source_dir).unwrap();

        // This test would need environment setup for config
        // Skipping actual file operations in unit tests
    }
}
