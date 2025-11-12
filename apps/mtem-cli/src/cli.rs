use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "mtem",
    about = "Masstemplate - Create projects from local templates",
    long_about = "Masstemplate creates new projects by copying files from template directories stored locally in ~/.local/masstemplate/.

Examples:
  mtem list                    # Show available templates
  mtem info rust               # Show details about the 'rust' template
  mtem apply rust              # Create a new Rust project in current directory
  mtem apply node --dest myapp # Create a Node.js project in 'myapp' directory

Templates are just directories containing your starter files. Add .mtem/post_install.sh for setup scripts."
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true, help = "Enable verbose output")]
    pub verbose: bool,

    #[arg(long, global = true, help = "Output in JSON format for machine processing")]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all available templates
    List,

    /// Apply a template to create a new project
    Apply {
        /// Name of the template to apply
        template: String,

        /// Destination directory (defaults to current directory)
        #[arg(short, long)]
        dest: Option<PathBuf>,

        /// Collision strategy: skip, overwrite, backup, or merge
        #[arg(short, long)]
        collision: Option<String>,

        /// Skip interactive prompts, use default values
        #[arg(short = 'y', long)]
        yes: bool,

        /// Set template variables (e.g., --data key=value)
        #[arg(short = 'D', long = "data", value_name = "KEY=VALUE")]
        data: Vec<String>,
    },

    /// Create a new project directory and apply a template
    Create {
        /// Name of the template to apply
        template: String,

        /// Path where the new project will be created
        path: PathBuf,

        /// Collision strategy: skip, overwrite, backup, or merge
        #[arg(short, long)]
        collision: Option<String>,

        /// Skip interactive prompts, use default values
        #[arg(short = 'y', long)]
        yes: bool,

        /// Set template variables (e.g., --data key=value)
        #[arg(short = 'D', long = "data", value_name = "KEY=VALUE")]
        data: Vec<String>,
    },

    /// Show detailed information about a template
    Info {
        /// Name of the template to inspect
        template: String,
    },

    /// Manage template source directories
    Sources {
        #[command(subcommand)]
        command: SourcesCommand,
    },
}

#[derive(Subcommand)]
pub enum SourcesCommand {
    /// Add a template source directory
    Add {
        /// Path to the template source directory
        path: PathBuf,
    },

    /// Remove a template source directory
    Remove {
        /// Path to the template source directory
        path: PathBuf,
    },

    /// List all template source directories
    List,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parse_list_command() {
        let args = vec!["mtem", "list"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::List => {},
            _ => panic!("Expected List command"),
        }
        assert!(!cli.verbose);
    }

    #[test]
    fn test_cli_parse_apply_command() {
        let args = vec!["mtem", "apply", "rust", "--dest", "/tmp/test"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Apply { template, dest, collision, yes, .. } => {
                assert_eq!(template, "rust");
                assert_eq!(dest, Some(PathBuf::from("/tmp/test")));
                assert!(collision.is_none());
                assert!(!yes);
            }
            _ => panic!("Expected Apply command"),
        }
    }

    #[test]
    fn test_cli_parse_apply_with_options() {
        let args = vec!["mtem", "apply", "node", "-d", "myapp", "-c", "overwrite", "-y"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Apply { template, dest, collision, yes, .. } => {
                assert_eq!(template, "node");
                assert_eq!(dest, Some(PathBuf::from("myapp")));
                assert_eq!(collision, Some("overwrite".to_string()));
                assert!(yes);
            }
            _ => panic!("Expected Apply command"),
        }
    }

    #[test]
    fn test_cli_parse_info_command() {
        let args = vec!["mtem", "info", "template-name"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Info { template } => {
                assert_eq!(template, "template-name");
            }
            _ => panic!("Expected Info command"),
        }
    }

    #[test]
    fn test_cli_verbose_flag() {
        let args = vec!["mtem", "--verbose", "list"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
    }
}