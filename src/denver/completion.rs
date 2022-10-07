use std::io;

use clap::CommandFactory;
use clap_complete::{
    generate,
    shells::{Bash, Zsh},
};

use crate::cli::{Cli, Completion};

pub enum CompletionError {
    UnkownShell(String),
}

fn print_completions<T: clap_complete::Generator>(gen: T) {
    generate(gen, &mut Cli::command(), "denver", &mut io::stdout());
}

pub fn completion(args: &Completion) -> Result<(), CompletionError> {
    match args.shell.to_lowercase().as_str() {
        "zsh" => print_completions(Zsh),
        "bash" => print_completions(Bash),
        s => return Err(CompletionError::UnkownShell(s.to_string())),
    }

    Ok(())
}
