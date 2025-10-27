use std::path::Path;
use crate::{Processor, ProcessorError, processors};

pub trait FileProcessor {
    fn process_content(&self, file_path: &Path, content: &str) -> Result<String, ProcessorError>;
    fn process_filename(&self, file_path: &Path) -> Result<String, ProcessorError>;
}

impl FileProcessor for Processor {
    fn process_content(&self, file_path: &Path, content: &str) -> Result<String, ProcessorError> {
        match self {
            Processor::DotenvSet { key, value } => {
                processors::dotenv_set::process_dotenv_set(key, value, file_path, content)
            }
            Processor::DotenvAppend { key, value } => {
                processors::dotenv_append::process_dotenv_append(key, value, file_path, content)
            }
            Processor::Replace { pattern, replacement } => {
                processors::replace::process_replace(pattern, replacement, file_path, content)
            }
            Processor::Template { variables } => {
                processors::template::process_template(variables, file_path, content)
            }
            Processor::Jinja2Content { variables } => {
                processors::jinja2::process_jinja2_content(variables, file_path, content)
            }
            Processor::Jinja2Filename { .. } => Ok(content.to_string()), // Filename processors don't modify content
            Processor::ReplaceFilename { .. } => Ok(content.to_string()), // Filename processors don't modify content
        }
    }

    fn process_filename(&self, file_path: &Path) -> Result<String, ProcessorError> {
        match self {
            Processor::DotenvSet { .. } | Processor::DotenvAppend { .. } | Processor::Replace { .. } | Processor::Template { .. } | Processor::Jinja2Content { .. } => {
                // Content processors don't modify filename
                Ok(file_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string())
            }
            Processor::Jinja2Filename { variables } => {
                processors::jinja2::process_jinja2_filename(variables, file_path)
            }
            Processor::ReplaceFilename { pattern, replacement } => {
                processors::replace_filename::process_replace_filename(pattern, replacement, file_path)
            }
        }
    }
}