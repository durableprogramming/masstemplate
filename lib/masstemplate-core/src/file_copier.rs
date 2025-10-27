use std::path::Path;
use masstemplate_fileops::{FileOps, CollisionStrategy};
use masstemplate_processing_dsl::DslProcessor;
use crate::CoreError;

pub struct TemplateFileCopier {
    dsl_processor: DslProcessor,
}

impl TemplateFileCopier {
    pub fn new() -> Self {
        Self {
            dsl_processor: DslProcessor::new(),
        }
    }
    
    /// Copy template files to destination with processing
    pub async fn copy_template_files(
        &mut self,
        template_dir: &Path,
        destination_dir: &Path,
        ignore_patterns: &[String],
    ) -> Result<(), CoreError> {
        self.copy_template_files_with_strategy(template_dir, destination_dir, ignore_patterns, CollisionStrategy::Skip).await
    }

    /// Copy template files to destination with processing and specific collision strategy
    pub async fn copy_template_files_with_strategy(
        &mut self,
        template_dir: &Path,
        destination_dir: &Path,
        ignore_patterns: &[String],
        strategy: CollisionStrategy,
    ) -> Result<(), CoreError> {
        // First, copy all files with specified collision strategy
        let fileops = FileOps::new(strategy);
        fileops.copy_dir_contents_with_ignore(template_dir, destination_dir, ignore_patterns)?;
        
        // Then apply DSL processing to the copied files
        self.dsl_processor.process_directory(&destination_dir.to_path_buf())
            .map_err(|e| CoreError::Generic(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get collision strategy for a specific file from DSL config
    pub fn get_collision_strategy(&mut self, file_path: &Path) -> Result<Option<masstemplate_fileops::CollisionStrategy>, CoreError> {
        let dsl_strategy = self.dsl_processor.get_collision_strategy(&file_path.to_path_buf())
            .map_err(|e| CoreError::Generic(e.to_string()))?;
        Ok(dsl_strategy.map(|s| match s {
            masstemplate_dsl_parser::CollisionStrategy::Skip => masstemplate_fileops::CollisionStrategy::Skip,
            masstemplate_dsl_parser::CollisionStrategy::Overwrite => masstemplate_fileops::CollisionStrategy::Overwrite,
            masstemplate_dsl_parser::CollisionStrategy::Backup => masstemplate_fileops::CollisionStrategy::Backup,
            masstemplate_dsl_parser::CollisionStrategy::Merge => masstemplate_fileops::CollisionStrategy::Merge,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[tokio::test]
    async fn test_copy_template_files() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("template");
        let dest_dir = temp_dir.path().join("dest");

        fs::create_dir(&template_dir).unwrap();
        fs::create_dir(&dest_dir).unwrap();

        // Create a test file in template
        fs::write(template_dir.join("test.txt"), "Hello {{NAME}}").unwrap();

        let mut copier = TemplateFileCopier::new();
        copier.copy_template_files(&template_dir, &dest_dir, &[]).await.unwrap();

        assert!(dest_dir.join("test.txt").exists());
        let content = fs::read_to_string(dest_dir.join("test.txt")).unwrap();
        assert_eq!(content, "Hello {{NAME}}"); // No processing applied yet
    }

    #[tokio::test]
    async fn test_copy_template_files_with_skip_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("template");
        let dest_dir = temp_dir.path().join("dest");

        fs::create_dir(&template_dir).unwrap();
        fs::create_dir(&dest_dir).unwrap();

        // Create existing file in destination
        fs::write(dest_dir.join("existing.txt"), "Original content").unwrap();

        // Create same file in template
        fs::write(template_dir.join("existing.txt"), "Template content").unwrap();

        let mut copier = TemplateFileCopier::new();
        copier.copy_template_files_with_strategy(&template_dir, &dest_dir, &[], CollisionStrategy::Skip).await.unwrap();

        // With Skip strategy, original content should be preserved
        let content = fs::read_to_string(dest_dir.join("existing.txt")).unwrap();
        assert_eq!(content, "Original content");
    }

    #[tokio::test]
    async fn test_copy_template_files_with_overwrite_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("template");
        let dest_dir = temp_dir.path().join("dest");

        fs::create_dir(&template_dir).unwrap();
        fs::create_dir(&dest_dir).unwrap();

        // Create existing file in destination
        fs::write(dest_dir.join("existing.txt"), "Original content").unwrap();

        // Create same file in template
        fs::write(template_dir.join("existing.txt"), "Template content").unwrap();

        let mut copier = TemplateFileCopier::new();
        copier.copy_template_files_with_strategy(&template_dir, &dest_dir, &[], CollisionStrategy::Overwrite).await.unwrap();

        // With Overwrite strategy, template content should replace original
        let content = fs::read_to_string(dest_dir.join("existing.txt")).unwrap();
        assert_eq!(content, "Template content");
    }

    #[tokio::test]
    async fn test_copy_template_files_with_backup_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("template");
        let dest_dir = temp_dir.path().join("dest");

        fs::create_dir(&template_dir).unwrap();
        fs::create_dir(&dest_dir).unwrap();

        // Create existing file in destination
        fs::write(dest_dir.join("existing.txt"), "Original content").unwrap();

        // Create same file in template
        fs::write(template_dir.join("existing.txt"), "Template content").unwrap();

        let mut copier = TemplateFileCopier::new();
        copier.copy_template_files_with_strategy(&template_dir, &dest_dir, &[], CollisionStrategy::Backup).await.unwrap();

        // With Backup strategy, original should be backed up and template content should be written
        let content = fs::read_to_string(dest_dir.join("existing.txt")).unwrap();
        assert_eq!(content, "Template content");

        // Check backup file exists (format: existing.backup.N.txt)
        let backup_files: Vec<_> = fs::read_dir(&dest_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("existing.backup.") && name.ends_with(".txt")
            })
            .collect();
        assert_eq!(backup_files.len(), 1);

        let backup_content = fs::read_to_string(backup_files[0].path()).unwrap();
        assert_eq!(backup_content, "Original content");
    }
}
