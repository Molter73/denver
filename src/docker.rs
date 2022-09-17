use futures::StreamExt;
use serde_json::Value;
use shiplift::{BuildOptions, Docker};

use crate::config::Config;

pub async fn run(config: Config) {
    let docker = Docker::unix(&config.socket);

    let container = &config.containers["falco-fedora"];
    let options = BuildOptions::builder(&container.context)
        .dockerfile(&container.dockerfile)
        .tag(&container.tag)
        .build();

    let mut stream = docker.images().build(&options);
    while let Some(build_result) = stream.next().await {
        match build_result {
            Ok(output) => {
                let stream = &output["stream"];
                match stream {
                    Value::String(log) if !log.trim().is_empty() => println!("{}", log.trim()),
                    Value::String(_) => {}
                    Value::Null => {}
                    _ => println!("{:?}", stream),
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
