//! Configuration parser and models

use std::collections::HashMap;
use std::{env, io::ErrorKind};

use serde::{Deserialize, Serialize};
use tokio::fs;

/// Object containing the application config
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default = "default_server")]
    pub server: ServerConfig,
    #[serde(default = "default_database")]
    pub databases: DatabaseConfig,
    pub message_queues: HashMap<String, MessageQueueConfig>,
    pub authentication: Option<AuthenticationConfig>,
    #[serde(default = "default_route")]
    pub routes: HashMap<String, RouteConfig>,
}

impl Config {
    /// Loads the application config from the provided file
    pub async fn load(filename: &str) -> Config {
        //TODO: allow reading from string?
        let read_result = fs::read_to_string(filename).await;
        let contents = match read_result {
            Ok(value) => {
                if value.len() > 0 {
                    value
                } else {
                    String::from("{}")
                }
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    String::from("{}")
                } else {
                    panic!("Error occurred while reading config file: {:?}", err)
                }
            }
        };
        let mut config: Config =
            serde_json::from_str(contents.as_str()).expect("Failed to parse config file");

        let args: Vec<String> = env::args().collect();
        let port_arg: Option<u16> = args.get(1).and_then(|x| x.parse().ok());
        let host_arg = args.get(2).and_then(|x| Some(x.to_string()));

        config.server.host = host_arg.unwrap_or(config.server.host);
        config.server.port = port_arg.unwrap_or(config.server.port);

        config
    }
}

/// Server configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Database configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    pub connections: HashMap<String, DatabaseConnectionConfig>,
    pub schemas: HashMap<String, DatabaseSchemaConfig>,
}

/// Database connection configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "driver")]
pub enum DatabaseConnectionConfig {
    SQLite3 { database: String },
}

/// Database schema configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseSchemaConfig {
    pub connection: String,
    pub table_prefix: Option<String>,
}

/// Message queue configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageQueueConfig {
    pub database_schema: Option<String>,
    pub message_expiry: Option<u64>,
    pub message_limit: Option<u64>,
}

/// Authentication configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthenticationConfig {
    pub database: String,
    pub defaults: AuthenticationDefaultsConfig,
}

/// Authentication defaults configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthenticationDefaultsConfig {
    pub roles: Vec<String>,
    pub users: HashMap<String, AuthenticationDefaultUserConfig>,
}

/// Authentication default user configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthenticationDefaultUserConfig {
    pub default_password: Option<String>,
    pub roles: Vec<String>,
}

/// Route configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "handler")]
pub enum RouteConfig {
    Redirect {
        redirect_target: String,
    },

    File {
        #[serde(default = "default_readonly_permissions")]
        permissions: RoutePermissions,
        server_file_path: String,
        index_file: Option<String>,
    },

    KeyValue {
        permissions: RoutePermissions,
        database_schema: Option<String>,
    },

    MessageQueue {
        permissions: RoutePermissions,
        database_schema: Option<String>,
    },

    Auth,

    AuthAdmin {
        permissions: RoutePermissions,
    },
}

/// Route permissions configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoutePermissions {
    pub read: RoutePermissionValue,
    pub write: RoutePermissionValue,
}

/// Route permission value
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RoutePermissionValue {
    Global(bool),
    Groups(Vec<String>),
}

/// Creates the default server configuration
fn default_server() -> ServerConfig {
    ServerConfig {
        host: String::from("0.0.0.0"),
        port: 8080,
    }
}

/// Creates the default database configuration
fn default_database() -> DatabaseConfig {
    DatabaseConfig {
        connections: HashMap::new(),
        schemas: HashMap::new(),
    }
}

/// Creates the default route configuration
fn default_route() -> HashMap<String, RouteConfig> {
    let mut routes = HashMap::<String, RouteConfig>::new();
    routes.insert(
        String::from("/"),
        RouteConfig::File {
            permissions: default_readonly_permissions(),
            server_file_path: String::from("./client/"),
            index_file: Some(String::from("index.html")),
        },
    );
    routes
}

/// Defines read-only route permission for default routes
fn default_readonly_permissions() -> RoutePermissions {
    RoutePermissions {
        read: RoutePermissionValue::Global(true),
        write: RoutePermissionValue::Global(false),
    }
}
