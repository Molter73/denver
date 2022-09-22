use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about="A Development ENVironment managER", long_about=None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    // Config file to use
    #[clap(
        short,
        long,
        value_parser,
        default_value = "~/.config/denver/config.yml",
        help = "The path to the configuration file to be used"
    )]
    pub config: String,
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(about = "Run a development container")]
    Run(Run),
    #[clap(about = "Build a development container, but don't run it")]
    Build(Build),
    #[clap(about = "List containers managed by denver")]
    Status(Status),
}

#[derive(Args)]
pub struct Common {
    // Container to be run/build
    #[clap(
        value_parser,
        help = "The name assigned to the container in the configuration file"
    )]
    pub container: String,

    // Build image with no cache
    #[clap(
        short,
        long,
        action,
        help = "Build the container image without using cache from previous buids"
    )]
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
    #[clap(long, action, help = "Run the container without rebuilding its image")]
    pub no_rebuild: bool,
}

#[derive(Args)]
pub struct Status {
    #[clap(
        value_parser,
        default_value = ".*",
        help = "List only containers matching this pattern"
    )]
    pub pattern: String,
}
