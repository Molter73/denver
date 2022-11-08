use std::collections::HashMap;
use std::env;

use serde::Deserialize;

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename = "build")]
pub struct BuildConfig {
    pub dockerfile: Option<String>,
    pub context: String,
    pub build_args: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename = "run")]
pub struct RunConfig {
    pub args: Option<Vec<String>>,
    pub workspace: String,
    pub volumes: Option<Vec<String>>,
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
        let build_args = r#"
            test_arg: value
            test_arg2: value2
            "#;
        let args = r#"
        - i
        - rm"#;
        let workspace = "/some/path/";
        let volumes = r#"
        - /some/other/path:/path
        - /dev:/dev:ro"#;
        let tag = "quay.io/org/some:tag";
        let config = format!(
            r#"
socket: {}
containers:
  {}:
    build:
        dockerfile: {}
        context: {}
        build_args: {}
    run:
        args: {}
        workspace: {}
        volumes: {}
    tag: {}
        "#,
            socket, name, dockerfile, context, build_args, args, workspace, volumes, tag
        );

        let config = Config::new(&config);

        assert_eq!(config.socket, socket);
        assert_eq!(config.containers.len(), 1);

        let container = &config.containers[name];
        assert_eq!(tag, container.tag);

        let build_config = &container.build;
        assert_eq!(dockerfile, build_config.dockerfile.as_ref().unwrap());
        assert_eq!(context, build_config.context);
        for (k, v) in build_config.build_args.as_ref().unwrap() {
            assert!(build_args.contains(k));
            assert!(build_args.contains(v));
        }

        let run_config = &container.run;
        let run_args = run_config.args.as_ref().unwrap();
        for arg in run_args {
            assert!(args.contains(arg));
        }
        assert_eq!(workspace, run_config.workspace);

        let run_volumes = run_config.volumes.as_ref().unwrap();
        assert_eq!(2, run_volumes.len());
        for volume in run_volumes {
            assert!(volumes.contains(volume));
        }
    }
}
