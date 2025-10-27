use std::collections::HashMap;
use std::path::Path;
use minijinja::{Environment, Value};
use crate::ProcessorError;

inventory::submit! {
    crate::ProcessorMetadata {
        name: "jinja2_content",
        description: "Processes file content as Jinja2 template",
    }
}

inventory::submit! {
    crate::ProcessorMetadata {
        name: "jinja2_filename",
        description: "Processes filename as Jinja2 template",
    }
}

pub fn process_jinja2_content(
    variables: &HashMap<String, serde_json::Value>,
    _file_path: &Path,
    content: &str,
) -> Result<String, ProcessorError> {
    let env = Environment::new();

    let template = env.template_from_str(content)
        .map_err(|e| ProcessorError::ProcessingError(format!("Failed to parse Jinja2 template: {}", e)))?;

    let context = Value::from_serialize(variables);

    template.render(context)
        .map_err(|e| ProcessorError::ProcessingError(format!("Failed to render Jinja2 template: {}", e)))
}

pub fn process_jinja2_filename(
    variables: &HashMap<String, serde_json::Value>,
    file_path: &Path,
) -> Result<String, ProcessorError> {
    let filename = file_path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| ProcessorError::ProcessingError("Invalid filename".to_string()))?;

    let env = Environment::new();

    let template = env.template_from_str(filename)
        .map_err(|e| ProcessorError::ProcessingError(format!("Failed to parse Jinja2 filename template: {}", e)))?;

    let context = Value::from_serialize(variables);

    template.render(context)
        .map_err(|e| ProcessorError::ProcessingError(format!("Failed to render Jinja2 filename template: {}", e)))
}