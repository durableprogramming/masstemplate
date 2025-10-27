//! # Masstemplate DSL Parser
//!
//! This crate provides parsing functionality for the masstemplate Domain Specific Language (DSL).
//! The DSL allows users to specify file processing rules, collision strategies, and other
//! configuration options for template processing.
//!
//! ## Features
//!
//! - Parse DSL configuration files
//! - Support for collision strategies (skip, overwrite, backup, merge)
//! - File processors (dotenv, replace, template)
//! - Matcher blocks for conditional processing
//! - Priority and recursive configuration options

pub mod parser;
pub mod types;

pub use masstemplate_processors::{apply_processors, FileProcessor, Processor, ProcessorError};
pub use parser::parse_dsl;
pub use types::*;