pub mod error;
pub mod types;
pub mod processor;
pub mod processors;

pub use error::*;
pub use types::*;
pub use processor::*;

inventory::collect!(ProcessorMetadata);

#[derive(Debug)]
pub struct ProcessorMetadata {
    pub name: &'static str,
    pub description: &'static str,
}

pub fn apply_content_processors(
    file_path: &std::path::Path,
    content: &str,
    processors: &[Processor],
) -> Result<String, ProcessorError> {
    let mut result = content.to_string();

    for processor in processors {
        result = processor.process_content(file_path, &result)?;
    }

    Ok(result)
}

pub fn apply_filename_processors(
    file_path: &std::path::Path,
    processors: &[Processor],
) -> Result<String, ProcessorError> {
    let mut result = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    for processor in processors {
        result = processor.process_filename(file_path)?;
    }

    Ok(result)
}

pub fn apply_processors(
    file_path: &std::path::Path,
    content: &str,
    processors: &[Processor],
) -> Result<String, ProcessorError> {
    apply_content_processors(file_path, content, processors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use crate::processors::dotenv_set::set_env_var;
    use crate::processors::dotenv_append::append_env_var;

    #[test]
    fn test_dotenv_set_new_var() {
        let content = "EXISTING=value";
        let result = processors::dotenv_set::set_env_var(content, "NEW_KEY", "new_value").unwrap();
        println!("Result: {}", result);
        assert!(result.contains("NEW_KEY=new_value"));
        assert!(result.contains("EXISTING=value"));
    }

    #[test]
    fn test_dotenv_set_existing_var() {
        let content = "KEY=old_value\nOTHER=val";
        let result = processors::dotenv_set::set_env_var(content, "KEY", "new_value").unwrap();
        println!("Result: {}", result);
        assert!(result.contains("KEY=new_value"));
        assert!(result.contains("OTHER=val"));
        assert!(!result.contains("KEY=old_value"));
    }

    #[test]
    fn test_dotenv_append_existing_var() {
        let content = "KEY=prefix\nOTHER=val";
        let result = processors::dotenv_append::append_env_var(content, "KEY", "_suffix").unwrap();
        println!("Result: {}", result);
        assert!(result.contains("KEY=prefix_suffix"));
        assert!(result.contains("OTHER=val"));
    }

    #[test]
    fn test_dotenv_append_new_var() {
        let content = "EXISTING=value";
        let result = processors::dotenv_append::append_env_var(content, "NEW_KEY", "value").unwrap();
        println!("Result: {}", result);
        assert!(result.contains("NEW_KEY=value"));
        assert!(result.contains("EXISTING=value"));
    }

    #[test]
    fn test_template_replacement() {
        let processor = Processor::Template {
            variables: {
                let mut map = HashMap::new();
                map.insert("NAME".to_string(), "World".to_string());
                map.insert("GREETING".to_string(), "Hello".to_string());
                map
            },
        };

        let content = "{{GREETING}} {{NAME}}!";
        let result = processor.process_content(&PathBuf::from("test.txt"), content).unwrap();
        println!("Result: {}", result);
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_replace_processor() {
        let processor = Processor::Replace {
            pattern: "old".to_string(),
            replacement: "new".to_string(),
        };

        let content = "This is old content";
        let result = processor.process_content(&PathBuf::from("test.txt"), content).unwrap();
        println!("Result: {}", result);
        assert_eq!(result, "This is new content");
    }

    #[test]
    fn test_dotenv_processor_only_processes_env_files() {
        let processor = Processor::DotenvSet {
            key: "KEY".to_string(),
            value: "value".to_string(),
        };

        let content = "some content";
        let result = processor.process_content(&PathBuf::from("test.txt"), content).unwrap();
        println!("Result for non-env file: {}", result);
        assert_eq!(result, "some content"); // Should not modify non-.env files
    }

    #[test]
    fn test_apply_processors_chain() {
        let processors = vec![
            Processor::Replace {
                pattern: "world".to_string(),
                replacement: "World".to_string(),
            },
            Processor::Template {
                variables: {
                    let mut map = HashMap::new();
                    map.insert("GREETING".to_string(), "Hello".to_string());
                    map
                },
            },
        ];

        let content = "{{GREETING}} world!";
        let result = apply_processors(&PathBuf::from("test.txt"), content, &processors).unwrap();
        println!("Chained result: {}", result);
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_dotenv_set_with_empty_content() {
        let content = "";
        let result = set_env_var(content, "KEY", "value").unwrap();
        println!("Result for empty content: {}", result);
        assert_eq!(result, "KEY=value");
    }

    #[test]
    fn test_dotenv_append_with_empty_value() {
        let content = "KEY=\nOTHER=value";
        let result = append_env_var(content, "KEY", "suffix").unwrap();
        println!("Result for append to empty value: {}", result);
        assert!(result.contains("KEY=suffix"));
    }

    #[test]
    fn test_template_with_no_variables() {
        let processor = Processor::Template {
            variables: HashMap::new(),
        };

        let content = "No variables here";
        let result = processor.process_content(&PathBuf::from("test.txt"), content).unwrap();
        println!("Result with no variables: {}", result);
        assert_eq!(result, "No variables here");
    }

    #[test]
    fn test_template_with_nested_braces() {
        let processor = Processor::Template {
            variables: {
                let mut map = HashMap::new();
                map.insert("VAR".to_string(), "value".to_string());
                map
            },
        };

        let content = "{{VAR}} and {{{{not a var}}}}";
        let result = processor.process_content(&PathBuf::from("test.txt"), content).unwrap();
        println!("Result with nested braces: {}", result);
        assert_eq!(result, "value and {{{{not a var}}}}");
    }

    #[test]
    fn test_jinja2_content_processor() {
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), serde_json::Value::String("World".to_string()));
        variables.insert("greeting".to_string(), serde_json::Value::String("Hello".to_string()));

        let processor = Processor::Jinja2Content { variables };

        let content = "{{ greeting }} {{ name }}!";
        let result = processor.process_content(&PathBuf::from("test.txt"), content).unwrap();
        println!("Jinja2 content result: {}", result);
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_jinja2_filename_processor() {
        let mut variables = HashMap::new();
        variables.insert("project_name".to_string(), serde_json::Value::String("myapp".to_string()));

        let processor = Processor::Jinja2Filename { variables };

        let result = processor.process_filename(&PathBuf::from("{{ project_name }}.rs")).unwrap();
        println!("Jinja2 filename result: {}", result);
        assert_eq!(result, "myapp.rs");
    }

    #[test]
    fn test_replace_filename_processor() {
        let processor = Processor::ReplaceFilename {
            pattern: "old".to_string(),
            replacement: "new".to_string(),
        };

        let result = processor.process_filename(&PathBuf::from("old_file.txt")).unwrap();
        println!("Replace filename result: {}", result);
        assert_eq!(result, "new_file.txt");
    }
}