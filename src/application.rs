use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    pin::Pin,
};

use crate::{
    auth::Auth,
    config::{Config, MessageQueueConfig, RoutePermissions},
    database::DbSchema,
};

pub struct Application {
    pub app_data: AppData,
    shutdown_queue: VecDeque<ShutdownFunction>,
}

#[derive(Clone)]
pub struct AppData {
    pub database_schemas: HashMap<String, DbSchema>,
    pub auth: Option<Auth>,
}

impl Application {
    pub async fn build(config: &Config) -> Self {
        let mut shutdown_queue = VecDeque::new();

        // connect to databases
        let database_schemas = DbSchema::connect_all(&config.databases);

        // set up authentication if configured
        let auth = match &config.authentication {
            Some(auth_config) => Some(Auth::new(
                auth_config,
                database_schemas
                    .get(&auth_config.database)
                    .expect("Authentication database schema not found")
                    .clone(),
            )),
            None => None,
        };

        let database_schemas = database::connect_schemas(&config.databases);


        Self {
            app_data: AppData {
                database_schemas,
                auth,
            },
            shutdown_queue,
        }
    }

    pub async fn stop(mut self) {
        while let Some(f) = self.shutdown_queue.pop_front() {
            match f {
                ShutdownFunction::Closure(f) => f(),
                ShutdownFunction::Future(f) => f.await,
            }
        }
    }
}

enum ShutdownFunction {
    Closure(Box<dyn FnOnce() -> ()>),
    Future(Pin<Box<dyn Future<Output = ()>>>),
}

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
