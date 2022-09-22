use crate::cli::{Cli, Commands, Common, Run, Status};
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
            docker.build_image(&args.common, container).await;
        }
        docker.create_container(name, container).await;
        docker.run_container().await;

        Ok(())
    }

    async fn build(config: &str, args: &Common) -> Result<(), DenverError> {
        let config = read_config(config);
        let docker = DockerClient::new(&config);
        let name = &args.container;
        let container = Denver::get_container_config(&config, name)?;

        docker.build_image(args, container).await;

        Ok(())
    }

    async fn status(config: &str, _args: &Status) -> Result<(), DenverError> {
        let config = read_config(config);
        let docker = DockerClient::new(&config);
        let containers = docker.list_containers().await?;

        println!("CONTAINER ID\tNAME\t\tIMAGE\t\t\t\t\t\tSTATE\t\tSTATUS");

        for container in &containers {
            println!(
                "{}\t{}\t{}\t{}\t\t{}",
                &container.id[..12],
                &container.names[0][1..],
                container.image,
                container.state.to_uppercase(),
                container.status
            );
        }

        for (name, config) in &config.containers {
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
}

pub enum DenverError {
    UnknownContainer(String),
    StatusError(String),
}

impl From<DockerError> for DenverError {
    fn from(e: DockerError) -> Self {
        match e {
            DockerError::ListError(e) => DenverError::StatusError(e),
        }
    }
}

pub async fn run(cli: Cli) {
    let result = match cli.command {
        Commands::Run(args) => Denver::run(&cli.config, &args).await,
        Commands::Build(args) => Denver::build(&cli.config, &args.common).await,
        Commands::Status(args) => Denver::status(&cli.config, &args).await,
    };

    match result {
        Ok(()) => {}
        Err(DenverError::UnknownContainer(e)) | Err(DenverError::StatusError(e)) => {
            eprintln!("Error: {}", e)
        }
    }
}
