use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_server")]
    pub server: ServerConfig,
    #[serde(default = "default_database")]
    pub databases: DatabaseConfig,
    pub authentication: Option<AuthenticationConfig>,
    #[serde(default = "default_route")]
    pub routes: HashMap<String, RouteConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseConfig {
    pub connections: HashMap<String, DatabaseConnectionConfig>,
    pub schemas: HashMap<String, DatabaseSchemaConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "driver")]
pub enum DatabaseConnectionConfig {
    SQLite3 { database: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseSchemaConfig {
    pub connection: String,
    pub table_prefix: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticationConfig {
    pub database: String,
    pub defaults: AuthenticationDefaultsConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticationDefaultsConfig {
    pub roles: Vec<String>,
    pub users: HashMap<String, AuthenticationDefaultUserConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticationDefaultUserConfig {
    pub default_password: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct RoutePermissions {
    pub read: RoutePermissionState,
    pub write: RoutePermissionState,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RoutePermissionState {
    Global(bool),
    Groups(Vec<String>),
}

fn default_server() -> ServerConfig {
    ServerConfig {
        host: String::from("0.0.0.0"),
        port: 8080,
    }
}

fn default_database() -> DatabaseConfig {
    DatabaseConfig {
        connections: HashMap::new(),
        schemas: HashMap::new(),
    }
}

fn default_route() -> HashMap<String, RouteConfig> {
    let mut routes = HashMap::<String, RouteConfig>::new();
    routes.insert(
        String::from("/"),
        RouteConfig::File {
            permissions: default_readonly_permissions(),
            server_file_path: String::from("./"),
            index_file: Some(String::from("index.html")),
        },
    );
    routes
}

fn default_readonly_permissions() -> RoutePermissions {
    RoutePermissions {
        read: RoutePermissionState::Global(true),
        write: RoutePermissionState::Global(false),
    }
}
