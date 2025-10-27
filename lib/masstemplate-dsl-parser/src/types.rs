use serde::{Deserialize, Serialize};
use masstemplate_processors::Processor;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollisionStrategy {
    Skip,
    Overwrite,
    Backup,
    Merge,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Matcher {
    pub pattern: String,
    pub processors: Vec<Processor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DslConfig {
    pub collision_strategy: Option<CollisionStrategy>,
    pub processors: Vec<Processor>,
    pub matchers: Vec<Matcher>,
    pub recursive: Option<bool>,
    pub priority: Option<i32>,
}

impl Default for DslConfig {
    fn default() -> Self {
        Self {
            collision_strategy: Some(CollisionStrategy::Skip),
            processors: Vec::new(),
            matchers: Vec::new(),
            recursive: Some(true),
            priority: Some(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use masstemplate_processors::Processor;

    #[test]
    fn test_collision_strategy_serialization() {
        let strategy = CollisionStrategy::Skip;
        let serialized = serde_json::to_string(&strategy).unwrap();
        let deserialized: CollisionStrategy = serde_json::from_str(&serialized).unwrap();
        assert_eq!(strategy, deserialized);
    }

    #[test]
    fn test_matcher_serialization() {
        let matcher = Matcher {
            pattern: "*.txt".to_string(),
            processors: vec![Processor::Replace {
                pattern: "old".to_string(),
                replacement: "new".to_string(),
            }],
        };
        let serialized = serde_json::to_string(&matcher).unwrap();
        let deserialized: Matcher = serde_json::from_str(&serialized).unwrap();
        assert_eq!(matcher, deserialized);
    }

    #[test]
    fn test_dsl_config_default() {
        let config = DslConfig::default();
        assert_eq!(config.collision_strategy, Some(CollisionStrategy::Skip));
        assert!(config.processors.is_empty());
        assert!(config.matchers.is_empty());
        assert_eq!(config.recursive, Some(true));
        assert_eq!(config.priority, Some(0));
    }

    #[test]
    fn test_dsl_config_serialization() {
        let config = DslConfig {
            collision_strategy: Some(CollisionStrategy::Overwrite),
            processors: vec![Processor::Template {
                variables: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("KEY".to_string(), "value".to_string());
                    map
                },
            }],
            matchers: vec![Matcher {
                pattern: "*.md".to_string(),
                processors: vec![Processor::Replace {
                    pattern: "placeholder".to_string(),
                    replacement: "replaced".to_string(),
                }],
            }],
            recursive: Some(false),
            priority: Some(10),
        };
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: DslConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_collision_strategy_variants() {
        // Test that all variants can be created and compared
        assert_eq!(CollisionStrategy::Skip, CollisionStrategy::Skip);
        assert_eq!(CollisionStrategy::Overwrite, CollisionStrategy::Overwrite);
        assert_eq!(CollisionStrategy::Backup, CollisionStrategy::Backup);
        assert_eq!(CollisionStrategy::Merge, CollisionStrategy::Merge);

        assert_ne!(CollisionStrategy::Skip, CollisionStrategy::Overwrite);
    }
}