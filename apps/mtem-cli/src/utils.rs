use colored::*;
use std::env;

/// Check if output should be colored
pub fn should_color_output() -> bool {
    // Respect NO_COLOR environment variable
    env::var("NO_COLOR").is_err()
}

/// Create a colored string if coloring is enabled
pub fn colorize<S: Into<String>>(text: S, color_fn: fn(&str) -> ColoredString) -> String {
    if should_color_output() {
        color_fn(&text.into()).to_string()
    } else {
        text.into()
    }
}

/// Convenience functions for common colors
pub fn bold_green<S: Into<String>>(text: S) -> String {
    colorize(text, |s| s.bold().green())
}

pub fn bold_cyan<S: Into<String>>(text: S) -> String {
    colorize(text, |s| s.bold().cyan())
}

pub fn yellow<S: Into<String>>(text: S) -> String {
    colorize(text, |s| s.yellow())
}

pub fn dimmed<S: Into<String>>(text: S) -> String {
    colorize(text, |s| s.dimmed())
}

pub fn green<S: Into<String>>(text: S) -> String {
    colorize(text, |s| s.green())
}