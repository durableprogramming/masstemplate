use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;
use glob::Pattern;

#[derive(Error, Debug)]
pub enum FileOpsError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to create backup: {0}")]
    Backup(String),
    #[error("Merge conflict: {0}")]
    MergeConflict(String),
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionStrategy {
    Skip,
    Overwrite,
    Backup,
    Merge,
}

impl Default for CollisionStrategy {
    fn default() -> Self {
        CollisionStrategy::Skip
    }
}

impl FromStr for CollisionStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "skip" => Ok(CollisionStrategy::Skip),
            "overwrite" => Ok(CollisionStrategy::Overwrite),
            "backup" => Ok(CollisionStrategy::Backup),
            "merge" => Ok(CollisionStrategy::Merge),
            _ => Err(format!("Unknown collision strategy: {}", s)),
        }
    }
}

pub struct FileOps {
    strategy: CollisionStrategy,
}

impl FileOps {
    pub fn new(strategy: CollisionStrategy) -> Self {
        Self { strategy }
    }

    pub fn copy_file(&self, src: &Path, dst: &Path) -> Result<(), FileOpsError> {
        if dst.exists() {
            match self.strategy {
                CollisionStrategy::Skip => {
                    println!("Skipping existing file: {}", dst.display());
                    return Ok(());
                }
                CollisionStrategy::Overwrite => {
                    // Just proceed with copy
                }
                CollisionStrategy::Backup => {
                    self.create_backup(dst)?;
                }
                CollisionStrategy::Merge => {
                    return self.merge_files(src, dst);
                }
            }
        }

        fs::copy(src, dst)?;
        Ok(())
    }

    pub fn copy_dir_contents(&self, src: &Path, dst: &Path) -> Result<(), FileOpsError> {
        self.copy_dir_contents_with_ignore(src, dst, &[])
    }

    pub fn copy_dir_contents_with_ignore(&self, src: &Path, dst: &Path, ignore_patterns: &[String]) -> Result<(), FileOpsError> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let file_name = src_path.file_name().ok_or_else(|| {
                FileOpsError::InvalidPath("Invalid file path in source".to_string())
            })?;

            // Skip .mtem directory
            if file_name == ".mtem" {
                continue;
            }

