use std::ops::Deref;

use futures::StreamExt;
use serde_json::Value;
use shiplift::{BuildOptions, ContainerOptions, Docker};

use crate::cli::{Common, Run};
use crate::config::{read_config, Config, ContainerConfig};

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
            .dockerfile(
                build_options
                    .dockerfile
                    .as_ref()
                    .unwrap_or(&"Dockerfile".to_string()),
            )
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

    fn create_run_options(name: &str, container: &ContainerConfig) -> ContainerOptions {
        static EMPTY_VEC: Vec<String> = vec![];
        let run_options = &container.run;
        let workspace_volume = format!("{}:{}", run_options.workspace, run_options.workspace);
        let mut volumes: Vec<&str> = vec![&workspace_volume[..]];
        let configured_volumes = run_options.volumes.as_ref().unwrap_or(&EMPTY_VEC);

        volumes.append(
            &mut configured_volumes
                .iter()
                .map(|s| s.deref())
                .collect::<Vec<&str>>(),
        );

        let args = run_options.args.as_ref();
        ContainerOptions::builder(&container.tag)
            .name(name)
            .attach_stdin(
                args.unwrap_or(&EMPTY_VEC)
                    .iter()
                    .any(|e| e == "i" || e == "interactive"),
            )
            .auto_remove(args.unwrap_or(&EMPTY_VEC).iter().any(|e| e == "rm"))
            .volumes(volumes)
            .working_dir(&run_options.workspace)
            .build()
    }

    async fn create_container(&mut self, name: &str, container: &ContainerConfig) {
        let docker = &self.docker;
        let options = Self::create_run_options(name, container);

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
    let docker = DockerIface::new(&config);
    let container = &config.containers.get(&args.container);

    if container.is_none() {
        eprintln!("Error: {} not found", args.container);
        return;
    }

    docker.build_image(args, container.unwrap()).await;
}
