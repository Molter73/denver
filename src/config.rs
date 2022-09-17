use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Container {
    pub dockerfile: String,
    pub context: String,
    pub tag: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Config {
    pub socket: String,
    pub containers: HashMap<String, Container>,
}

impl Config {
    pub fn new(config: &str) -> Self {
        serde_yaml::from_str(config).unwrap()
    }
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
        let tag = "quay.io/org/some:tag";
        let config = format!(
            r#"
socket: {}
containers:
  {}:
    dockerfile: {}
    context: {}
    tag: {}
        "#,
            socket, name, dockerfile, context, tag
        );

        let config: Config = serde_yaml::from_str(config.as_str()).unwrap();

        assert_eq!(config.socket, socket);
        assert_eq!(config.containers.len(), 1);

        let container = &config.containers[name];
        assert_eq!(dockerfile, container.dockerfile);
        assert_eq!(context, container.context);
        assert_eq!(tag, container.tag);
    }
}
