use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about="A Development ENVironment managER", long_about=None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    // Config file to use
    #[arg(
        short,
        long,
        default_value = "~/.config/denver/config.yml",
        help = "The path to the configuration file to be used"
    )]
    pub config: String,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Run a development container")]
    Run(Run),
    #[command(about = "Build a development container, but don't run it")]
    Build(Build),
    #[command(about = "List containers managed by denver")]
    Status(Status),
    #[command(about = "Stop running containers")]
    Stop(Stop),
    #[command(about = "Generate auto-completions")]
    Completion(Completion),
}

#[derive(Args)]
pub struct Common {
    // Container to be run/build
    #[arg(help = "The name assigned to the container in the configuration file")]
    pub container: String,

    // Build image with no cache
    #[arg(
        short,
        long,
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
    #[arg(long, help = "Run the container without rebuilding its image")]
    pub no_rebuild: bool,
}

#[derive(Args)]
pub struct Status {
    #[arg(
        default_value = ".*",
        help = "List only containers matching this pattern"
    )]
    pub pattern: String,
}

#[derive(Args)]
pub struct Stop {
    #[arg(
        default_value = ".*",
        help = "Stop only containers matching this pattern"
    )]
    pub pattern: String,
}

#[derive(Args)]
pub struct Completion {
    #[arg(help = "Generate auto-completions for this shell")]
    pub shell: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_test() {
        use clap::CommandFactory;

        Cli::command().debug_assert();
    }
}
