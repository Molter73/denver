use crate::cli::{Common, Run};
use crate::config::read_config;
use crate::docker::DockerClient;
/*
pub enum DenverError {
    UnknownContainer(String),
}
*/
pub struct Denver;

impl Denver {
    pub async fn run(args: &Run) {
        let config = read_config(&args.common.config);
        let mut docker = DockerClient::new(&config);
        let name = &args.common.container;
        let container = &config.containers.get(&args.common.container);

        if container.is_none() {
            eprintln!("Error: {} not found", args.common.container);
            return;
        }

        let container = container.unwrap();

        if !args.no_rebuild {
            docker.build_image(&args.common, container).await;
        }
        docker.create_container(name, container).await;
        docker.run_container().await;
    }

    pub async fn build(args: &Common) {
        let config = read_config(&args.config);
        let docker = DockerClient::new(&config);
        let container = &config.containers.get(&args.container);

        if container.is_none() {
            eprintln!("Error: {} not found", args.container);
            return;
        }

        docker.build_image(args, container.unwrap()).await;
    }
}
