use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use masstemplate_core::{TemplateApplicator, CoreError};
use masstemplate_config::GlobalConfig;
use masstemplate_fileops::CollisionStrategy;

/// Helper function to create a temporary template directory structure
fn create_test_template(temp_dir: &TempDir, template_name: &str) -> PathBuf {
    let template_dir = temp_dir.path().join("templates").join(template_name);
    fs::create_dir_all(&template_dir).unwrap();

    // Create some template files
    fs::write(template_dir.join("README.md"), "# My Template\n\nThis is a test template.").unwrap();
    fs::write(template_dir.join("config.txt"), "key=value").unwrap();

    // Create a subdirectory
    let sub_dir = template_dir.join("src");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(sub_dir.join("main.rs"), "fn main() { println!(\"Hello!\"); }").unwrap();

    template_dir
}

/// Helper function to create a global config pointing to our test template directory
fn create_test_config(temp_dir: &TempDir) -> GlobalConfig {
    GlobalConfig {
        template_directory: Some(temp_dir.path().join("templates").to_string_lossy().to_string()),
        default_collision_strategy: Some("skip".to_string()),
        verbose: Some(false),
        template_sources: Some(Vec::new()),
    }
}

#[tokio::test]
async fn test_apply_template_basic() {
    let temp_dir = TempDir::new().unwrap();
    let _template_dir = create_test_template(&temp_dir, "basic-template");
    let config = create_test_config(&temp_dir);

    // Create destination directory
    let dest_dir = temp_dir.path().join("project");
    fs::create_dir_all(&dest_dir).unwrap();
    env::set_current_dir(&dest_dir).unwrap();

    // Apply template
    let applicator = TemplateApplicator::new(config);
    let result = applicator.apply_template_with_strategy("basic-template", &dest_dir, CollisionStrategy::Skip).await;
    assert!(result.is_ok());

    // Verify files were copied
    assert!(dest_dir.join("README.md").exists());
    assert!(dest_dir.join("config.txt").exists());
    assert!(dest_dir.join("src").join("main.rs").exists());

    // Verify content
    let readme_content = fs::read_to_string(dest_dir.join("README.md")).unwrap();
    assert_eq!(readme_content, "# My Template\n\nThis is a test template.");
}

#[tokio::test]
async fn test_apply_template_with_pre_install_script() {
    let temp_dir = TempDir::new().unwrap();
    let template_dir = create_test_template(&temp_dir, "script-template");
    let config = create_test_config(&temp_dir);

    // Create .mtem directory and pre-install script
    let mtem_dir = template_dir.join(".mtem");
    fs::create_dir_all(&mtem_dir).unwrap();
    fs::write(mtem_dir.join("pre_install.sh"), "#!/bin/sh\necho 'pre-install executed' > pre_install_marker.txt\n").unwrap();

    // Create destination directory
    let dest_dir = temp_dir.path().join("project");
    fs::create_dir_all(&dest_dir).unwrap();
    env::set_current_dir(&dest_dir).unwrap();

    // Apply template
    let applicator = TemplateApplicator::new(config);
    let result = applicator.apply_template_with_strategy("script-template", &dest_dir, CollisionStrategy::Skip).await;
    assert!(result.is_ok());

    // Verify pre-install script was executed
    assert!(dest_dir.join("pre_install_marker.txt").exists());
    let marker_content = fs::read_to_string(dest_dir.join("pre_install_marker.txt")).unwrap();
    assert_eq!(marker_content.trim(), "pre-install executed");
}

#[tokio::test]
async fn test_apply_template_with_post_install_script() {
    let temp_dir = TempDir::new().unwrap();
    let template_dir = create_test_template(&temp_dir, "post-script-template");
    let config = create_test_config(&temp_dir);

    // Create .mtem directory and post-install script
    let mtem_dir = template_dir.join(".mtem");
    fs::create_dir_all(&mtem_dir).unwrap();
    fs::write(mtem_dir.join("post_install.sh"), "#!/bin/sh\necho 'post-install executed' > post_install_marker.txt\n").unwrap();

    // Create destination directory
    let dest_dir = temp_dir.path().join("project");
    fs::create_dir_all(&dest_dir).unwrap();
    env::set_current_dir(&dest_dir).unwrap();

    // Apply template
    let applicator = TemplateApplicator::new(config);
    let result = applicator.apply_template_with_strategy("post-script-template", &dest_dir, CollisionStrategy::Skip).await;
    assert!(result.is_ok());

    // Verify post-install script was executed
    assert!(dest_dir.join("post_install_marker.txt").exists());
    let marker_content = fs::read_to_string(dest_dir.join("post_install_marker.txt")).unwrap();
    assert_eq!(marker_content.trim(), "post-install executed");
}

