use colored::*;
use masstemplate_config::{get_template_info, load_global_config, GlobalConfig};
use masstemplate_copier::{CopierConfig, TaskRunner, VariablePrompter};
use masstemplate_fileops::CollisionStrategy;
use masstemplate_hooks::HookManager;
use minijinja::{Environment, Value};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Serialize)]
struct ApplyOutput {
    success: bool,
    template: String,
    destination: String,
    error: Option<String>,
}

pub async fn execute(
    template_name: &str,
    dest: Option<PathBuf>,
    collision_strategy: Option<String>,
    non_interactive: bool,
    data: Vec<String>,
    json: bool,
) -> anyhow::Result<()> {
    // Validate template name
    if template_name.is_empty() {
        return Err(anyhow::anyhow!("Template name cannot be empty"));
    }

    // Allow github: prefix, otherwise validate as local template name
    if !template_name.starts_with("github:") {
        if template_name.contains('/') || template_name.contains('\\') || template_name.contains("..") {
            return Err(anyhow::anyhow!("Invalid template name: contains path separators"));
        }
    }

    // Validate destination if provided
    if let Some(ref dest_path) = dest {
        if dest_path.is_absolute() && !dest_path.starts_with("/") {
            return Err(anyhow::anyhow!("Invalid destination path"));
        }
    }

    let config = load_global_config().await.unwrap_or_else(|_| GlobalConfig::default());
    let template_info = get_template_info(&config, template_name).await
        .map_err(|e| anyhow::anyhow!("Template '{}' not found: {}", template_name, e))?;
    let template_path = template_info.path;

    let dest_dir = dest.unwrap_or_else(|| std::env::current_dir().unwrap());
    // Ensure dest_dir is absolute for proper command execution
    let dest_dir = dest_dir.canonicalize()
        .unwrap_or_else(|_| {
            // If canonicalize fails (e.g., path doesn't exist yet), make it absolute
            if dest_dir.is_absolute() {
                dest_dir
            } else {
                std::env::current_dir().unwrap().join(dest_dir)
            }
        });

    println!(
        "{}",
        format!("Applying template '{}' to {}", template_name, dest_dir.display())
            .bold()
            .green()
    );

    // Check for copier.yml
    let copier_path = template_path.join("copier.yml");
    let variables = if copier_path.exists() {
        println!("\n{}", "Template uses copier.yml, prompting for variables...".cyan());
        let copier_config = CopierConfig::load(&copier_path)
            .map_err(|e| anyhow::anyhow!("Failed to load copier.yml configuration: {}", e))?;

        let mut prompter = VariablePrompter::new(copier_config.clone());
        prompter.set_non_interactive(non_interactive);

        // Automatically set project_name based on destination directory
        if let Some(dir_name) = dest_dir.file_name() {
            if let Some(name_str) = dir_name.to_str() {
                prompter.set_default("project_name".to_string(), Value::from(name_str));
            }
        }

        // Parse and set data from CLI arguments
        for item in &data {
            if let Some((key, value)) = item.split_once('=') {
                prompter.set_default(key.to_string(), Value::from(value));
            } else {
                return Err(anyhow::anyhow!("Invalid data format '{}'. Expected key=value", item));
            }
        }

        let vars = prompter.prompt_all()?;
        println!();
        Some((copier_config, vars))
    } else {
        None
    };

    // Load hooks
    let hook_manager = HookManager::load_from_template(&template_path).await?;

    // Execute pre-copy hooks
    if hook_manager.has_hooks() {
        println!("{}", "Executing pre-copy hooks...".cyan());
        let collision = collision_strategy
            .as_ref()
            .and_then(|s| CollisionStrategy::from_str(s).ok())
            .unwrap_or(CollisionStrategy::Skip);

        let hook_context = masstemplate_hooks::HookContext::new(
            template_name.to_string(),
            template_path.clone(),
            dest_dir.clone(),
            collision,
        );

        hook_manager.execute_pre_copy_hooks(&hook_context).await?;
    }

    // Copy files and process
    println!("{}", "Copying and processing files...".cyan());

    if let Some((copier_config, vars)) = variables {
        // Process with Jinja2 if we have variables
        apply_copier_template(&template_path, &dest_dir, &copier_config, &vars).await?;
        println!("{}", "✓ Files copied successfully".green());

        // Run post-generation tasks
        if let Some(tasks) = copier_config.tasks {
            println!("\n{}", "Running post-generation tasks...".cyan());
            let task_runner = TaskRunner::new(dest_dir.clone());
            task_runner.run_tasks(&tasks).await?;
        }
    } else {
        // Regular template application - use template_path directly since we already resolved it
        use masstemplate_core::{TemplateFileCopier, ScriptRunner};
        use masstemplate_fileops::CollisionStrategy;

        let collision = collision_strategy
            .as_ref()
            .and_then(|s| CollisionStrategy::from_str(s).ok())
            .unwrap_or(CollisionStrategy::Skip);

        // Read ignore patterns
        let ignore_patterns = read_ignore_patterns(&template_path);

        // Run pre-install script
        ScriptRunner::run_pre_install_script(&template_path, &dest_dir).await?;

        // Copy files with processing
        let mut file_copier = TemplateFileCopier::new();
        file_copier.copy_template_files_with_strategy(&template_path, &dest_dir, &ignore_patterns, collision).await?;

        // Run post-install script
        ScriptRunner::run_post_install_script(&template_path, &dest_dir).await?;
    }

    // Execute post-copy hooks
    if hook_manager.has_hooks() {
        println!("{}", "Executing post-copy hooks...".cyan());
        let collision = collision_strategy
            .and_then(|s| CollisionStrategy::from_str(&s).ok())
            .unwrap_or(CollisionStrategy::Skip);

        let hook_context = masstemplate_hooks::HookContext::new(
            template_name.to_string(),
            template_path,
            dest_dir.clone(),
            collision,
        );

        hook_manager.execute_post_copy_hooks(&hook_context).await?;
    }

    if json {
        let output = ApplyOutput {
            success: true,
            template: template_name.to_string(),
            destination: dest_dir.to_string_lossy().to_string(),
            error: None,
        };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!();
        println!("{}", format!("✓ Template '{}' applied successfully!", template_name).bold().green());
    }

    Ok(())
}

