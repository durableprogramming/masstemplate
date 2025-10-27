use std::path::Path;
use crate::ProcessorError;

inventory::submit! {
    crate::ProcessorMetadata {
        name: "dotenv_append",
        description: "Appends to environment variable in .env files",
    }
}

pub fn process_dotenv_append(key: &str, value: &str, file_path: &Path, content: &str) -> Result<String, ProcessorError> {
    if file_path.ends_with(".env") {
        append_env_var(content, key, value)
    } else {
        Ok(content.to_string())
    }
}

pub(crate) fn append_env_var(content: &str, key: &str, value: &str) -> Result<String, ProcessorError> {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Find existing line with this key
    for line in &mut lines {
        if let Some((existing_key, existing_value)) = line.split_once('=') {
            if existing_key.trim() == key {
                let trimmed_value = existing_value.trim();
                let new_value = format!("{}{}", trimmed_value, value);
                *line = format!("{}={}", existing_key, new_value);
                return Ok(lines.join("\n"));
            }
        }
    }

    // Key not found, add it
    lines.push(format!("{}={}", key, value));
    Ok(lines.join("\n"))
}