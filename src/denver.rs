use std::fmt::Display;

use regex::Regex;

use crate::cli::{Cli, Commands, Common, Run, Status, Stop};
use crate::config::{read_config, Config, ContainerConfig};
use crate::docker::{DockerClient, DockerError};

pub struct Denver;

impl Denver {
    fn get_container_config<'a>(
        config: &'a Config,
        container_name: &String,
    ) -> Result<&'a ContainerConfig, DenverError> {
        let container = &config.containers.get(container_name);

        match container {
            Some(container) => Ok(container),
            None => Err(DenverError::UnknownContainer(format!(
                "{} not found",
                container_name
            ))),
        }
    }

    async fn run(config: &str, args: &Run) -> Result<(), DenverError> {
        let config = read_config(config);
        let mut docker = DockerClient::new(&config);
        let name = &args.common.container;
        let container = Denver::get_container_config(&config, name)?;

        if !args.no_rebuild {
            docker.build_image(&args.common, container).await?;
        }
        docker.create_container(name, container).await?;
        docker.run_container().await?;

        Ok(())
    }

    async fn build(config: &str, args: &Common) -> Result<(), DenverError> {
        let config = read_config(config);
        let docker = DockerClient::new(&config);
        let name = &args.container;
        let container = Denver::get_container_config(&config, name)?;

        docker.build_image(args, container).await?;

        Ok(())
    }

    async fn status(config: &str, args: &Status) -> Result<(), DenverError> {
        let config = read_config(config);
        let docker = DockerClient::new(&config);
        let containers = docker.list_containers().await?;
        let re = Regex::new(&args.pattern)?;

        println!("CONTAINER ID\tNAME\t\tIMAGE\t\t\t\t\t\tSTATE\t\tSTATUS");

        for container in &containers {
            let name = &container.names[0][1..];

            if re.is_match(name) {
                println!(
                    "{}\t{}\t{}\t{}\t\t{}",
                    &container.id[..12],
                    name,
                    container.image,
                    container.state.to_uppercase(),
                    container.status
                );
            }
        }

        for (name, config) in &config.containers {
            if !re.is_match(name) {
                continue;
            }

            if !containers
                .iter()
                .flat_map(|e| &e.names)
                .map(|e| &e[1..])
                .any(|e| e == name.as_str())
            {
                println!("{}\t{}\t{}\tNOT CREATED", "-".repeat(12), name, config.tag);
            }
        }

        Ok(())
    }

    async fn stop(config: &str, args: &Stop) -> Result<(), DenverError> {
        let config = read_config(config);
        let docker = DockerClient::new(&config);
        let containers = docker.list_containers().await?;
        let re = Regex::new(&args.pattern)?;

        for container in &containers {
            let name = &container.names[0][1..];

            if re.is_match(name) {
                println!("Stopping {} - {}", &container.id[..12], name);
                docker.stop_container(&container.id).await?;
            }
        }

        Ok(())
    }
}

pub enum DenverError {
    UnknownContainer(String),
    BuildError(String),
    RunError(String),
    StopError(String),
    StatusError(String),
    InvalidRegex(String),
}

impl Display for DenverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DenverError::UnknownContainer(e)
            | DenverError::StatusError(e)
            | DenverError::InvalidRegex(e)
            | DenverError::RunError(e)
            | DenverError::StopError(e)
            | DenverError::BuildError(e) => {
                write!(f, "{}", e)
            }
        }
    }
}

impl From<DockerError> for DenverError {
    fn from(e: DockerError) -> Self {
        match e {
            DockerError::List(e) => DenverError::StatusError(e),
            DockerError::Build(e) => DenverError::BuildError(e),
            DockerError::Run(e) => DenverError::RunError(e),
            DockerError::Stop(e) => DenverError::StopError(e),
        }
    }
}

impl From<regex::Error> for DenverError {
    fn from(e: regex::Error) -> Self {
        DenverError::InvalidRegex(e.to_string())
    }
}

pub async fn run(cli: Cli) {
    let result = match cli.command {
        Commands::Run(args) => Denver::run(&cli.config, &args).await,
        Commands::Build(args) => Denver::build(&cli.config, &args.common).await,
        Commands::Status(args) => Denver::status(&cli.config, &args).await,
        Commands::Stop(args) => Denver::stop(&cli.config, &args).await,
    };

    match result {
        Ok(()) => {}
        Err(e) => println!("Error: {}", e),
    }
}
