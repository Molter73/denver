use clap::Parser;
use std::env;

mod cli;
use cli::{Cli, Commands};

mod config;
use config::Config;

mod docker;

fn read_config(config: &str) -> Config {
    let config = if let Some(relative_path) = config.strip_prefix('~') {
        format!("{}{}", env::var("HOME").unwrap(), relative_path)
    } else {
        config.to_owned()
    };

    let config = std::fs::read_to_string(&config)
        .unwrap_or_else(|_| panic!("Failed to read configuration file: {}", config));

    Config::new(config.as_str())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => {
            let config = read_config(&args.config);
            docker::run(&config).await;
        }
        Commands::Build(args) => {
            let config = read_config(&args.config);
            docker::run(&config).await;
        }
    }
}