            // Check if file matches any ignore pattern
            let relative_path = src_path.strip_prefix(src).unwrap_or(&src_path);
            let path_str = relative_path.to_string_lossy();
            let should_ignore = ignore_patterns.iter().any(|pattern| {
                if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                    if let Ok(p) = Pattern::new(pattern) {
                        p.matches(&path_str)
                    } else {
                        false
                    }
                } else {
                    // Exact match or directory match
                    &*path_str == pattern || path_str.starts_with(&(pattern.to_string() + "/"))
                }
            });

            if should_ignore {
                println!("Ignoring file: {}", path_str);
                continue;
            }

            let dst_path = dst.join(file_name);

            if src_path.is_dir() {
                fs::create_dir_all(&dst_path)?;
                self.copy_dir_contents_with_ignore(&src_path, &dst_path, ignore_patterns)?;
            } else {
                self.copy_file(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    fn create_backup(&self, path: &Path) -> Result<(), FileOpsError> {
        let mut backup_path = path.to_path_buf();
        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| FileOpsError::InvalidPath("Invalid file name".to_string()))?;
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let mut counter = 1;
        loop {
            let backup_name = if extension.is_empty() {
                format!("{}.backup.{}", file_stem, counter)
            } else {
                format!("{}.backup.{}.{}", file_stem, counter, extension)
            };
            backup_path.set_file_name(backup_name);

            if !backup_path.exists() {
                fs::copy(path, &backup_path)?;
                println!("Created backup: {}", backup_path.display());
                return Ok(());
            }
            counter += 1;
            if counter > 1000 {
                return Err(FileOpsError::Backup("Too many backup files".to_string()));
            }
        }
    }

    fn merge_files(&self, src: &Path, dst: &Path) -> Result<(), FileOpsError> {
        let src_content = fs::read_to_string(src)?;
        let dst_content = fs::read_to_string(dst)?;

        let merged_content = if let Some(extension) = src.extension() {
            match extension.to_str() {
                Some("json") => self.merge_json(&src_content, &dst_content)?,
                Some("yaml") | Some("yml") => self.merge_yaml(&src_content, &dst_content)?,
                _ => self.merge_text(&src_content, &dst_content),
            }
        } else {
            self.merge_text(&src_content, &dst_content)
        };

        fs::write(dst, merged_content)?;
        println!("Merged file: {}", dst.display());
        Ok(())
    }

    fn merge_json(&self, src_content: &str, dst_content: &str) -> Result<String, FileOpsError> {
        let src_json: Result<serde_json::Value, _> = serde_json::from_str(src_content);
        let dst_json: Result<serde_json::Value, _> = serde_json::from_str(dst_content);

        if let (Ok(serde_json::Value::Object(src_map)), Ok(serde_json::Value::Object(mut dst_map))) = (src_json, dst_json) {
            // Merge src into dst, with src values taking precedence for conflicts
            for (key, value) in src_map {
                dst_map.insert(key, value);
            }
            Ok(serde_json::to_string_pretty(&serde_json::Value::Object(dst_map)).unwrap())
        } else {
            // If not both valid JSON objects, fall back to text merge
            Ok(self.merge_text(src_content, dst_content))
        }
    }

    fn merge_yaml(&self, src_content: &str, dst_content: &str) -> Result<String, FileOpsError> {
        // For now, fall back to text merge for YAML
        // Could implement proper YAML merging if needed
        Ok(self.merge_text(src_content, dst_content))
    }

    fn merge_text(&self, src_content: &str, dst_content: &str) -> String {
        format!("{}\n--- MERGED FROM TEMPLATE ---\n{}", dst_content, src_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;
    use std::path::PathBuf;

    fn setup_test_dirs(test_name: &str) -> (PathBuf, PathBuf, PathBuf) {
        let temp_dir = env::temp_dir().join(format!("masstemplate_fileops_test_{}", test_name));
        let src_dir = temp_dir.join("src");
        let dst_dir = temp_dir.join("dst");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);

        // Create directories
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&dst_dir).unwrap();

        (temp_dir, src_dir, dst_dir)
    }

    fn cleanup_test_dirs(temp_dir: &Path) {
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_skip_strategy() {
        let (temp_dir, src_dir, dst_dir) = setup_test_dirs("skip");

        // Create source file
        fs::write(src_dir.join("test.txt"), "source content").unwrap();
        // Create existing destination file
        fs::write(dst_dir.join("test.txt"), "existing content").unwrap();

        let fileops = FileOps::new(CollisionStrategy::Skip);
        fileops.copy_file(&src_dir.join("test.txt"), &dst_dir.join("test.txt")).unwrap();

        // Should still have existing content
        let content = fs::read_to_string(dst_dir.join("test.txt")).unwrap();
        assert_eq!(content, "existing content");

        cleanup_test_dirs(&temp_dir);
    }

    #[test]
    fn test_overwrite_strategy() {
        let (temp_dir, src_dir, dst_dir) = setup_test_dirs("overwrite");

        let src_file = src_dir.join("test.txt");
        let dst_file = dst_dir.join("test.txt");

        fs::write(&src_file, "source content").unwrap();
        fs::write(&dst_file, "existing content").unwrap();

        let fileops = FileOps::new(CollisionStrategy::Overwrite);
        fileops.copy_file(&src_file, &dst_file).unwrap();

        let content = fs::read_to_string(&dst_file).unwrap();
        assert_eq!(content, "source content");

        cleanup_test_dirs(&temp_dir);
    }

    #[test]
    fn test_backup_strategy() {
        let (temp_dir, src_dir, dst_dir) = setup_test_dirs("backup");

        let src_file = src_dir.join("test.txt");
        let dst_file = dst_dir.join("test.txt");

        fs::write(&src_file, "source content").unwrap();
        fs::write(&dst_file, "existing content").unwrap();

        let fileops = FileOps::new(CollisionStrategy::Backup);
        fileops.copy_file(&src_file, &dst_file).unwrap();

        // Find the backup file (should be the first available)
        let backup_files = fs::read_dir(&dst_dir).unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("test.backup.") && n.ends_with(".txt"))
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();

        assert_eq!(backup_files.len(), 1);
        let backup_content = fs::read_to_string(&backup_files[0]).unwrap();
        assert_eq!(backup_content, "existing content");

        // New file should have source content
        let content = fs::read_to_string(&dst_file).unwrap();
        assert_eq!(content, "source content");

        cleanup_test_dirs(&temp_dir);
    }

    #[test]
    fn test_merge_strategy() {
        let (temp_dir, src_dir, dst_dir) = setup_test_dirs("merge");

        fs::write(src_dir.join("test.txt"), "source content").unwrap();
        fs::write(dst_dir.join("test.txt"), "existing content").unwrap();

        let fileops = FileOps::new(CollisionStrategy::Merge);
        fileops.copy_file(&src_dir.join("test.txt"), &dst_dir.join("test.txt")).unwrap();

        let content = fs::read_to_string(dst_dir.join("test.txt")).unwrap();
        assert!(content.contains("existing content"));
        assert!(content.contains("source content"));
        assert!(content.contains("--- MERGED FROM TEMPLATE ---"));

        cleanup_test_dirs(&temp_dir);
    }

    #[test]
    fn test_merge_json_objects() {
        let (temp_dir, src_dir, dst_dir) = setup_test_dirs("merge_json");

        let src_json = r#"{"name": "template", "version": "1.0.0", "new_field": "value"}"#;
        let dst_json = r#"{"name": "existing", "description": "existing desc"}"#;
        let expected = r#"{
  "description": "existing desc",
  "name": "template",
  "new_field": "value",
  "version": "1.0.0"
}"#;

        fs::write(src_dir.join("config.json"), src_json).unwrap();
        fs::write(dst_dir.join("config.json"), dst_json).unwrap();

        let fileops = FileOps::new(CollisionStrategy::Merge);
        fileops.copy_file(&src_dir.join("config.json"), &dst_dir.join("config.json")).unwrap();

        let content = fs::read_to_string(dst_dir.join("config.json")).unwrap();
        // Parse both to compare as JSON (order doesn't matter)
        let parsed_content: serde_json::Value = serde_json::from_str(&content).unwrap();
        let parsed_expected: serde_json::Value = serde_json::from_str(expected).unwrap();
        assert_eq!(parsed_content, parsed_expected);

        cleanup_test_dirs(&temp_dir);
    }

    #[test]
    fn test_merge_invalid_json_fallback() {
        let (temp_dir, src_dir, dst_dir) = setup_test_dirs("merge_invalid_json");

        fs::write(src_dir.join("config.json"), "invalid json").unwrap();
        fs::write(dst_dir.join("config.json"), "existing content").unwrap();

        let fileops = FileOps::new(CollisionStrategy::Merge);
        fileops.copy_file(&src_dir.join("config.json"), &dst_dir.join("config.json")).unwrap();

        let content = fs::read_to_string(dst_dir.join("config.json")).unwrap();
        // Should fall back to text merge
        assert!(content.contains("existing content"));
        assert!(content.contains("invalid json"));
        assert!(content.contains("--- MERGED FROM TEMPLATE ---"));

        cleanup_test_dirs(&temp_dir);
    }

    #[test]
    fn test_merge_yaml_fallback() {
        let (temp_dir, src_dir, dst_dir) = setup_test_dirs("merge_yaml");

        fs::write(src_dir.join("config.yaml"), "key: template_value").unwrap();
        fs::write(dst_dir.join("config.yaml"), "existing: value").unwrap();

        let fileops = FileOps::new(CollisionStrategy::Merge);
        fileops.copy_file(&src_dir.join("config.yaml"), &dst_dir.join("config.yaml")).unwrap();

        let content = fs::read_to_string(dst_dir.join("config.yaml")).unwrap();
        // Should use text merge for YAML
        assert!(content.contains("existing: value"));
        assert!(content.contains("key: template_value"));
        assert!(content.contains("--- MERGED FROM TEMPLATE ---"));

        cleanup_test_dirs(&temp_dir);
    }

    #[test]
    fn test_copy_dir_contents() {
        let (temp_dir, src_dir, dst_dir) = setup_test_dirs("copy_dir");

        // Create source structure
        fs::create_dir_all(src_dir.join("subdir")).unwrap();
        fs::write(src_dir.join("file1.txt"), "content1").unwrap();
        fs::write(src_dir.join("subdir").join("file2.txt"), "content2").unwrap();

        let fileops = FileOps::new(CollisionStrategy::Skip);
        fileops.copy_dir_contents(&src_dir, &dst_dir).unwrap();

        assert!(dst_dir.join("file1.txt").exists());
        assert!(dst_dir.join("subdir").join("file2.txt").exists());
        assert_eq!(fs::read_to_string(dst_dir.join("file1.txt")).unwrap(), "content1");

        cleanup_test_dirs(&temp_dir);
    }

    #[test]
    fn test_copy_dir_contents_with_ignore() {
        let (temp_dir, src_dir, dst_dir) = setup_test_dirs("copy_ignore");

        // Create source structure
        fs::create_dir_all(src_dir.join("subdir")).unwrap();
        fs::create_dir_all(src_dir.join(".mtem")).unwrap();
        fs::write(src_dir.join("file1.txt"), "content1").unwrap();
        fs::write(src_dir.join("file2.log"), "content2").unwrap();
        fs::write(src_dir.join("subdir").join("file3.txt"), "content3").unwrap();
        fs::write(src_dir.join(".mtem").join("script.sh"), "script").unwrap();

        let ignore_patterns = vec!["*.log".to_string(), "subdir".to_string()];
        let fileops = FileOps::new(CollisionStrategy::Skip);
        fileops.copy_dir_contents_with_ignore(&src_dir, &dst_dir, &ignore_patterns).unwrap();

        // file1.txt should be copied
        assert!(dst_dir.join("file1.txt").exists());
        assert_eq!(fs::read_to_string(dst_dir.join("file1.txt")).unwrap(), "content1");

        // file2.log should be ignored
        assert!(!dst_dir.join("file2.log").exists());

        // subdir should be ignored
        assert!(!dst_dir.join("subdir").exists());

        // .mtem should be ignored
        assert!(!dst_dir.join(".mtem").exists());

        cleanup_test_dirs(&temp_dir);
    }

    #[test]
    fn test_backup_multiple_times() {
        let (temp_dir, src_dir, dst_dir) = setup_test_dirs("backup_multi");

        let src_file = src_dir.join("test.txt");
        let dst_file = dst_dir.join("test.txt");

        // Create initial file
        fs::write(&dst_file, "content1").unwrap();

        let fileops = FileOps::new(CollisionStrategy::Backup);

        // First backup
        fs::write(&src_file, "source1").unwrap();
        fileops.copy_file(&src_file, &dst_file).unwrap();

        // Second backup
        fs::write(&src_file, "source2").unwrap();
        fileops.copy_file(&src_file, &dst_file).unwrap();

        // Check that multiple backups exist
        let backup_files = fs::read_dir(&dst_dir).unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("test.backup.") && n.ends_with(".txt"))
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();

        assert!(backup_files.len() >= 2);

        cleanup_test_dirs(&temp_dir);
    }

    #[test]
    fn test_collision_strategy_from_str() {
        assert_eq!(CollisionStrategy::from_str("skip").unwrap(), CollisionStrategy::Skip);
        assert_eq!(CollisionStrategy::from_str("SKIP").unwrap(), CollisionStrategy::Skip);
        assert_eq!(CollisionStrategy::from_str("overwrite").unwrap(), CollisionStrategy::Overwrite);
        assert_eq!(CollisionStrategy::from_str("backup").unwrap(), CollisionStrategy::Backup);
        assert_eq!(CollisionStrategy::from_str("merge").unwrap(), CollisionStrategy::Merge);
        assert!(CollisionStrategy::from_str("invalid").is_err());
    }

    #[test]
    fn test_collision_strategy_default() {
        assert_eq!(CollisionStrategy::default(), CollisionStrategy::Skip);
    }

    #[test]
    fn test_copy_file_non_existent_source() {
        let (temp_dir, _, dst_dir) = setup_test_dirs("error");

        let fileops = FileOps::new(CollisionStrategy::Skip);
        let src = Path::new("non_existent.txt");
        let dst = dst_dir.join("test.txt");

        let result = fileops.copy_file(src, &dst);
        assert!(result.is_err());

        cleanup_test_dirs(&temp_dir);
    }
}