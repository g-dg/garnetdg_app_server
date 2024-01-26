//! Handles the application lifecycle

use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    pin::Pin,
};

use crate::{
    auth::Auth,
    config::{Config, RoutePermissions},
    database::DbSchema,
    key_value_store::KeyValueStore,
    message_queue::MessageQueue,
};

/// Represents the application
pub struct Application {
    /// App data
    pub app_data: AppData,
    /// Queue of functions to run at application shutdown
    shutdown_queue: VecDeque<ShutdownFunction>,
}

/// Contains the application data that is passed to each endpoint
#[derive(Clone)]
pub struct AppData {
    /// Hashmap of database schemas
    pub database_schemas: HashMap<String, DbSchema>,
    /// Authentication system
    pub auth: Option<Auth>,
}

impl Application {
    /// Builds the application
    pub async fn build(config: &Config) -> Self {
        let mut shutdown_queue = VecDeque::new();

        // connect to databases
        let database_schemas = DbSchema::connect_all(&config.databases);

        // set up authentication if configured
        let auth = match &config.authentication {
            Some(auth_config) => Some(Auth::new(
                auth_config,
                database_schemas
                    .get(&auth_config.database_schema)
                    .expect("Authentication database schema not found in config")
                    .clone(),
            )),
            None => None,
        };

        Self {
            app_data: AppData {
                database_schemas,
                auth,
            },
            shutdown_queue,
        }
    }

    /// Shuts down the application
    /// The application cannot be accessed after this is run
    pub async fn stop(mut self) {
        while let Some(f) = self.shutdown_queue.pop_front() {
            match f {
                ShutdownFunction::Closure(f) => f(),
                ShutdownFunction::Future(f) => f.await,
            }
        }
    }
}

/// Represents an application endpoint type
pub enum ApplicationEndpoint {
    Redirect {
        target: String,
    },
    File {
        permissions: RoutePermissions,
        server_file_path: String,
        index_file: Option<String>,
    },
    KeyValue {
        permissions: RoutePermissions,
        key_value: KeyValueStore<String>,
        database_schema: DbSchema,
    },
    MessageQueue {
        permissions: RoutePermissions,
        message_queue: MessageQueue<String>,
        database_schema: DbSchema,
    },
    Auth {
        database_schema: DbSchema,
    },
    AuthAdmin {
        permissions: RoutePermissions,
        database_schema: DbSchema,
    },
}

/// Enum for application shutdown functions
enum ShutdownFunction {
    Closure(Box<dyn FnOnce() -> ()>),
    Future(Pin<Box<dyn Future<Output = ()>>>),
}