async fn apply_copier_template(
    template_path: &Path,
    dest_dir: &Path,
    config: &CopierConfig,
    variables: &std::collections::HashMap<String, Value>,
) -> anyhow::Result<()> {
    use masstemplate_copier::FileFilter;
    use walkdir::WalkDir;

    let filter = FileFilter::new(
        config.skip_if_exists.clone().unwrap_or_default(),
        config.templates_suffix.clone(),
    );

    let mut env = Environment::new();

    // Apply custom environment options from copier config
    if let Some(ref envops) = config.envops {
        use minijinja::Syntax;
        use std::borrow::Cow;

        let syntax = Syntax {
            block_start: Cow::Owned(envops.block_start_string.clone().unwrap_or_else(|| "{%".into())),
            block_end: Cow::Owned(envops.block_end_string.clone().unwrap_or_else(|| "%}".into())),
            variable_start: Cow::Owned(envops.variable_start_string.clone().unwrap_or_else(|| "{{".into())),
            variable_end: Cow::Owned(envops.variable_end_string.clone().unwrap_or_else(|| "}}".into())),
            comment_start: Cow::Owned(envops.comment_start_string.clone().unwrap_or_else(|| "{#".into())),
            comment_end: Cow::Owned(envops.comment_end_string.clone().unwrap_or_else(|| "#}".into())),
        };

        env.set_syntax(syntax)
            .map_err(|e| anyhow::anyhow!("Failed to configure Jinja2 environment syntax: {}", e))?;
    }

    for entry in WalkDir::new(template_path) {
        let entry = entry?;
        let path = entry.path();

        // Skip .mtem and copier.yml
        if path.components().any(|c| c.as_os_str() == ".mtem") {
            continue;
        }
        if path.file_name().is_some_and(|n| n == "copier.yml") {
            continue;
        }

        if path.is_file() {
            let relative = path.strip_prefix(template_path)?;

            if filter.should_skip(relative) {
                continue;
            }

            // Process filename with Jinja2
            let mut dest_relative = PathBuf::new();
            for component in relative.components() {
                let comp_str = component.as_os_str().to_string_lossy();
                let template = env.template_from_str(&comp_str)
                    .map_err(|e| anyhow::anyhow!(
                        "Failed to parse filename template for '{}': {}",
                        relative.display(),
                        e
                    ))?;
                let ctx = Value::from_serialize(variables);
                let rendered = template.render(ctx)
                    .map_err(|e| anyhow::anyhow!(
                        "Failed to render filename template for '{}': {}",
                        relative.display(),
                        e
                    ))?;
                dest_relative.push(rendered);
            }

            let dest_file = dest_dir.join(&dest_relative);

            // Create parent directories
            if let Some(parent) = dest_file.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            // Try to read as text; if it fails (binary file), copy directly
            match tokio::fs::read_to_string(path).await {
                Ok(content) => {
                    // Process file content with Jinja2
                    let template = env.template_from_str(&content)
                        .map_err(|e| anyhow::anyhow!(
                            "Failed to parse template in file '{}': {}",
                            relative.display(),
                            e
                        ))?;
                    let ctx = Value::from_serialize(variables);
                    let rendered = template.render(ctx)
                        .map_err(|e| anyhow::anyhow!(
                            "Failed to render template in file '{}': {}",
                            relative.display(),
                            e
                        ))?;

                    tokio::fs::write(&dest_file, rendered).await
                        .map_err(|e| anyhow::anyhow!(
                            "Failed to write file '{}': {}",
                            dest_relative.display(),
                            e
                        ))?;
                }
                Err(_) => {
                    // Binary file - copy directly without processing
                    tokio::fs::copy(path, &dest_file).await
                        .map_err(|e| anyhow::anyhow!(
                            "Failed to copy binary file '{}': {}",
                            dest_relative.display(),
                            e
                        ))?;
                }
            }
        }
    }

    Ok(())
}

/// Read ignore patterns from .mtemignore or .mtem/ignore
fn read_ignore_patterns(template_dir: &Path) -> Vec<String> {
    let mut patterns = Vec::new();

    // Always ignore .mtem directory and .mtemignore file
    patterns.push(".mtem".to_string());
    patterns.push(".mtemignore".to_string());

    // Try .mtemignore first
    let mtemignore_path = template_dir.join(".mtemignore");
    if mtemignore_path.exists() {
        if let Ok(content) = fs::read_to_string(&mtemignore_path) {
            patterns.extend(
                content
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .map(|l| l.to_string())
            );
            return patterns;
        }
    }

    // Try .mtem/ignore
    let mtem_ignore_path = template_dir.join(".mtem").join("ignore");
    if mtem_ignore_path.exists() {
        if let Ok(content) = fs::read_to_string(&mtem_ignore_path) {
            patterns.extend(
                content
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .map(|l| l.to_string())
            );
        }
    }

    patterns
}