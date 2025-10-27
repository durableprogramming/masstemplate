use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CopierConfig {
    #[serde(rename = "_template", skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    #[serde(rename = "_templates_suffix", skip_serializing_if = "Option::is_none")]
    pub templates_suffix: Option<String>,

    #[serde(rename = "_skip_if_exists", skip_serializing_if = "Option::is_none")]
    pub skip_if_exists: Option<Vec<String>>,

    #[serde(rename = "_envops", skip_serializing_if = "Option::is_none")]
    pub envops: Option<EnvOps>,

    #[serde(rename = "_tasks", skip_serializing_if = "Option::is_none")]
    pub tasks: Option<Vec<Task>>,

    #[serde(flatten)]
    pub questions: HashMap<String, Question>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EnvOps {
    pub block_start_string: Option<String>,
    pub block_end_string: Option<String>,
    pub variable_start_string: Option<String>,
    pub variable_end_string: Option<String>,
    pub comment_start_string: Option<String>,
    pub comment_end_string: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Task {
    Command(Vec<String>),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Question {
    #[serde(rename = "type")]
    pub question_type: QuestionType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub validator: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QuestionType {
    Str,
    Bool,
    Int,
    Float,
}

impl CopierConfig {
    pub fn load(path: &Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: CopierConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn get_variable_start(&self) -> &str {
        self.envops
            .as_ref()
            .and_then(|e| e.variable_start_string.as_deref())
            .unwrap_or("{{")
    }

    pub fn get_variable_end(&self) -> &str {
        self.envops
            .as_ref()
            .and_then(|e| e.variable_end_string.as_deref())
            .unwrap_or("}}")
    }

    pub fn get_block_start(&self) -> &str {
        self.envops
            .as_ref()
            .and_then(|e| e.block_start_string.as_deref())
            .unwrap_or("{%")
    }

    pub fn get_block_end(&self) -> &str {
        self.envops
            .as_ref()
            .and_then(|e| e.block_end_string.as_deref())
            .unwrap_or("%}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_parse_copier_config() {
        let yaml = r#"
_templates_suffix: ""
_envops:
  block_start_string: "{%"
  block_end_string: "%}"
  variable_start_string: "{{"
  variable_end_string: "}}"

project_name:
  type: str
  help: What is the name of your project?
  validator: >-
    {% if not (project_name | regex_search('^[a-zA-Z][a-zA-Z0-9_-]*$')) %}
    Project name must start with a letter
    {% endif %}

python_version:
  type: str
  help: Python version to use
  default: "3.11"

include_poetry:
  type: bool
  help: Include Poetry for dependency management?
  default: true

_tasks:
  - ["git", "init"]
  - ["git", "add", "."]
"#;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("copier.yml");
        fs::write(&config_path, yaml).unwrap();

        let config = CopierConfig::load(&config_path).unwrap();

        assert_eq!(config.templates_suffix, Some("".to_string()));
        assert!(config.questions.contains_key("project_name"));
        assert_eq!(config.questions["project_name"].question_type, QuestionType::Str);
        assert_eq!(config.questions["python_version"].default, Some(serde_json::Value::String("3.11".to_string())));
        assert_eq!(config.tasks.as_ref().unwrap().len(), 2);
    }
}
