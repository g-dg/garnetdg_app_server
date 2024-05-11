use std::time::Duration;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    config::{DatabaseSchemaConfig, DatastoreConfig},
    database::{drivers::DbConnection, models::datastore::DatastoreValueMeta, DbSchema},
};

pub struct DatastoreDatabaseConfig {
    pub namespace: Option<String>,
    pub store_name: String,
    pub keep_history: bool,
    pub max_age: Option<Duration>,
    pub max_entries: Option<u64>,
}

impl DatastoreDatabaseConfig {
    pub fn new(
        store_name: &str,
        schema_config: &DatabaseSchemaConfig,
        datastore_config: &DatastoreConfig,
    ) -> Self {
        Self {
            namespace: schema_config.table_prefix.clone(),
            store_name: String::from(store_name),
            keep_history: datastore_config.keep_history,
            max_age: datastore_config.history_max_age.map(Duration::from_secs),
            max_entries: datastore_config.history_max_entries,
        }
    }
}

impl DbSchema {
    /// Creates a datastore's tables
    pub fn datastore_create(&self, store_name: &str, datastore_config: &DatastoreConfig) {
        self.connection
            .datastore_create(&DatastoreDatabaseConfig::new(
                store_name,
                &self.config,
                datastore_config,
            ))
    }

    /// Gets the metadata for the current value of a datastore key
    pub fn datastore_get_current(
        &self,
        store_name: &str,
        datastore_config: &DatastoreConfig,
        path: &[&str],
    ) -> Option<DatastoreValueMeta> {
        self.connection.datastore_get_current(
            &DatastoreDatabaseConfig::new(store_name, &self.config, datastore_config),
            path,
        )
    }

    /// Gets the metadata for the current and previous values of a datastore key
    pub fn datastore_get_history(
        &self,
        store_name: &str,
        datastore_config: &DatastoreConfig,
        path: &[&str],
        last_change_id: Option<Uuid>,
    ) -> Vec<DatastoreValueMeta> {
        self.connection.datastore_get_history(
            &DatastoreDatabaseConfig::new(store_name, &self.config, datastore_config),
            path,
            last_change_id,
        )
    }

    /// Gets the value of a change id
    pub fn datastore_get_value(
        &self,
        store_name: &str,
        datastore_config: &DatastoreConfig,
        change_id: Uuid,
    ) -> Option<String> {
        self.connection.datastore_get_value(
            &DatastoreDatabaseConfig::new(store_name, &self.config, datastore_config),
            change_id,
        )
    }

    /// Lists child keys of a key that have values
    pub fn datastore_list(
        &self,
        store_name: &str,
        datastore_config: &DatastoreConfig,
        path: &[&str],
    ) -> Vec<String> {
        self.connection.datastore_list(
            &DatastoreDatabaseConfig::new(store_name, &self.config, datastore_config),
            path,
        )
    }

    /// Sets a value to a datastore key
    pub fn datastore_set(
        &self,
        store_name: &str,
        datastore_config: &DatastoreConfig,
        path: &[&str],
        value: Option<&str>,
    ) -> Uuid {
        self.connection.datastore_set(
            &DatastoreDatabaseConfig::new(store_name, &self.config, datastore_config),
            path,
            value,
        )
    }
}

impl DbConnection {
    /// Creates a datastore's tables
    pub fn datastore_create(&self, config: &DatastoreDatabaseConfig) {
        match self {
            DbConnection::SQLite3(connection) => connection.datastore_create(config),
        }
    }

    /// Gets the metadata for the current value of a datastore key
    pub fn datastore_get_current(
        &self,
        config: &DatastoreDatabaseConfig,
        path: &[&str],
    ) -> Option<DatastoreValueMeta> {
        match self {
            DbConnection::SQLite3(connection) => connection.datastore_get_current(config, path),
        }
    }

    /// Gets the metadata for the current and previous values of a datastore key
    pub fn datastore_get_history(
        &self,
        config: &DatastoreDatabaseConfig,
        path: &[&str],
        last_change_id: Option<Uuid>,
    ) -> Vec<DatastoreValueMeta> {
        match self {
            DbConnection::SQLite3(connection) => {
                connection.datastore_get_history(config, path, last_change_id)
            }
        }
    }

    /// Gets the value of a change id
    pub fn datastore_get_value(
        &self,
        config: &DatastoreDatabaseConfig,
        change_id: Uuid,
    ) -> Option<String> {
        match self {
            DbConnection::SQLite3(connection) => connection.datastore_get_value(config, change_id),
        }
    }

    /// Lists child keys of a key that have values
    pub fn datastore_list(&self, config: &DatastoreDatabaseConfig, path: &[&str]) -> Vec<String> {
        match self {
            DbConnection::SQLite3(connection) => connection.datastore_list(config, path),
        }
    }

    /// Sets a value to a datastore key
    pub fn datastore_set(
        &self,
        config: &DatastoreDatabaseConfig,
        path: &[&str],
        value: Option<&str>,
    ) -> Uuid {
        match self {
            DbConnection::SQLite3(connection) => connection.datastore_set(config, path, value),
        }
    }
}
