use crate::{CopierConfig, CopierError, Question, QuestionType, Result};
use dialoguer::{Input, Confirm};
use minijinja::{Environment, Value};
use serde_json;
use std::collections::HashMap;

pub struct VariablePrompter {
    config: CopierConfig,
    non_interactive: bool,
    defaults: HashMap<String, Value>,
}

impl VariablePrompter {
    pub fn new(config: CopierConfig) -> Self {
        Self {
            config,
            non_interactive: false,
            defaults: HashMap::new(),
        }
    }

    pub fn set_non_interactive(&mut self, non_interactive: bool) {
        self.non_interactive = non_interactive;
    }

    pub fn set_default(&mut self, name: String, value: Value) {
        self.defaults.insert(name, value);
    }

    /// Prompt user for all variables
    pub fn prompt_all(&self) -> Result<HashMap<String, Value>> {
        let mut answers = HashMap::new();

        // Add preset defaults first
        for (name, value) in &self.defaults {
            answers.insert(name.clone(), value.clone());
        }

        // Sort questions to ensure consistent order
        let mut question_names: Vec<_> = self.config.questions.keys().collect();
        question_names.sort();

        for name in question_names {
            // Skip if we already have a preset default
            if self.defaults.contains_key(name) {
                continue;
            }

            let question = &self.config.questions[name];

            // Check if we should prompt for this variable
            if !self.should_prompt(question, &answers)? {
                continue;
            }

            let value = self.prompt_variable(name, question, &answers)?;
            answers.insert(name.clone(), value);
        }

        Ok(answers)
    }

    /// Prompt for single variable
    fn prompt_variable(
        &self,
        name: &str,
        question: &Question,
        context: &HashMap<String, Value>,
    ) -> Result<Value> {
        // Evaluate default value
        let default_value = if let Some(ref default_json) = question.default {
            self.evaluate_default(default_json, context)
                .map_err(|e| CopierError::Template(format!("Error in variable '{}': {}", name, e)))?
        } else {
            Value::from(())  // None/null
        };

        // In non-interactive mode, use defaults
        if self.non_interactive {
            return Ok(default_value);
        }

        let help_text = question.help.as_deref().unwrap_or(name);

        let value = match question.question_type {
            QuestionType::Str => {
                let result = if let Some(s) = default_value.as_str() {
                    Input::<String>::new()
                        .with_prompt(help_text)
                        .default(s.to_string())
                        .interact_text()
                        .map_err(|e| CopierError::PromptError(format!("Error prompting for '{}': {}", name, e)))?
                } else {
                    Input::<String>::new()
                        .with_prompt(help_text)
                        .interact_text()
                        .map_err(|e| CopierError::PromptError(format!("Error prompting for '{}': {}", name, e)))?
                };
                Value::from(result)
            }
            QuestionType::Bool => {
                let default_bool = default_value.is_true();

                let result = Confirm::new()
                    .with_prompt(help_text)
                    .default(default_bool)
                    .interact()
                    .map_err(|e| CopierError::PromptError(format!("Error prompting for '{}': {}", name, e)))?;
                Value::from(result)
            }
            QuestionType::Int => {
                let result = if let Ok(json_val) = serde_json::to_value(&default_value) {
                    if let Ok(i) = serde_json::from_value::<i64>(json_val) {
                        Input::<i64>::new()
                            .with_prompt(help_text)
                            .default(i)
                            .interact_text()
                            .map_err(|e| CopierError::PromptError(format!("Error prompting for '{}': {}", name, e)))?
                    } else {
                        Input::<i64>::new()
                            .with_prompt(help_text)
                            .interact_text()
                            .map_err(|e| CopierError::PromptError(format!("Error prompting for '{}': {}", name, e)))?
                    }
                } else {
                    Input::<i64>::new()
                        .with_prompt(help_text)
                        .interact_text()
                        .map_err(|e| CopierError::PromptError(format!("Error prompting for '{}': {}", name, e)))?
                };
                Value::from(result)
            }
            QuestionType::Float => {
                let result = if let Ok(json_val) = serde_json::to_value(&default_value) {
                    if let Ok(f) = serde_json::from_value::<f64>(json_val) {
                        Input::<f64>::new()
                            .with_prompt(help_text)
                            .default(f)
                            .interact_text()
                            .map_err(|e| CopierError::PromptError(format!("Error prompting for '{}': {}", name, e)))?
                    } else {
                        Input::<f64>::new()
                            .with_prompt(help_text)
                            .interact_text()
                            .map_err(|e| CopierError::PromptError(format!("Error prompting for '{}': {}", name, e)))?
                    }
                } else {
                    Input::<f64>::new()
                        .with_prompt(help_text)
                        .interact_text()
                        .map_err(|e| CopierError::PromptError(format!("Error prompting for '{}': {}", name, e)))?
                };
                Value::from(result)
            }
        };

        // Validate
        if let Some(ref validator) = question.validator {
            self.validate(&value, validator, context)
                .map_err(|e| match e {
                    CopierError::Validation(msg) => CopierError::Validation(format!("Validation error for '{}': {}", name, msg)),
                    CopierError::Template(msg) => CopierError::Template(format!("Error in validator for '{}': {}", name, msg)),
                    other => other,
                })?;
        }

        Ok(value)
    }

