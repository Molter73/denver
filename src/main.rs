use clap::Parser;
mod cli;
use cli::{Cli, Commands};
mod config;
mod denver;
use denver::Denver;
mod docker;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => {
            Denver::run(&args).await;
        }
        Commands::Build(args) => {
            Denver::build(&args.common).await;
        }
    }
}
