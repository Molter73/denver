use crate::cli::{Cli, Commands, Common, Run};
use crate::config::{read_config, Config, ContainerConfig};
use crate::docker::DockerClient;

pub enum DenverError {
    UnknownContainer(String),
}

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

    async fn run(args: &Run) -> Result<(), DenverError> {
        let config = read_config(&args.common.config);
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

    async fn build(args: &Common) -> Result<(), DenverError> {
        let config = read_config(&args.config);
        let docker = DockerClient::new(&config);
        let name = &args.container;
        let container = Denver::get_container_config(&config, name)?;

        docker.build_image(args, container).await;

        Ok(())
    }
}

pub async fn run(cli: Cli) {
    let result = match cli.command {
        Commands::Run(args) => Denver::run(&args).await,
        Commands::Build(args) => Denver::build(&args.common).await,
    };

    match result {
        Ok(()) => {}
        Err(DenverError::UnknownContainer(e)) => eprintln!("Error: {}", e),
    }
}
