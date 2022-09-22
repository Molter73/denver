use std::collections::HashMap;
use std::ops::Deref;

use futures::StreamExt;
use serde_json::Value;
use shiplift::{
    rep::Container, BuildOptions, ContainerFilter, ContainerListOptions, ContainerOptions, Docker,
};

use crate::cli::Common;
use crate::config::{Config, ContainerConfig};

const DENVER_LABEL: (&str, &str) = ("manager", "denver");

pub enum DockerError {
    ListError(String),
}

pub struct DockerClient {
    docker: Docker,
    id: String,
}

impl DockerClient {
    pub fn new(config: &Config) -> Self {
        let docker = Docker::unix(&config.socket);

        DockerClient {
            docker,
            id: String::from(""),
        }
    }

    pub async fn build_image(&self, args: &Common, container: &ContainerConfig) {
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
            .labels(&HashMap::from([DENVER_LABEL]))
            .build()
    }

    pub async fn create_container(&mut self, name: &str, container: &ContainerConfig) {
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

    pub async fn run_container(&self) {
        let docker = &self.docker;

        match docker.containers().get(&self.id).start().await {
            Ok(info) => println!("{:?}", info),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    pub async fn list_containers(&self) -> Result<Vec<Container>, DockerError> {
        let (label_key, label_value) = DENVER_LABEL;
        let options = ContainerListOptions::builder()
            .filter(vec![ContainerFilter::Label(
                label_key.to_string(),
                label_value.to_string(),
            )])
            .build();

        match self.docker.containers().list(&options).await {
            Ok(info) => Ok(info),
            Err(e) => Err(DockerError::ListError(format!("{:?}", e))),
        }
    }
}
