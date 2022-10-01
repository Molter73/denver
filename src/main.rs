use clap::Parser;
mod cli;
use cli::Cli;
mod config;
mod denver;
mod docker;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    denver::run(cli).await;
}
