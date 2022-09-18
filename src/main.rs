use clap::Parser;
mod cli;
use cli::{Cli, Commands};
mod config;
mod docker;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => {
            docker::run(&args).await;
        }
        Commands::Build(args) => {
            docker::build(&args.common).await;
        }
    }
}
