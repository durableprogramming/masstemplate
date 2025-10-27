use crate::loader::ConfigLoader;
use masstemplate_dsl_parser::CollisionStrategy;
use masstemplate_processors::Processor;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Processor that applies DSL configurations to files and directories
#[derive(Debug)]
pub struct DslProcessor {
    loader: ConfigLoader,
}

impl Default for DslProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl DslProcessor {
    pub fn new() -> Self {
        Self {
            loader: ConfigLoader::new(),
        }
    }

    /// Process all files in a directory tree according to their DSL configurations
    pub fn process_directory(&mut self, root_dir: &PathBuf) -> anyhow::Result<()> {
        for entry in WalkDir::new(root_dir) {
            let entry = entry?;
            let path = entry.path().to_path_buf();

            // Skip .mtem directories and config files
            if path.components().any(|c| c.as_os_str() == ".mtem") {
                continue;
            }
            if path.file_name().is_some_and(|name| name.to_string_lossy().ends_with(".mtem.config")) {
                continue;
            }

            if path.is_file() {
                self.process_file(&path)?;
            }
        }
        Ok(())
    }

    /// Process a single file according to its DSL configuration
    pub fn process_file(&mut self, file_path: &PathBuf) -> anyhow::Result<()> {
        let config = self.loader.load_for_file(file_path)?;

        // Collect all processors to apply
        let mut all_processors = config.processors.clone();

        // Apply matcher processors if file matches pattern
        for matcher in &config.matchers {
            if self.file_matches_pattern(file_path, &matcher.pattern) {
                all_processors.extend(matcher.processors.iter().cloned());
            }
        }

        // Apply all processors at once
        if !all_processors.is_empty() {
            self.apply_processors(file_path, &all_processors)?;
        }

        Ok(())
    }

    /// Apply multiple processors to a file
    fn apply_processors(&self, file_path: &PathBuf, processors: &[Processor]) -> anyhow::Result<()> {
        let content = fs::read_to_string(file_path)?;
        let new_content = masstemplate_processors::apply_content_processors(file_path, &content, processors)
            .map_err(anyhow::Error::msg)?;
        fs::write(file_path, new_content)?;

        // Check if any processors modify filename
        let has_filename_processors = processors.iter().any(|p| matches!(p, Processor::Jinja2Filename { .. } | Processor::ReplaceFilename { .. }));
        if has_filename_processors {
            let new_filename = masstemplate_processors::apply_filename_processors(file_path, processors)
                .map_err(anyhow::Error::msg)?;
            if new_filename != file_path.file_name().and_then(|n| n.to_str()).unwrap_or("") {
                let new_path = file_path.with_file_name(new_filename);
                fs::rename(file_path, &new_path)?;
            }
        }

        Ok(())
    }



    /// Check if a file matches a glob pattern
    fn file_matches_pattern(&self, file_path: &Path, pattern: &str) -> bool {
        use glob::Pattern;

        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        match Pattern::new(pattern) {
            Ok(pat) => pat.matches(file_name),
            Err(_) => false, // Invalid pattern, no match
        }
    }

    /// Get the collision strategy for a file
    pub fn get_collision_strategy(&mut self, file_path: &PathBuf) -> anyhow::Result<Option<CollisionStrategy>> {
        let config = self.loader.load_for_file(file_path)?;
        Ok(config.collision_strategy)
    }
}