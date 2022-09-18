use futures::StreamExt;
use serde_json::Value;
use shiplift::{builder::ContainerOptionsBuilder, BuildOptions, ContainerOptions, Docker};

use crate::cli::{Common, Run};
use crate::config::{read_config, Config, ContainerConfig, RunConfig};

struct DockerIface {
    docker: Docker,
    id: String,
}

impl DockerIface {
    fn new(config: &Config) -> Self {
        let docker = Docker::unix(&config.socket);

        DockerIface {
            docker,
            id: String::from(""),
        }
    }

    async fn build_image(&self, args: &Common, container: &ContainerConfig) {
        let docker = &self.docker;
        let build_options = &container.build;

        let options = BuildOptions::builder(&build_options.context)
            .dockerfile(&build_options.dockerfile)
            .tag(&container.tag)
            .nocache(args.no_cache)
            .build();

        let mut stream = docker.images().build(&options);
        while let Some(build_result) = stream.next().await {
            match build_result {
                Ok(output) => {
                    let stream = &output["stream"];
                    match stream {
                        Value::String(log) => {
                            print!("{}", log);
                        }
                        Value::Null => {}
                        _ => println!("{:?}", stream),
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }

    fn handle_run_options(&self, options: &mut ContainerOptionsBuilder, run_options: &RunConfig) {
        let args = &run_options.args;

        for arg in args {
            match &arg[..] {
                "i" | "interactive" => options.attach_stdin(true),
                "rm" => options.auto_remove(true),
                _ => options,
            };
        }
    }

    async fn create_container(&mut self, container: &ContainerConfig) {
        let docker = &self.docker;

        let options = &mut ContainerOptions::builder(container.tag.as_str());

        self.handle_run_options(options, &container.run);

        options.name("falco-fedora");
        let options = options.build();
        match docker.containers().create(&options).await {
            Ok(info) => {
                println!("{}", info.id);
                self.id = info.id;
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    async fn run_container(&self) {
        let docker = &self.docker;

        match docker.containers().get(&self.id).start().await {
            Ok(info) => println!("{:?}", info),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

pub async fn run(args: &Run) {
    let config = read_config(&args.common.config);
    let mut docker = DockerIface::new(&config);
    let container = &config.containers[&args.common.container];

    docker.build_image(&args.common, container).await;
    docker.create_container(container).await;
    docker.run_container().await;
}

pub async fn build(args: &Common) {
    let config = read_config(&args.config);
    let docker = DockerIface::new(&config);
    let container = &config.containers.get(&args.container);

    if container.is_none() {
        eprintln!("Error: {} not found", args.container);
        return;
    }
    docker.build_image(args, container.unwrap()).await;
}
