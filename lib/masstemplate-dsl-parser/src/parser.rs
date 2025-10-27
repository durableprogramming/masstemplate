use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, multispace0, multispace1, line_ending},
    combinator::{map, opt},
    multi::many0,
    sequence::{preceded, separated_pair, tuple},
    IResult,
};
use std::collections::HashMap;

use crate::types::{CollisionStrategy, DslConfig, Matcher};
use masstemplate_processors::Processor;

pub fn parse_dsl(input: &str) -> Result<DslConfig, String> {
    match parse_dsl_config(input) {
        Ok((remaining, config)) => {
            if remaining.trim().is_empty() {
                Ok(config)
            } else {
                Err(format!("Failed to parse DSL: unexpected input '{}'", remaining.trim()))
            }
        }
        Err(err) => Err(format!("Failed to parse DSL: {:?}", err)),
    }
}

fn parse_dsl_config(input: &str) -> IResult<&str, DslConfig> {
        map(
            many0(alt((
                map(parse_comment_or_empty, |_| None),
                map(parse_collision_command, Some),
                map(parse_processor_command, Some),
                map(parse_matcher_block, Some),
                map(parse_recursive_command, Some),
                map(parse_priority_command, Some),
            ))),
        |commands| {
            let mut config = DslConfig::default();
            for command in commands.into_iter().flatten() {
                match command {
                    ParsedCommand::Collision(strategy) => {
                        config.collision_strategy = Some(strategy);
                    }
                    ParsedCommand::Processor(processor) => {
                        config.processors.push(processor);
                    }
                    ParsedCommand::Matcher(matcher) => {
                        config.matchers.push(matcher);
                    }
                    ParsedCommand::Recursive(recursive) => {
                        config.recursive = Some(recursive);
                    }
                    ParsedCommand::Priority(priority) => {
                        config.priority = Some(priority);
                    }
                }
            }
            config
        },
    )(input)
}

fn parse_comment_or_empty(input: &str) -> IResult<&str, ()> {
    alt((
        map(
            tuple((multispace0, tag("#"), take_while1(|c| c != '\n'), opt(line_ending))),
            |_| (),
        ),
        map(multispace1, |_| ()),
        map(line_ending, |_| ()),
    ))(input)
}

#[derive(Debug)]
enum ParsedCommand {
    Collision(CollisionStrategy),
    Processor(Processor),
    Matcher(Matcher),
    Recursive(bool),
    Priority(i32),
}



fn parse_collision_command(input: &str) -> IResult<&str, ParsedCommand> {
    map(
        preceded(
            tuple((tag("collision"), multispace1)),
            alt((
                map(tag("skip"), |_| CollisionStrategy::Skip),
                map(tag("overwrite"), |_| CollisionStrategy::Overwrite),
                map(tag("backup"), |_| CollisionStrategy::Backup),
                map(tag("merge"), |_| CollisionStrategy::Merge),
            )),
        ),
        ParsedCommand::Collision,
    )(input)
}

fn parse_processor_command(input: &str) -> IResult<&str, ParsedCommand> {
    alt((
        parse_dotenv_set,
        parse_dotenv_append,
        parse_replace,
        parse_template,
    ))(input)
}

fn parse_matcher_block(input: &str) -> IResult<&str, ParsedCommand> {
    map(
        tuple((
            multispace0,
            tag("match"),
            multispace1,
            take_while1(|c: char| !c.is_whitespace() && c != '{'),
            multispace0,
            tag("{"),
            multispace0,
            many0(alt((
                map(parse_comment_or_empty, |_| None),
                map(parse_processor_command, Some),
            ))),
            multispace0,
            tag("}"),
            multispace0,
        )),
        |(_, _, _, pattern, _, _, _, commands, _, _, _)| {
            let processors = commands.into_iter().flatten().filter_map(|cmd| {
                match cmd {
                    ParsedCommand::Processor(processor) => Some(processor),
                    _ => None, // Only processors are allowed in match blocks
                }
            }).collect();
            ParsedCommand::Matcher(Matcher {
                pattern: pattern.to_string(),
                processors,
            })
        },
    )(input)
}

fn parse_dotenv_set(input: &str) -> IResult<&str, ParsedCommand> {
    map(
        preceded(
            tuple((tag("dotenv"), multispace1, tag("set"), multispace1)),
            separated_pair(
                take_while1(|c: char| c.is_alphanumeric() || c == '_'),
                char('='),
                take_while1(|c| c != '\n' && c != '\r'),
            ),
        ),
        |(key, value): (&str, &str)| {
            ParsedCommand::Processor(Processor::DotenvSet {
                key: key.to_string(),
                value: value.to_string(),
            })
        },
    )(input)
}

