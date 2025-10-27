use std::path::PathBuf;
use masstemplate_config::{GlobalConfig, get_template_path};
use crate::CoreError;

pub struct TemplateFinder;

impl TemplateFinder {
    /// Find and validate a template by name
    pub fn find_template(config: &GlobalConfig, template_name: &str) -> Result<PathBuf, CoreError> {
        let template_path = get_template_path(config, template_name)?;
        
        if !template_path.exists() {
            return Err(CoreError::TemplateNotFound(template_name.to_string()));
        }
        
        if !template_path.is_dir() {
            return Err(CoreError::InvalidTemplatePath(template_path));
        }
        
        Ok(template_path)
    }
    
    /// List all available templates
    pub fn list_templates(config: &GlobalConfig) -> Result<Vec<String>, CoreError> {
        let templates_dir = masstemplate_config::get_templates_directory(config)?;
        
        if !templates_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut templates = Vec::new();
        for entry in std::fs::read_dir(templates_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    templates.push(name.to_string());
                }
            }
        }
        
        templates.sort();
        Ok(templates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_find_template_exists() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("my_template");
        std::fs::create_dir(&template_dir).unwrap();
        
        let mut config = GlobalConfig::default();
        config.template_directory = Some(temp_dir.path().to_string_lossy().to_string());
        
        let result = TemplateFinder::find_template(&config, "my_template");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), template_dir);
    }
    
    #[tokio::test]
    async fn test_find_template_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        
        let mut config = GlobalConfig::default();
        config.template_directory = Some(temp_dir.path().to_string_lossy().to_string());
        
        let result = TemplateFinder::find_template(&config, "nonexistent");
        assert!(matches!(result, Err(CoreError::TemplateNotFound(_))));
    }
}
