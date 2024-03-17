use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::{
    drivers::DbConnection,
    models::data_store::{DataStoreTreeNode, DataStoreValue},
    DbSchema,
};

impl DbSchema {
    /// Creates the data-store schema
    pub fn schema_create_data_store(&self, store_name: &str) {
        self.connection.schema_create_data_store(
            self.config
                .table_prefix
                .as_ref()
                .and_then(|x| Some(x.as_str())),
            store_name,
        )
    }
}

impl DbConnection {
    /// Creates a data-store schema
    pub fn schema_create_data_store(&self, namespace: Option<&str>, name: &str) {
        match self {
            DbConnection::SQLite3(connection) => {
                connection.schema_create_data_store(namespace, name)
            }
        }
    }
}
