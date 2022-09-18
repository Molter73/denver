use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about=None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Run(Run),
    Build(Build),
}

#[derive(Args)]
pub struct Build {
    // Container to be run/build
    #[clap(value_parser)]
    pub container: String,

    // Config file to use
    #[clap(
        short,
        long,
        value_parser,
        default_value = "~/.config/denver/config.yml"
    )]
    pub config: String,
}

#[derive(Args)]
pub struct Run {
    // Container to be run/build
    #[clap(value_parser)]
    pub container: String,

    // Config file to use
    #[clap(
        short,
        long,
        value_parser,
        default_value = "~/.config/denver/config.yml"
    )]
    pub config: String,

    // If set, doesn't rebuild the image
    #[clap(short, long, action)]
    pub rebuild: bool,
}