    /// Evaluate default value (may contain Jinja2)
    fn evaluate_default(
        &self,
        default: &serde_json::Value,
        context: &HashMap<String, Value>,
    ) -> Result<Value> {
        // If default is a string, it might contain Jinja2 template
        if let Some(s) = default.as_str() {
            let env = Environment::new();
            let template = env
                .template_from_str(s)
                .map_err(|e| CopierError::Template(format!("Failed to parse default value template '{}': {}", s, e)))?;

            let ctx = Value::from_serialize(context);
            let rendered = template
                .render(ctx)
                .map_err(|e| CopierError::Template(format!("Failed to render default value template '{}': {}", s, e)))?;

            Ok(Value::from(rendered))
        } else {
            // Convert serde_json::Value to minijinja::Value
            Ok(Value::from_serialize(default))
        }
    }

    /// Check if variable should be prompted (when clause)
    fn should_prompt(
        &self,
        question: &Question,
        context: &HashMap<String, Value>,
    ) -> Result<bool> {
        if let Some(ref when_clause) = question.when {
            let env = Environment::new();
            let template = env
                .template_from_str(when_clause)
                .map_err(|e| CopierError::Template(format!("Failed to parse 'when' clause '{}': {}", when_clause, e)))?;

            let ctx = Value::from_serialize(context);
            let result = template
                .render(ctx)
                .map_err(|e| CopierError::Template(format!("Failed to render 'when' clause '{}': {}", when_clause, e)))?;

            // Parse result as boolean
            Ok(!result.trim().is_empty() && result.trim() != "False" && result.trim() != "false")
        } else {
            Ok(true)
        }
    }

    /// Validate user input
    fn validate(
        &self,
        value: &Value,
        validator: &str,
        context: &HashMap<String, Value>,
    ) -> Result<()> {
        let mut ctx = context.clone();
        ctx.insert("value".to_string(), value.clone());

        let env = Environment::new();
        let template = env
            .template_from_str(validator)
            .map_err(|e| CopierError::Template(format!("Failed to parse validator template: {}\nValidator: {}", e, validator)))?;

        let ctx_value = Value::from_serialize(&ctx);
        let result = template
            .render(ctx_value)
            .map_err(|e| CopierError::Template(format!("Failed to render validator template: {}\nValidator: {}", e, validator)))?;

        let error_message = result.trim();
        if !error_message.is_empty() {
            return Err(CopierError::Validation(error_message.to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_default() {
        let config = CopierConfig {
            template: None,
            templates_suffix: None,
            skip_if_exists: None,
            envops: None,
            tasks: None,
            questions: HashMap::new(),
        };

        let prompter = VariablePrompter {
            config,
            non_interactive: false,
            defaults: HashMap::new(),
        };

        let mut context = HashMap::new();
        context.insert("name".to_string(), Value::from("test"));

        let default = serde_json::Value::String("prefix_{{ name }}".to_string());
        let result = prompter.evaluate_default(&default, &context).unwrap();

        assert_eq!(result.as_str().unwrap(), "prefix_test");
    }

    #[test]
    fn test_should_prompt_true() {
        let mut questions = HashMap::new();
        questions.insert(
            "test".to_string(),
            Question {
                question_type: QuestionType::Str,
                help: None,
                default: None,
                validator: None,
                when: Some("{{ enable_feature }}".to_string()),
                choices: None,
            },
        );

        let config = CopierConfig {
            template: None,
            templates_suffix: None,
            skip_if_exists: None,
            envops: None,
            tasks: None,
            questions,
        };

        let prompter = VariablePrompter {
            config,
            non_interactive: false,
            defaults: HashMap::new(),
        };

        let mut context = HashMap::new();
        context.insert("enable_feature".to_string(), Value::from(true));

        let result = prompter.should_prompt(&prompter.config.questions["test"], &context).unwrap();
        assert!(result);
    }
}
