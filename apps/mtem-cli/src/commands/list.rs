use colored::*;
use masstemplate_config::{discover_templates, load_global_config, GlobalConfig};
use serde::Serialize;

#[derive(Serialize)]
struct TemplateInfo {
    name: String,
    description: Option<String>,
    path: String,
}

#[derive(Serialize)]
struct ListOutput {
    templates: Vec<TemplateInfo>,
}

pub async fn execute(json: bool) -> anyhow::Result<()> {
    let config = load_global_config().await.unwrap_or_else(|_| GlobalConfig::default());
    let templates = discover_templates(&config).await
        .map_err(|e| anyhow::anyhow!("Failed to discover templates: {}", e))?;

    if json {
        let template_infos: Vec<TemplateInfo> = templates
            .into_iter()
            .map(|t| TemplateInfo {
                name: t.name,
                description: t.config.and_then(|c| c.description),
                path: t.path.to_string_lossy().to_string(),
            })
            .collect();

        let output = ListOutput {
            templates: template_infos,
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if templates.is_empty() {
            println!("{}", "No templates found.".yellow());
            println!("Create template directories in ~/.local/masstemplate/");
            return Ok(());
        }

        println!("{}", "Available templates:".bold());
        println!();

        for template in templates {
            let name = template.name.bold().cyan();
            if let Some(config) = template.config {
                if let Some(desc) = config.description {
                    println!("  {} - {}", name, desc);
                } else {
                    println!("  {}", name);
                }
            } else {
                println!("  {}", name);
            }
        }
    }

    Ok(())
}