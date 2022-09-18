use std::collections::HashMap;
use std::env;

use serde::Deserialize;

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename = "build")]
pub struct BuildConfig {
    pub dockerfile: String,
    pub context: String,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename = "run")]
pub struct RunConfig {
    pub args: Vec<String>,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct ContainerConfig {
    pub build: BuildConfig,
    pub run: RunConfig,
    pub tag: String,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct Config {
    pub socket: String,
    pub containers: HashMap<String, ContainerConfig>,
}

impl Config {
    pub fn new(config: &str) -> Self {
        serde_yaml::from_str(config).unwrap()
    }
}

pub fn read_config(config: &str) -> Config {
    let config = if let Some(relative_path) = config.strip_prefix('~') {
        format!("{}{}", env::var("HOME").unwrap(), relative_path)
    } else {
        config.to_owned()
    };

    let config = std::fs::read_to_string(&config)
        .unwrap_or_else(|_| panic!("Failed to read configuration file: {}", config));

    Config::new(config.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let socket = "/sockets/docker/fedora.sock/";
        let name = "test-container";
        let dockerfile = "Dockerfile";
        let context = "ctx/";
        let args = r#"
        - i
        - rm"#;
        let tag = "quay.io/org/some:tag";
        let config = format!(
            r#"
socket: {}
containers:
  {}:
    build:
        dockerfile: {}
        context: {}
    run:
        args: {}
    tag: {}
        "#,
            socket, name, dockerfile, context, args, tag
        );

        let config = Config::new(&config);

        assert_eq!(config.socket, socket);
        assert_eq!(config.containers.len(), 1);

        let container = &config.containers[name];
        assert_eq!(tag, container.tag);

        let build_config = &container.build;
        assert_eq!(dockerfile, build_config.dockerfile);
        assert_eq!(context, build_config.context);

        let run_config = &container.run;
        assert!(run_config.args.contains(&"i".to_string()));
        assert!(run_config.args.contains(&"rm".to_string()));
    }
}
