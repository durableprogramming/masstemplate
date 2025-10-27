use serde::{Deserialize, Serialize};

/// Global configuration for masstemplate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Default collision strategy to use when applying templates
    pub default_collision_strategy: Option<String>,
    /// Whether to show verbose output
    pub verbose: Option<bool>,
    /// Custom template directory (defaults to ~/.local/masstemplate/)
    pub template_directory: Option<String>,
    /// Additional template source directories
    pub template_sources: Option<Vec<String>>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            default_collision_strategy: Some("skip".to_string()),
            verbose: Some(false),
            template_directory: None,
            template_sources: Some(Vec::new()),
        }
    }
}

/// Template-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Human-readable name for the template
    pub name: Option<String>,
    /// Description of what this template provides
    pub description: Option<String>,
    /// Default collision strategy for this template
    pub collision_strategy: Option<String>,
    /// Tags for categorizing templates
    pub tags: Option<Vec<String>>,
    /// Version of the template
    pub version: Option<String>,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            collision_strategy: None,
            tags: None,
            version: None,
        }
    }
}

/// Information about a discovered template
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub path: std::path::PathBuf,
    pub config: Option<TemplateConfig>,
}