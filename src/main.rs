use std::env;

use clap::Parser;

mod config;
use config::Config;

mod docker;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    // Config file to use
    #[clap(
        short,
        long,
        value_parser,
        default_value = "~/.config/denver/config.yml"
    )]
    config: String,
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();
    let config = if args.config.starts_with('~') {
        let relative_path = args.config[1..].to_string();
        format!("{}{}", env::var("HOME").unwrap(), relative_path)
    } else {
        args.config
    };

    let config = std::fs::read_to_string(&config)
        .unwrap_or_else(|_| panic!("Failed to read configuration file: {}", config));

    let config = Config::new(config.as_str());

    docker::run(config).await
}
