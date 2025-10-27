use std::path::Path;
use masstemplate_config::GlobalConfig;
use crate::{TemplateFinder, TemplateFileCopier, ScriptRunner, CoreError};

pub struct TemplateApplicator {
    config: GlobalConfig,
}

impl TemplateApplicator {
    pub fn new(config: GlobalConfig) -> Self {
        Self { config }
    }
    
    /// Apply a template to a destination directory
    pub async fn apply_template(
        &self,
        template_name: &str,
        destination_dir: &Path,
    ) -> Result<(), CoreError> {
        self.apply_template_with_strategy(template_name, destination_dir, masstemplate_fileops::CollisionStrategy::Skip).await
    }

    /// Apply a template to a destination directory with specific collision strategy
    pub async fn apply_template_with_strategy(
        &self,
        template_name: &str,
        destination_dir: &Path,
        strategy: masstemplate_fileops::CollisionStrategy,
    ) -> Result<(), CoreError> {
        // Find the template
        let template_path = TemplateFinder::find_template(&self.config, template_name)?;

        // Run pre-install script
        ScriptRunner::run_pre_install_script(&template_path, destination_dir).await?;

        // Copy files with processing
        let mut file_copier = TemplateFileCopier::new();
        file_copier.copy_template_files_with_strategy(&template_path, destination_dir, &[], strategy).await?;

        // Run post-install script
        ScriptRunner::run_post_install_script(&template_path, destination_dir).await?;

        Ok(())
    }
    
    /// Apply template with custom ignore patterns
    pub async fn apply_template_with_ignore(
        &self,
        template_name: &str,
        destination_dir: &Path,
        ignore_patterns: &[String],
    ) -> Result<(), CoreError> {
        let template_path = TemplateFinder::find_template(&self.config, template_name)?;
        
        ScriptRunner::run_pre_install_script(&template_path, destination_dir).await?;
        
        let mut file_copier = TemplateFileCopier::new();
        file_copier.copy_template_files(&template_path, destination_dir, ignore_patterns).await?;
        
        ScriptRunner::run_post_install_script(&template_path, destination_dir).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[tokio::test]
    async fn test_apply_template() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("templates").join("test_template");
        let dest_dir = temp_dir.path().join("dest");

        // Create template structure
        fs::create_dir_all(&template_dir).unwrap();
        fs::create_dir_all(&dest_dir).unwrap();

        // Create a test file
        fs::write(template_dir.join("README.md"), "# Test Template").unwrap();

        // Create config
        let mut config = GlobalConfig::default();
        config.template_directory = Some(temp_dir.path().join("templates").to_string_lossy().to_string());

        let applicator = TemplateApplicator::new(config);
        applicator.apply_template("test_template", &dest_dir).await.unwrap();

        // Check file was copied
        assert!(dest_dir.join("README.md").exists());
        let content = fs::read_to_string(dest_dir.join("README.md")).unwrap();
        assert_eq!(content, "# Test Template");
    }

    #[tokio::test]
    async fn test_apply_template_with_skip_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("templates").join("test_template");
        let dest_dir = temp_dir.path().join("dest");

        // Create template structure
        fs::create_dir_all(&template_dir).unwrap();
        fs::create_dir_all(&dest_dir).unwrap();

        // Create existing file in destination
        fs::write(dest_dir.join("config.yml"), "existing: true").unwrap();

        // Create same file in template
        fs::write(template_dir.join("config.yml"), "template: true").unwrap();

        // Create config
        let mut config = GlobalConfig::default();
        config.template_directory = Some(temp_dir.path().join("templates").to_string_lossy().to_string());

        let applicator = TemplateApplicator::new(config);
        applicator.apply_template_with_strategy("test_template", &dest_dir, masstemplate_fileops::CollisionStrategy::Skip).await.unwrap();

        // With Skip strategy, existing file should be preserved
        let content = fs::read_to_string(dest_dir.join("config.yml")).unwrap();
        assert_eq!(content, "existing: true");
    }

    #[tokio::test]
    async fn test_apply_template_with_overwrite_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("templates").join("test_template");
        let dest_dir = temp_dir.path().join("dest");

        // Create template structure
        fs::create_dir_all(&template_dir).unwrap();
        fs::create_dir_all(&dest_dir).unwrap();

        // Create existing file in destination
        fs::write(dest_dir.join("config.yml"), "existing: true").unwrap();

        // Create same file in template
        fs::write(template_dir.join("config.yml"), "template: true").unwrap();

        // Create config
        let mut config = GlobalConfig::default();
        config.template_directory = Some(temp_dir.path().join("templates").to_string_lossy().to_string());

        let applicator = TemplateApplicator::new(config);
        applicator.apply_template_with_strategy("test_template", &dest_dir, masstemplate_fileops::CollisionStrategy::Overwrite).await.unwrap();

        // With Overwrite strategy, template file should replace existing
        let content = fs::read_to_string(dest_dir.join("config.yml")).unwrap();
        assert_eq!(content, "template: true");
    }

    #[tokio::test]
    async fn test_apply_template_with_backup_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("templates").join("test_template");
        let dest_dir = temp_dir.path().join("dest");

        // Create template structure
        fs::create_dir_all(&template_dir).unwrap();
        fs::create_dir_all(&dest_dir).unwrap();

        // Create existing file in destination
        fs::write(dest_dir.join("config.yml"), "existing: true").unwrap();

        // Create same file in template
        fs::write(template_dir.join("config.yml"), "template: true").unwrap();

        // Create config
        let mut config = GlobalConfig::default();
        config.template_directory = Some(temp_dir.path().join("templates").to_string_lossy().to_string());

        let applicator = TemplateApplicator::new(config);
        applicator.apply_template_with_strategy("test_template", &dest_dir, masstemplate_fileops::CollisionStrategy::Backup).await.unwrap();

        // With Backup strategy, template file should be written and original backed up
        let content = fs::read_to_string(dest_dir.join("config.yml")).unwrap();
        assert_eq!(content, "template: true");

        // Check backup exists (format: config.backup.N.yml)
        let backup_files: Vec<_> = fs::read_dir(&dest_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("config.backup.") && name.ends_with(".yml")
            })
            .collect();
        assert_eq!(backup_files.len(), 1);

        let backup_content = fs::read_to_string(backup_files[0].path()).unwrap();
        assert_eq!(backup_content, "existing: true");
    }
}
