use masstemplate_dsl_parser::{CollisionStrategy, Processor};
use masstemplate_processing_dsl::{ConfigLoader, DslProcessor};
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_temp_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_file(dir: &TempDir, path: &str, content: &str) {
        let full_path = dir.path().join(path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(full_path, content).unwrap();
    }

    #[test]
    fn test_directory_config_loading() {
        let temp_dir = create_temp_dir();
        let config_path = "project/.mtem/config";
        let config_content = r#"
            collision overwrite
            recursive false
            priority 5
            dotenv set API_KEY=test_key
        "#;

        write_file(&temp_dir, config_path, config_content);
        write_file(&temp_dir, "project/test.txt", "content");

        let mut loader = ConfigLoader::new();
        let merged = loader.load_for_file(&temp_dir.path().join("project/test.txt")).unwrap();

        assert_eq!(merged.collision_strategy, Some(CollisionStrategy::Overwrite));
        assert_eq!(merged.processors.len(), 1);
        match &merged.processors[0] {
            Processor::DotenvSet { key, value } => {
                assert_eq!(key, "API_KEY");
                assert_eq!(value, "test_key");
            }
            _ => panic!("Expected DotenvSet processor"),
        }
    }

    #[test]
    fn test_file_config_loading() {
        let temp_dir = create_temp_dir();
        let file_path = "project/src/main.rs";
        let config_path = "project/src/main.rs.mtem.config";
        let config_content = r#"
            collision merge
            priority 10
            replace Hello Hi
        "#;

        write_file(&temp_dir, file_path, "fn main() { println!(\"Hello\"); }");
        write_file(&temp_dir, config_path, config_content);

        let mut loader = ConfigLoader::new();
        let merged = loader.load_for_file(&temp_dir.path().join(file_path)).unwrap();

        assert_eq!(merged.collision_strategy, Some(CollisionStrategy::Merge));
        assert_eq!(merged.processors.len(), 1);
        match &merged.processors[0] {
            Processor::Replace { pattern, replacement } => {
                assert_eq!(pattern, "Hello");
                assert_eq!(replacement, "Hi");
            }
            _ => panic!("Expected Replace processor"),
        }
    }

    #[test]
    fn test_config_inheritance() {
        let temp_dir = create_temp_dir();

        // Root config
        write_file(&temp_dir, ".mtem/config", r#"
            collision skip
            dotenv set ROOT_VAR=root_value
            priority 1
        "#);

        // Subdir config
        write_file(&temp_dir, "src/.mtem/config", r#"
            collision overwrite
            dotenv set SUB_VAR=sub_value
            priority 2
        "#);

        // File config
        write_file(&temp_dir, "src/main.rs", "content");
        write_file(&temp_dir, "src/main.rs.mtem.config", r#"
            collision merge
            dotenv set FILE_VAR=file_value
            priority 3
        "#);

        let mut loader = ConfigLoader::new();
        let merged = loader.load_for_file(&temp_dir.path().join("src/main.rs")).unwrap();

        // File config should override directory configs
        assert_eq!(merged.collision_strategy, Some(CollisionStrategy::Merge));

        // Should have processors from all levels
        assert_eq!(merged.processors.len(), 3);

        // Check that all processors are present
        let mut has_root = false;
        let mut has_sub = false;
        let mut has_file = false;

        for processor in &merged.processors {
            match processor {
                Processor::DotenvSet { key, value } => match key.as_str() {
                    "ROOT_VAR" => {
                        assert_eq!(value, "root_value");
                        has_root = true;
                    }
                    "SUB_VAR" => {
                        assert_eq!(value, "sub_value");
                        has_sub = true;
                    }
                    "FILE_VAR" => {
                        assert_eq!(value, "file_value");
                        has_file = true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        assert!(has_root);
        assert!(has_sub);
        assert!(has_file);
    }

    #[test]
    fn test_non_recursive_config() {
        let temp_dir = create_temp_dir();

        // Root config (recursive)
        write_file(&temp_dir, ".mtem/config", r#"
            collision skip
            dotenv set ROOT_VAR=root_value
            recursive true
        "#);

        // Subdir config (non-recursive)
        write_file(&temp_dir, "src/.mtem/config", r#"
            collision overwrite
            recursive false
        "#);

        // Deep subdir - should not inherit root config
        write_file(&temp_dir, "src/deep/file.txt", "content");

        let mut loader = ConfigLoader::new();
        let merged = loader.load_for_file(&temp_dir.path().join("src/deep/file.txt")).unwrap();

        // Should only have the subdir config collision strategy
        assert_eq!(merged.collision_strategy, Some(CollisionStrategy::Overwrite));
        // Should not have ROOT_VAR processor
        assert!(!merged.processors.iter().any(|p| matches!(p, Processor::DotenvSet { key, .. } if key == "ROOT_VAR")));
    }

    #[test]
    fn test_file_processing() {
        let temp_dir = create_temp_dir();
        let env_file = "project/.env";
        let template_file = "project/template.txt";

        write_file(&temp_dir, env_file, "EXISTING=value\n");
        write_file(&temp_dir, template_file, "Hello {{NAME}}!");

        // Create config for .env file
        write_file(&temp_dir, "project/.env.mtem.config", r#"
            dotenv set NEW_KEY=new_value
            dotenv append EXISTING=_appended
        "#);

        // Create config for template file
        write_file(&temp_dir, "project/template.txt.mtem.config", r#"
            template NAME=World
        "#);

        let mut processor = DslProcessor::new();
        processor.process_file(&temp_dir.path().join(env_file)).unwrap();
        processor.process_file(&temp_dir.path().join(template_file)).unwrap();

        let env_content = fs::read_to_string(temp_dir.path().join(env_file)).unwrap();
        assert!(env_content.contains("NEW_KEY=new_value"));
        assert!(env_content.contains("EXISTING=value_appended"));

        let template_content = fs::read_to_string(temp_dir.path().join(template_file)).unwrap();
        assert_eq!(template_content, "Hello World!");
    }

    #[test]
    fn test_directory_processing() {
        let temp_dir = create_temp_dir();

        // Create directory structure
        write_file(&temp_dir, ".mtem/config", "replace template_var actual_value");
        write_file(&temp_dir, "file1.txt", "This is template_var");
        write_file(&temp_dir, "subdir/file2.txt", "Also template_var");

        let mut processor = DslProcessor::new();
        processor.process_directory(&temp_dir.path().to_path_buf()).unwrap();

        let content1 = fs::read_to_string(temp_dir.path().join("file1.txt")).unwrap();
        let content2 = fs::read_to_string(temp_dir.path().join("subdir/file2.txt")).unwrap();

        assert_eq!(content1, "This is actual_value");
        assert_eq!(content2, "Also actual_value");
    }

    #[test]
    fn test_collision_strategy_retrieval() {
        let temp_dir = create_temp_dir();

        write_file(&temp_dir, ".mtem/config", "collision backup");
        write_file(&temp_dir, "test.txt", "content");

        let mut processor = DslProcessor::new();
        let strategy = processor.get_collision_strategy(&temp_dir.path().join("test.txt")).unwrap();

        assert_eq!(strategy, Some(CollisionStrategy::Backup));
    }

    #[test]
    fn test_matcher_processing() {
        let temp_dir = create_temp_dir();

        // Create config with matcher
        write_file(&temp_dir, ".mtem/config", r#"
            match *.txt {
                replace old_text new_text
            }
            match *.md {
                template TITLE=Document
            }
        "#);

        // Create files
        write_file(&temp_dir, "file1.txt", "This is old_text");
        write_file(&temp_dir, "file2.txt", "Also old_text");
        write_file(&temp_dir, "file.md", "Title: {{TITLE}}");
        write_file(&temp_dir, "file.rs", "This is old_text"); // Should not be processed

        let mut processor = DslProcessor::new();
        processor.process_directory(&temp_dir.path().to_path_buf()).unwrap();

        let txt1_content = fs::read_to_string(temp_dir.path().join("file1.txt")).unwrap();
        let txt2_content = fs::read_to_string(temp_dir.path().join("file2.txt")).unwrap();
        let md_content = fs::read_to_string(temp_dir.path().join("file.md")).unwrap();
        let rs_content = fs::read_to_string(temp_dir.path().join("file.rs")).unwrap();

        assert_eq!(txt1_content, "This is new_text");
        assert_eq!(txt2_content, "Also new_text");
        assert_eq!(md_content, "Title: Document");
        assert_eq!(rs_content, "This is old_text"); // Unchanged
    }

    #[test]
    fn test_invalid_config_error() {
        let temp_dir = create_temp_dir();
        let config_path = "project/.mtem/config";
        let invalid_config = "collision invalid_strategy";

        write_file(&temp_dir, config_path, invalid_config);
        write_file(&temp_dir, "project/test.txt", "content");

        let mut loader = ConfigLoader::new();
        let result = loader.load_for_file(&temp_dir.path().join("project/test.txt"));

        assert!(result.is_err());
    }

    #[test]
    fn test_file_config_with_matchers() {
        let temp_dir = create_temp_dir();
        let file_path = "project/src/main.rs";
        let config_path = "project/src/main.rs.mtem.config";
        let config_content = r#"
            match *.rs {
                replace println! hello
            }
        "#;

        write_file(&temp_dir, file_path, "fn main() { println!(\"Hello\"); }");
        write_file(&temp_dir, config_path, config_content);

        let mut processor = DslProcessor::new();
        processor.process_file(&temp_dir.path().join(file_path)).unwrap();

        let content = fs::read_to_string(temp_dir.path().join(file_path)).unwrap();
        assert_eq!(content, "fn main() { hello(\"Hello\"); }");
    }

    #[test]
    fn test_priority_ordering() {
        let temp_dir = create_temp_dir();

        // Low priority config
        write_file(&temp_dir, ".mtem/config", r#"
            collision skip
            priority 1
        "#);

        // High priority config
        write_file(&temp_dir, "src/.mtem/config", r#"
            collision overwrite
            priority 10
        "#);

        write_file(&temp_dir, "src/test.txt", "content");

        let mut loader = ConfigLoader::new();
        let merged = loader.load_for_file(&temp_dir.path().join("src/test.txt")).unwrap();

        // Higher priority should override
        assert_eq!(merged.collision_strategy, Some(CollisionStrategy::Overwrite));
    }
}