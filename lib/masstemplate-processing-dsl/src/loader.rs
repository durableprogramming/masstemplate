use crate::config::{DirectoryConfig, FileConfig, MergedConfig};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration loader that handles directory traversal and config merging
#[derive(Debug)]
pub struct ConfigLoader {
    /// Cache of loaded directory configs
    dir_configs: HashMap<PathBuf, DirectoryConfig>,
    /// Cache of loaded file configs
    file_configs: HashMap<PathBuf, FileConfig>,
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            dir_configs: HashMap::new(),
            file_configs: HashMap::new(),
        }
    }

    /// Load configuration for a specific file path
    pub fn load_for_file(&mut self, file_path: &PathBuf) -> anyhow::Result<MergedConfig> {
        let mut merged = MergedConfig::default();

        // Load directory configs from root to the file's directory
        if let Some(parent) = file_path.parent() {
            let dir_configs = self.load_directory_configs(parent)?;
            self.merge_directory_configs(&mut merged, &dir_configs);
        }

        // Load file-specific config if it exists
        if let Some(file_config) = self.load_file_config(file_path)? {
            self.merge_file_config(&mut merged, &file_config);
        }

        Ok(merged)
    }

    /// Load all directory configs from root to the given directory
    fn load_directory_configs(&mut self, dir_path: &Path) -> anyhow::Result<Vec<DirectoryConfig>> {
        let mut configs = Vec::new();
        let mut current = Some(dir_path);

        while let Some(path) = current {
            if let Some(config) = self.load_directory_config(path)? {
                configs.push(config);
            }

            // Stop if we reach a non-recursive config
            if let Some(last_config) = configs.last() {
                if !last_config.recursive {
                    break;
                }
            }

            current = path.parent();
        }

        // Reverse so root configs come first
        configs.reverse();
        Ok(configs)
    }

    /// Load directory config from .mtem/config in the given directory
    fn load_directory_config(&mut self, dir_path: &Path) -> anyhow::Result<Option<DirectoryConfig>> {
        if let Some(cached) = self.dir_configs.get(dir_path) {
            return Ok(Some(cached.clone()));
        }

        let config_path = dir_path.join(".mtem").join("config");
        if !config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&config_path)?;
        let dsl_config = masstemplate_dsl_parser::parse_dsl(&content).map_err(anyhow::Error::msg)?;

        let recursive = dsl_config.recursive.unwrap_or(true);
        let priority = dsl_config.priority.unwrap_or(0);

        let config = DirectoryConfig {
            dsl_config,
            recursive,
            priority,
        };

        self.dir_configs.insert(dir_path.to_path_buf(), config.clone());
        Ok(Some(config))
    }

    /// Load file-specific config
    fn load_file_config(&mut self, file_path: &PathBuf) -> anyhow::Result<Option<FileConfig>> {
        if let Some(cached) = self.file_configs.get(file_path) {
            return Ok(Some(cached.clone()));
        }

        // Look for FILENAME.ext.mtem.config
        let file_name = file_path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
            .to_string_lossy();
        let config_file_name = format!("{}.mtem.config", file_name);
        let config_path = file_path.with_file_name(config_file_name);

        if !config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&config_path)?;
        let dsl_config = masstemplate_dsl_parser::parse_dsl(&content).map_err(anyhow::Error::msg)?;

        let priority = dsl_config.priority.unwrap_or(0);

        let config = FileConfig {
            dsl_config,
            priority,
        };

        self.file_configs.insert(file_path.clone(), config.clone());
        Ok(Some(config))
    }

    /// Merge directory configs into the merged config
    fn merge_directory_configs(&self, merged: &mut MergedConfig, dir_configs: &[DirectoryConfig]) {
        // Sort by priority (higher priority configs override lower ones)
        let mut sorted_configs: Vec<_> = dir_configs.iter().collect();
        sorted_configs.sort_by_key(|c| c.priority);

        for config in sorted_configs {
            if let Some(ref strategy) = config.dsl_config.collision_strategy {
                merged.collision_strategy = Some(strategy.clone());
            }
            merged.processors.extend(config.dsl_config.processors.iter().cloned());
            merged.matchers.extend(config.dsl_config.matchers.iter().cloned());
        }
    }

    /// Merge file config into the merged config
    fn merge_file_config(&self, merged: &mut MergedConfig, file_config: &FileConfig) {
        // File configs override directory configs
        if let Some(ref strategy) = file_config.dsl_config.collision_strategy {
            merged.collision_strategy = Some(strategy.clone());
        }
        merged.processors.extend(file_config.dsl_config.processors.iter().cloned());
        merged.matchers.extend(file_config.dsl_config.matchers.iter().cloned());
    }
}