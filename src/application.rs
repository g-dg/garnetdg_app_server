use std::collections::HashMap;

use crate::{
    config::{Config, RoutePermissions},
    database::{self, DbSchema},
    message_queue::{self, MessageQueue},
};

pub struct Application {
    pub app_data: AppData,
}

impl Application {
    pub async fn build(config: &Config) -> Self {
        // connect to databases
        let database_schemas = database::connect_schemas(&config.databases);

        Self {
            app_data: AppData { database_schemas },
        }
    }

    pub async fn stop(self) {}
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

#[derive(Clone)]
pub struct AppData {
    pub database_schemas: HashMap<String, DbSchema>,
}
