use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Processor {
    DotenvSet { key: String, value: String },
    DotenvAppend { key: String, value: String },
    Replace { pattern: String, replacement: String },
    Template { variables: HashMap<String, String> },
    Jinja2Content { variables: HashMap<String, serde_json::Value> },
    Jinja2Filename { variables: HashMap<String, serde_json::Value> },
    ReplaceFilename { pattern: String, replacement: String },
}