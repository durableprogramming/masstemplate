use masstemplate_dsl_parser::{CollisionStrategy, DslConfig, Matcher};
use masstemplate_processors::Processor;
use serde::{Deserialize, Serialize};

/// Configuration for a directory, stored in .mtem/config
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectoryConfig {
    /// The DSL configuration for this directory
    pub dsl_config: DslConfig,
    /// Whether this config applies recursively to subdirectories
    /// (default: true)
    pub recursive: bool,
    /// Priority for this config (higher values override lower ones)
    pub priority: i32,
}

impl Default for DirectoryConfig {
    fn default() -> Self {
        Self {
            dsl_config: DslConfig::default(),
            recursive: true,
            priority: 0,
        }
    }
}

/// Configuration for a specific file, stored in FILENAME.ext.mtem.config
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileConfig {
    /// The DSL configuration for this file
    pub dsl_config: DslConfig,
    /// Priority for this config (higher values override lower ones)
    pub priority: i32,
}

/// Combined configuration for a file, merging directory and file-specific configs
#[derive(Debug, Clone, PartialEq)]
pub struct MergedConfig {
    /// The final collision strategy to use
    pub collision_strategy: Option<CollisionStrategy>,
    /// The list of processors to apply, in order
    pub processors: Vec<Processor>,
    /// The list of matchers with their processors
    pub matchers: Vec<Matcher>,
}

impl Default for MergedConfig {
    fn default() -> Self {
        Self {
            collision_strategy: Some(CollisionStrategy::Skip),
            processors: Vec::new(),
            matchers: Vec::new(),
        }
    }
}