fn parse_dotenv_append(input: &str) -> IResult<&str, ParsedCommand> {
    map(
        preceded(
            tuple((tag("dotenv"), multispace1, tag("append"), multispace1)),
            separated_pair(
                take_while1(|c: char| c.is_alphanumeric() || c == '_'),
                char('='),
                take_while1(|c| c != '\n' && c != '\r'),
            ),
        ),
        |(key, value): (&str, &str)| {
            ParsedCommand::Processor(Processor::DotenvAppend {
                key: key.to_string(),
                value: value.to_string(),
            })
        },
    )(input)
}

fn parse_replace(input: &str) -> IResult<&str, ParsedCommand> {
    map(
        preceded(
            tuple((tag("replace"), multispace1)),
            separated_pair(
                take_while1(|c| c != ' ' && c != '\t' && c != '\n' && c != '\r'),
                multispace1,
                take_while1(|c| c != '\n' && c != '\r'),
            ),
        ),
        |(pattern, replacement): (&str, &str)| {
            ParsedCommand::Processor(Processor::Replace {
                pattern: pattern.to_string(),
                replacement: replacement.trim_end().to_string(),
            })
        },
    )(input)
}

fn parse_template(input: &str) -> IResult<&str, ParsedCommand> {
    // Parse template variables: template KEY1=VALUE1 KEY2=VALUE2
    map(
        preceded(
            tuple((tag("template"), multispace1)),
            take_while(|c| c != '\n' && c != '\r'),
        ),
        |line: &str| {
            let mut variables = HashMap::new();
            for pair in line.split_whitespace() {
                if let Some((key, value)) = pair.split_once('=')
                    && !key.is_empty() && key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    variables.insert(key.to_string(), value.to_string());
                }
            }
            ParsedCommand::Processor(Processor::Template { variables })
        },
    )(input)
}

fn parse_recursive_command(input: &str) -> IResult<&str, ParsedCommand> {
    map(
        preceded(
            tuple((tag("recursive"), multispace1)),
            alt((
                map(tag("true"), |_| true),
                map(tag("false"), |_| false),
            )),
        ),
        ParsedCommand::Recursive,
    )(input)
}

