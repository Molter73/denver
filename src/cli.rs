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
pub struct Common {
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

    // Build image with no cache
    #[clap(short, long, action)]
    pub no_cache: bool,
}

#[derive(Args)]
pub struct Build {
    #[clap(flatten)]
    pub common: Common,
}

#[derive(Args)]
pub struct Run {
    #[clap(flatten)]
    pub common: Common,

    // If set, doesn't rebuild the image
    #[clap(long, action)]
    pub no_rebuild: bool,
}
