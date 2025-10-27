use crate::ProcessorError;

inventory::submit! {
    crate::ProcessorMetadata {
        name: "replace",
        description: "Replaces text patterns in files",
    }
}

pub fn process_replace(pattern: &str, replacement: &str, _file_path: &std::path::Path, content: &str) -> Result<String, ProcessorError> {
    Ok(content.replace(pattern, replacement))
}