fn parse_priority_command(input: &str) -> IResult<&str, ParsedCommand> {
    map(
        preceded(
            tuple((tag("priority"), multispace1)),
            nom::character::complete::i32,
        ),
        ParsedCommand::Priority,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_collision_skip() {
        let result = parse_dsl("collision skip");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.collision_strategy, Some(CollisionStrategy::Skip));
    }

    #[test]
    fn test_parse_collision_overwrite() {
        let result = parse_dsl("collision overwrite");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.collision_strategy, Some(CollisionStrategy::Overwrite));
    }

    #[test]
    fn test_parse_dotenv_set() {
        let result = parse_dsl("dotenv set API_KEY=secret123");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.processors.len(), 1);
        match &config.processors[0] {
            Processor::DotenvSet { key, value } => {
                assert_eq!(key, "API_KEY");
                assert_eq!(value, "secret123");
            }
            _ => panic!("Expected DotenvSet processor"),
        }
    }

    #[test]
    fn test_parse_dotenv_append() {
        let result = parse_dsl("dotenv append PATH=:/usr/local/bin");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.processors.len(), 1);
        match &config.processors[0] {
            Processor::DotenvAppend { key, value } => {
                assert_eq!(key, "PATH");
                assert_eq!(value, ":/usr/local/bin");
            }
            _ => panic!("Expected DotenvAppend processor"),
        }
    }

    #[test]
    fn test_parse_multiple_commands() {
        let dsl = r#"
            collision overwrite
            dotenv set KEY1=value1
            dotenv set KEY2=value2
        "#;
        let result = parse_dsl(dsl);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.collision_strategy, Some(CollisionStrategy::Overwrite));
        assert_eq!(config.processors.len(), 2);
    }

    #[test]
    fn test_parse_with_comments() {
        let dsl = r#"
            # This is a comment
            collision skip
            # Another comment
            dotenv set TEST_KEY=test_value
        "#;
        let result = parse_dsl(dsl);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.collision_strategy, Some(CollisionStrategy::Skip));
        assert_eq!(config.processors.len(), 1);
    }

    #[test]
    fn test_parse_matcher_block() {
        let dsl = r#"match *.txt {
            replace old new
            template KEY=value
        }"#;
        let result = parse_dsl(dsl);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.matchers.len(), 1);
        assert_eq!(config.matchers[0].pattern, "*.txt");
        assert_eq!(config.matchers[0].processors.len(), 2);
    }

    #[test]
    fn test_parse_recursive_true() {
        let result = parse_dsl("recursive true");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.recursive, Some(true));
    }

    #[test]
    fn test_parse_recursive_false() {
        let result = parse_dsl("recursive false");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.recursive, Some(false));
    }

    #[test]
    fn test_parse_priority() {
        let result = parse_dsl("priority 10");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.priority, Some(10));
    }

    #[test]
    fn test_parse_template_simple() {
        let result = parse_dsl("template NAME=World");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.processors.len(), 1);
        match &config.processors[0] {
            Processor::Template { variables } => {
                assert_eq!(variables.get("NAME"), Some(&"World".to_string()));
            }
            _ => panic!("Expected Template processor"),
        }
    }

    #[test]
    fn test_parse_template_multiple_variables() {
        let result = parse_dsl("template NAME=World GREETING=Hello");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.processors.len(), 1);
        match &config.processors[0] {
            Processor::Template { variables } => {
                assert_eq!(variables.get("NAME"), Some(&"World".to_string()));
                assert_eq!(variables.get("GREETING"), Some(&"Hello".to_string()));
            }
            _ => panic!("Expected Template processor"),
        }
    }

    #[test]
    fn test_parse_replace_with_spaces() {
        let result = parse_dsl("replace old_text new_text");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.processors.len(), 1);
        match &config.processors[0] {
            Processor::Replace { pattern, replacement } => {
                assert_eq!(pattern, "old_text");
                assert_eq!(replacement, "new_text");
            }
            _ => panic!("Expected Replace processor"),
        }
    }

    #[test]
    fn test_parse_replace_with_special_chars() {
        let result = parse_dsl("replace __VAR__ actual_value");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.processors.len(), 1);
        match &config.processors[0] {
            Processor::Replace { pattern, replacement } => {
                assert_eq!(pattern, "__VAR__");
                assert_eq!(replacement, "actual_value");
            }
            _ => panic!("Expected Replace processor"),
        }
    }

    #[test]
    fn test_parse_template_with_underscores() {
        let result = parse_dsl("template API_KEY=secret_value DATABASE_URL=postgres://localhost");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.processors.len(), 1);
        match &config.processors[0] {
            Processor::Template { variables } => {
                assert_eq!(variables.get("API_KEY"), Some(&"secret_value".to_string()));
                assert_eq!(variables.get("DATABASE_URL"), Some(&"postgres://localhost".to_string()));
            }
            _ => panic!("Expected Template processor"),
        }
    }

    #[test]
    fn test_parse_full_config() {
        let dsl = r#"
            collision overwrite
            recursive false
            priority 5
            dotenv set API_KEY=secret
            template NAME=World
            match *.txt {
                replace hello hi
            }
        "#;
        let result = parse_dsl(dsl);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.collision_strategy, Some(CollisionStrategy::Overwrite));
        assert_eq!(config.recursive, Some(false));
        assert_eq!(config.priority, Some(5));
        assert_eq!(config.processors.len(), 2); // dotenv and template
        assert_eq!(config.matchers.len(), 1);
    }

    #[test]
    fn test_parse_advanced_example() {
        let dsl = r#"
            collision backup
            recursive true
            priority 10
            dotenv set DATABASE_URL=postgres://prod-db:5432/myapp
            dotenv set REDIS_URL=redis://prod-cache:6379
            template SERVICE_NAME=MyMicroservice VERSION=0.1.0
            replace old-api-endpoint.com new-api-endpoint.com
            match *.md {
                replace placeholder actual_content
            }
        "#;
        let result = parse_dsl(dsl);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.collision_strategy, Some(CollisionStrategy::Backup));
        assert_eq!(config.recursive, Some(true));
        assert_eq!(config.priority, Some(10));
        assert_eq!(config.processors.len(), 4); // 2 dotenv, 1 template, 1 replace
        assert_eq!(config.matchers.len(), 1);
        assert_eq!(config.matchers[0].pattern, "*.md");
        assert_eq!(config.matchers[0].processors.len(), 1);
    }

    #[test]
    fn test_parse_dev_example() {
        let dsl = r#"
            collision skip
            recursive false
            priority 5
            dotenv set DATABASE_URL=postgres://localhost:5432/myapp
            template ENV=development DEBUG=true
            replace prod-server.com localhost:3000
            match *.log {
                replace sensitive_info [REDACTED]
            }
        "#;
        let result = parse_dsl(dsl);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.collision_strategy, Some(CollisionStrategy::Skip));
        assert_eq!(config.recursive, Some(false));
        assert_eq!(config.priority, Some(5));
        assert_eq!(config.processors.len(), 3); // 1 dotenv, 1 template, 1 replace
        assert_eq!(config.matchers.len(), 1);
        assert_eq!(config.matchers[0].pattern, "*.log");
        assert_eq!(config.matchers[0].processors.len(), 1);
    }
}