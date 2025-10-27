use std::path::Path;
use crate::ProcessorError;

inventory::submit! {
    crate::ProcessorMetadata {
        name: "dotenv_set",
        description: "Sets environment variable in .env files",
    }
}

pub fn process_dotenv_set(key: &str, value: &str, file_path: &Path, content: &str) -> Result<String, ProcessorError> {
    if file_path.ends_with(".env") {
        set_env_var(content, key, value)
    } else {
        Ok(content.to_string())
    }
}

pub(crate) fn set_env_var(content: &str, key: &str, value: &str) -> Result<String, ProcessorError> {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let target_line = format!("{}={}", key, value);

    // Remove any existing lines with this key
    lines.retain(|line| {
        if let Some((existing_key, _)) = line.split_once('=') {
            existing_key.trim() != key
        } else {
            true
        }
    });

    // Add the new line
    lines.push(target_line);

    Ok(lines.join("\n"))
}