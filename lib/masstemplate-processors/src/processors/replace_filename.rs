use std::path::Path;
use crate::ProcessorError;

inventory::submit! {
    crate::ProcessorMetadata {
        name: "replace_filename",
        description: "Replaces patterns in filenames",
    }
}

pub fn process_replace_filename(
    pattern: &str,
    replacement: &str,
    file_path: &Path,
) -> Result<String, ProcessorError> {
    let filename = file_path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| ProcessorError::ProcessingError("Invalid filename".to_string()))?;

    Ok(filename.replace(pattern, replacement))
}