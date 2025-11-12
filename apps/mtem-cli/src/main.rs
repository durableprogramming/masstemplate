use clap::Parser;
use mtem_cli::cli::{Cli, Commands, SourcesCommand};
use mtem_cli::commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => commands::list::execute(cli.json).await,
        Commands::Apply {
            template,
            dest,
            collision,
            yes,
            data,
        } => commands::apply::execute(&template, dest, collision, yes, data, cli.json).await,
        Commands::Create {
            template,
            path,
            collision,
            yes,
            data,
        } => commands::create::execute(&template, path, collision, yes, data, cli.json).await,
        Commands::Info { template } => commands::info::execute(&template, cli.json).await,
        Commands::Sources { command } => match command {
            SourcesCommand::Add { path } => commands::sources::add_source(path).await.map_err(Into::into),
            SourcesCommand::Remove { path } => commands::sources::remove_source(path).await.map_err(Into::into),
            SourcesCommand::List => commands::sources::list_sources().await.map_err(Into::into),
        },
    }
}


