pub mod model;

use std::{io::ErrorKind, env};

use self::model::Config;
use tokio::fs;

pub async fn load_config(filename: &str) -> Config {
    let read_result = fs::read_to_string(filename).await;
    let contents = match read_result {
        Ok(value) => {
            if value.len() > 0 {
                value
            } else {
                String::from("{}")
            }
        },
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                String::from("{}")
            } else {
                panic!("Error occurred while reading config file: {:?}", err)
            }
        }
    };
    let mut config: Config = serde_json::from_str(contents.as_str()).unwrap();

    let args: Vec<String> = env::args().collect();
    let port_arg: Option<u16> = args.get(1).and_then(|x|x.parse().ok());
    let host_arg = args.get(2).and_then(|x|Some(x.to_string()));

    config.server.host = host_arg.unwrap_or(config.server.host);
    config.server.port = port_arg.unwrap_or(config.server.port);

    config
}