#[tokio::test]
async fn test_apply_template_with_ignore_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let template_dir = create_test_template(&temp_dir, "ignore-template");
    let config = create_test_config(&temp_dir);

    // Create files that should be ignored
    fs::write(template_dir.join("temp.log"), "log content").unwrap();
    fs::write(template_dir.join("cache.tmp"), "cache content").unwrap();

    // Create .mtemignore file
    fs::write(template_dir.join(".mtemignore"), "*.log\n*.tmp\n").unwrap();

    // Create destination directory
    let dest_dir = temp_dir.path().join("project");
    fs::create_dir_all(&dest_dir).unwrap();
    env::set_current_dir(&dest_dir).unwrap();

    // Apply template
    let applicator = TemplateApplicator::new(config);
    let result = applicator.apply_template_with_strategy("ignore-template", &dest_dir, CollisionStrategy::Skip).await;
    assert!(result.is_ok());

    // Verify ignored files were not copied
    assert!(!dest_dir.join("temp.log").exists());
    assert!(!dest_dir.join("cache.tmp").exists());

    // Verify non-ignored files were copied
    assert!(dest_dir.join("README.md").exists());
    assert!(dest_dir.join("config.txt").exists());
}

#[tokio::test]
async fn test_apply_template_collision_skip() {
    let temp_dir = TempDir::new().unwrap();
    let _template_dir = create_test_template(&temp_dir, "collision-skip-template");
    let config = create_test_config(&temp_dir);

    // Create destination directory with existing file
    let dest_dir = temp_dir.path().join("project");
    fs::create_dir_all(&dest_dir).unwrap();
    fs::write(dest_dir.join("README.md"), "existing content").unwrap();
    env::set_current_dir(&dest_dir).unwrap();

    // Apply template with skip strategy
    let applicator = TemplateApplicator::new(config);
    let result = applicator.apply_template_with_strategy("collision-skip-template", &dest_dir, CollisionStrategy::Skip).await;
    assert!(result.is_ok());

    // Verify existing file was not overwritten
    let content = fs::read_to_string(dest_dir.join("README.md")).unwrap();
    assert_eq!(content, "existing content");

    // Verify new files were still copied
    assert!(dest_dir.join("config.txt").exists());
}

#[tokio::test]
async fn test_apply_template_collision_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let _template_dir = create_test_template(&temp_dir, "collision-overwrite-template");
    let config = create_test_config(&temp_dir);

    // Create destination directory with existing file
    let dest_dir = temp_dir.path().join("project");
    fs::create_dir_all(&dest_dir).unwrap();
    fs::write(dest_dir.join("README.md"), "existing content").unwrap();
    env::set_current_dir(&dest_dir).unwrap();

    // Apply template with overwrite strategy
    let applicator = TemplateApplicator::new(config);
    let result = applicator.apply_template_with_strategy("collision-overwrite-template", &dest_dir, CollisionStrategy::Overwrite).await;
    assert!(result.is_ok());

    // Verify existing file was overwritten
    let content = fs::read_to_string(dest_dir.join("README.md")).unwrap();
    assert_eq!(content, "# My Template\n\nThis is a test template.");
}

#[tokio::test]
async fn test_apply_template_nonexistent_template() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);

    // Create destination directory
    let dest_dir = temp_dir.path().join("project");
    fs::create_dir_all(&dest_dir).unwrap();
    env::set_current_dir(&dest_dir).unwrap();

    // Try to apply nonexistent template
    let applicator = TemplateApplicator::new(config);
    let result = applicator.apply_template_with_strategy("nonexistent-template", &dest_dir, CollisionStrategy::Skip).await;
    assert!(result.is_err());

    // Verify it's the right kind of error
    match result {
        Err(CoreError::TemplateNotFound(name)) => {
            assert_eq!(name, "nonexistent-template");
        }
        _ => panic!("Expected TemplateNotFound error"),
    }
}

#[tokio::test]
async fn test_script_failure() {
    let temp_dir = TempDir::new().unwrap();
    let template_dir = create_test_template(&temp_dir, "failing-script-template");
    let config = create_test_config(&temp_dir);

    // Create .mtem directory and failing script
    let mtem_dir = template_dir.join(".mtem");
    fs::create_dir_all(&mtem_dir).unwrap();
    fs::write(mtem_dir.join("pre_install.sh"), "#!/bin/sh\nexit 1\n").unwrap();

    // Create destination directory
    let dest_dir = temp_dir.path().join("project");
    fs::create_dir_all(&dest_dir).unwrap();
    env::set_current_dir(&dest_dir).unwrap();

    // Try to apply template - should fail due to script
    let applicator = TemplateApplicator::new(config);
    let result = applicator.apply_template_with_strategy("failing-script-template", &dest_dir, CollisionStrategy::Skip).await;
    assert!(result.is_err());

    // Verify it's a script execution error
    match result {
        Err(CoreError::ScriptExecutionFailed(_)) => {
            // Expected error
        }
        _ => panic!("Expected ScriptExecutionFailed error"),
    }
}