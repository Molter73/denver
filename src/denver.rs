use crate::cli::{Cli, Commands, Common, Run};
use crate::config::read_config;
use crate::docker::DockerClient;

pub enum DenverError {
    UnknownContainer(String),
}

pub struct Denver;

impl Denver {
    pub async fn run(args: &Run) -> Result<(), DenverError> {
        let config = read_config(&args.common.config);
        let mut docker = DockerClient::new(&config);
        let name = &args.common.container;
        let container = &config.containers.get(&args.common.container);

        if container.is_none() {
            return Err(DenverError::UnknownContainer(format!(
                "{} not found",
                args.common.container
            )));
        }

        let container = container.unwrap();

        if !args.no_rebuild {
            docker.build_image(&args.common, container).await;
        }
        docker.create_container(name, container).await;
        docker.run_container().await;

        Ok(())
    }

    pub async fn build(args: &Common) -> Result<(), DenverError> {
        let config = read_config(&args.config);
        let docker = DockerClient::new(&config);
        let container = &config.containers.get(&args.container);

        if container.is_none() {
            return Err(DenverError::UnknownContainer(format!(
                "Error: {} not found",
                args.container
            )));
        }

        docker.build_image(args, container.unwrap()).await;

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
