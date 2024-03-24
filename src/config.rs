//! Configuration parser and models

use std::collections::HashMap;
use std::{env, io::ErrorKind};

use serde::{Deserialize, Serialize};
use tokio::fs;

/// Object containing the application config
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Basic server configuration
    #[serde(default = "default_server")]
    pub server: ServerConfig,

    /// Database configuration
    #[serde(default = "default_database")]
    pub databases: DatabaseConfig,

    /// Datastore configuration
    #[serde(default = "default_data_stores")]
    pub data_stores: HashMap<String, DataStoreConfig>,

    /// Authentication configuration
    pub authentication: Option<AuthenticationConfig>,

    /// Route configuration
    #[serde(default = "default_routes")]
    pub routes: HashMap<String, RouteConfig>,
}

impl Config {
    /// Loads the application config from the provided file
    pub async fn load_file(filename: &str) -> Config {
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

        Self::load(&contents)
    }

    /// Loads the application config from the provided string.
    /// Uses first and second command line arguments as port and host overrides.
    pub fn load(config_json: &str) -> Config {
        let mut config: Config =
            serde_json::from_str(config_json).expect("Failed to parse config file");

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
    /// Server host (0.0.0.0 for all hosts, 127.0.0.1 for localhost)
    pub host: String,

    /// Server port
    pub port: u16,
}

/// Database configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    /// Database connection configuration
    pub connections: HashMap<String, DatabaseConnectionConfig>,

    /// Database schema configuration
    pub schemas: HashMap<String, DatabaseSchemaConfig>,
}

/// Database connection configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "driver")]
pub enum DatabaseConnectionConfig {
    /// Sqlite3 database driver
    SQLite3 { database: String },
}

/// Database schema configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseSchemaConfig {
    /// Database connection for the schema to use
    pub connection: String,

    /// Optional table prefix for this schema
    pub table_prefix: Option<String>,
}

/// Data store configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataStoreConfig {
    /// Database schema for persisting data.
    /// If not set, data is not persisted to a database and will be lost at application shutdown.
    pub database_schema: Option<String>,

    /// Whether to keep history.
    /// If not set, the default (false) is used.
    pub keep_history: Option<bool>,

    /// Maximum age in seconds of previous values to keep for each key.
    /// Age is determined by the time the value was set.
    pub history_max_age: Option<u64>,

    /// Maximum number of previous values to keep for each key.
    /// Set to 0 to not keep history entries.
    pub history_max_entries: Option<u64>,
}

/// Authentication configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthenticationConfig {
    /// Database schema to use for authentication
    pub database_schema: String,

    /// Authentication default setup
    pub defaults: AuthenticationDefaultsConfig,
}

/// Authentication defaults configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthenticationDefaultsConfig {
    /// List of default roles
    pub roles: Vec<String>,

    /// Hashmap of default users
    pub users: HashMap<String, AuthenticationDefaultUserConfig>,
}

/// Authentication default user configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthenticationDefaultUserConfig {
    /// Default user password
    pub default_password: Option<String>,

    /// List of default user roles
    pub roles: Vec<String>,
}

/// Route configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "handler")]
pub enum RouteConfig {
    /// Simple redirect handler
    Redirect { redirect_target: String },

    /// Basic file handler
    File {
        #[serde(default = "default_readonly_permissions")]
        permissions: RoutePermissions,
        server_file_path: String,
        index_file: Option<String>,
    },

    /// Datastore
    Data {
        permissions: RoutePermissions,
        data_store: Option<String>,
    },

    /// Authentication endpoints
    Auth,

    /// Authentication management endpoints
    AuthAdmin { permissions: RoutePermissions },
}

/// Route permissions configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoutePermissions {
    /// Read permissions
    pub read: RoutePermissionValue,

    /// Write permissions
    pub write: RoutePermissionValue,
}

/// Route permission value
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RoutePermissionValue {
    /// Completely allow or deny permissions
    Global(bool),

    /// Only allow the listed roles
    Roles(Vec<String>),
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

fn default_data_stores() -> HashMap<String, DataStoreConfig> {
    HashMap::new()
}

/// Creates the default route configuration
fn default_routes() -> HashMap<String, RouteConfig> {
    HashMap::from([(
        String::from("/"),
        RouteConfig::File {
            permissions: default_readonly_permissions(),
            server_file_path: String::from("./client/"),
            index_file: Some(String::from("index.html")),
        },
    )])
}

/// Defines read-only route permission for default routes
fn default_readonly_permissions() -> RoutePermissions {
    RoutePermissions {
        read: RoutePermissionValue::Global(true),
        write: RoutePermissionValue::Global(false),
    }
}
