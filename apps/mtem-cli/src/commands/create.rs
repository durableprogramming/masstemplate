use std::path::PathBuf;

pub async fn execute(
    template_name: &str,
    path: PathBuf,
    collision_strategy: Option<String>,
    non_interactive: bool,
    data: Vec<String>,
    json: bool,
) -> anyhow::Result<()> {
    // Check if path already exists
    if path.exists() {
        return Err(anyhow::anyhow!(
            "Path '{}' already exists. Use 'mtem apply' to apply a template to an existing directory.",
            path.display()
        ));
    }

    // Create the directory
    tokio::fs::create_dir_all(&path).await
        .map_err(|e| anyhow::anyhow!("Failed to create directory '{}': {}", path.display(), e))?;

    // Apply the template using the existing apply command
    super::apply::execute(template_name, Some(path), collision_strategy, non_interactive, data, json).await
}
