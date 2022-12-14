use std::fmt::Display;
use std::path::Path;
use std::time::{Duration, SystemTime};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;

mod completion;
mod status;

use crate::cli::{Cli, Commands, Common, Completion, Run, Status, Stop};
use crate::config::{read_config, Config, ContainerConfig};
use crate::docker::{DockerClient, DockerError};

use self::completion::CompletionError;

pub struct Denver {
    config: Config,
    docker: DockerClient,
}

impl Denver {
    fn new(config: &str) -> Self {
        let config = read_config(config);
        let docker = DockerClient::new(&config);

        Denver { config, docker }
    }

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

    async fn run(&self, args: &Run) -> Result<(), DenverError> {
        let name = &args.common.container;
        let container = Denver::get_container_config(&self.config, name)?;

        if !args.no_rebuild {
            self.docker.build_image(&args.common, container).await?;
        }

        let running = self.docker.list_containers().await?;
        let running_container = running.iter().find(|c| &c.names[0][1..] == name.as_str());

        if let Some(running_container) = running_container {
            println!("Removing {}", &running_container.names[0][1..]);
            self.docker
                .remove_container(&running_container.id, true)
                .await?;
        }

        println!("Creating {} with image {}", name, container.tag);
        let id = self.docker.create_container(name, container).await?;
        self.docker.run_container(&id).await?;

        println!("Started {} - {}", &id[..12], name);

        Ok(())
    }

    async fn build(&self, args: &Common) -> Result<(), DenverError> {
        let name = &args.container;
        let container = Denver::get_container_config(&self.config, name)?;

        self.docker.build_image(args, container).await?;

        Ok(())
    }

    async fn status(&self, args: &Status) -> Result<(), DenverError> {
        static EMPTY_ID: &str = "------------";
        let containers = self.docker.list_containers().await?;
        let re = Regex::new(&args.pattern)?;
        let mut lines = status::Containers::new();

        // We first print all created containers
        for container in containers.iter().filter(|c| {
            let name = &c.names[0][1..];
            re.is_match(name)
        }) {
            let name = &container.names[0][1..];

            lines.push(status::Container::new(
                &container.id[..12],
                name,
                &container.image,
                &container.state,
                &container.status,
            ));
        }

        // And now we can print any containers that are not created
        for (name, config) in self
            .config
            .containers
            .iter()
            .filter(|(name, _)| re.is_match(name))
        {
            if !containers
                .iter()
                .map(|c| &c.names[0][1..])
                .any(|c| c == name)
            {
                lines.push(status::Container::new(
                    EMPTY_ID,
                    name,
                    &config.tag,
                    "NOT CREATED",
                    "",
                ));
            }
        }

        print!("{}", lines);

        Ok(())
    }

    async fn stop(&self, args: &Stop) -> Result<(), DenverError> {
        let containers = self.docker.list_containers().await?;
        let re = Regex::new(&args.pattern)?;

        for container in &containers {
            let name = &container.names[0][1..];

            if re.is_match(name) {
                println!("Stopping {} - {}", &container.id[..12], name);
                self.docker.stop_container(&container.id).await?;
            }
        }

        Ok(())
    }

    fn completion(args: &Completion) -> Result<(), DenverError> {
        completion::completion(args)?;
        Ok(())
    }

    async fn watch(&mut self, args: &Run) -> Result<(), DenverError> {
        let name = &args.common.container;
        let container = Denver::get_container_config(&self.config, name)?;
        let context = &container.build.context;
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let rt = tokio::runtime::Handle::current();

        let mut watcher = RecommendedWatcher::new(
            move |res| rt.block_on(async { tx.send(res).await.unwrap() }),
            notify::Config::default(),
        )?;

        watcher.watch(Path::new(context), RecursiveMode::Recursive)?;

        // Run the container and wait for changes to its context
        self.run(args).await?;
        let mut last_build = SystemTime::now();

        println!("Watching {} context for {}", context, name);

        while let Some(res) = rx.recv().await {
            match res {
                Ok(_) => {
                    let duration = SystemTime::now()
                        .duration_since(last_build)
                        .expect("Time went backwards?");

                    if duration >= Duration::from_secs(2) {
                        self.run(args).await?;
                        last_build = SystemTime::now();
                    }
                }
                Err(e) => return Err(DenverError::RunError(e.to_string())),
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
    RemoveError(String),
    StatusError(String),
    InvalidRegex(String),
    CompletionError(String),
}

impl Display for DenverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DenverError::UnknownContainer(e)
            | DenverError::CompletionError(e)
            | DenverError::StatusError(e)
            | DenverError::InvalidRegex(e)
            | DenverError::RunError(e)
            | DenverError::StopError(e)
            | DenverError::RemoveError(e)
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
            DockerError::Remove(e) => DenverError::RemoveError(e),
        }
    }
}

impl From<regex::Error> for DenverError {
    fn from(e: regex::Error) -> Self {
        DenverError::InvalidRegex(e.to_string())
    }
}

impl From<CompletionError> for DenverError {
    fn from(e: CompletionError) -> Self {
        match e {
            CompletionError::UnkownShell(e) => {
                DenverError::CompletionError(format!("No completion available for {}", e))
            }
        }
    }
}

impl From<notify::Error> for DenverError {
    fn from(e: notify::Error) -> Self {
        DenverError::RunError(e.to_string())
    }
}

pub async fn run(cli: Cli) {
    let mut denver = Denver::new(&cli.config);

    let result = match cli.command {
        Commands::Run(args) => denver.run(&args).await,
        Commands::Build(args) => denver.build(&args.common).await,
        Commands::Status(args) => denver.status(&args).await,
        Commands::Stop(args) => denver.stop(&args).await,
        Commands::Completion(args) => Denver::completion(&args),
        Commands::Watch(args) => denver.watch(&args).await,
    };

    match result {
        Ok(()) => {}
        Err(e) => println!("Error: {}", e),
    }
}
