use colored::*;
use masstemplate_config::{get_template_info, load_global_config, GlobalConfig};
use masstemplate_copier::CopierConfig;
use serde::Serialize;

#[derive(Serialize)]
struct VariableInfo {
    name: String,
    help: String,
    default: Option<String>,
}

#[derive(Serialize)]
struct InfoOutput {
    name: String,
    path: String,
    description: Option<String>,
    version: Option<String>,
    tags: Option<Vec<String>>,
    uses_copier: bool,
    variables: Vec<VariableInfo>,
}

pub async fn execute(template_name: &str, json: bool) -> anyhow::Result<()> {
    // Validate template name
    if template_name.is_empty() {
        return Err(anyhow::anyhow!("Template name cannot be empty"));
    }
    if template_name.contains('/') || template_name.contains('\\') || template_name.contains("..") {
        return Err(anyhow::anyhow!("Invalid template name: contains path separators"));
    }

    let config = load_global_config().await.unwrap_or_else(|_| GlobalConfig::default());
    let template_info = get_template_info(&config, template_name).await
        .map_err(|_| anyhow::anyhow!("Template '{}' not found. Check that it exists in ~/.local/masstemplate/", template_name))?;

    // Check for copier.yml
    let copier_path = template_info.path.join("copier.yml");
    let (uses_copier, variables) = if copier_path.exists() {
        let copier_config = CopierConfig::load(&copier_path)
            .map_err(|e| anyhow::anyhow!("Failed to load copier.yml: {}", e))?;

        let vars: Vec<VariableInfo> = copier_config.questions.iter()
            .map(|(name, question)| VariableInfo {
                name: name.clone(),
                help: question.help.as_ref().unwrap_or(name).clone(),
                default: question.default.as_ref().map(|v| v.to_string()),
            })
            .collect();

        (true, vars)
    } else {
        (false, vec![])
    };

    if json {
        let output = InfoOutput {
            name: template_name.to_string(),
            path: template_info.path.to_string_lossy().to_string(),
            description: template_info.config.as_ref().and_then(|c| c.description.clone()),
            version: template_info.config.as_ref().and_then(|c| c.version.clone()),
            tags: template_info.config.as_ref().and_then(|c| c.tags.clone()),
            uses_copier,
            variables,
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{}", format!("Template: {}", template_name).bold().cyan());
        println!("{}", format!("Path: {}", template_info.path.display()).dimmed());

        if let Some(config) = template_info.config {
            if let Some(desc) = config.description {
                println!("\n{}", desc);
            }
            if let Some(version) = config.version {
                println!("\nVersion: {}", version);
            }
            if let Some(tags) = config.tags {
                if !tags.is_empty() {
                    println!("\nTags: {}", tags.join(", "));
                }
            }
        }

        if uses_copier {
            println!("\n{}", "This template uses copier.yml configuration".green());

            if !variables.is_empty() {
                println!("\n{}:", "Variables".bold());
                for var in variables {
                    println!("  - {}: {}", var.name.bold(), var.help);
                }
            }
        }
    }

    Ok(())
}