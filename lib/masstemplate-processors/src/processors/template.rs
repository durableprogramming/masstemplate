use std::collections::HashMap;
use crate::ProcessorError;

inventory::submit! {
    crate::ProcessorMetadata {
        name: "template",
        description: "Replaces template variables in files",
    }
}

pub fn process_template(variables: &HashMap<String, String>, _file_path: &std::path::Path, content: &str) -> Result<String, ProcessorError> {
    let mut result = content.to_string();
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    Ok(result)
}