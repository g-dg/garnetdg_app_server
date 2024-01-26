//! Database module

pub mod drivers;

use std::collections::HashMap;

use self::drivers::DbConnection;
use crate::config::{DatabaseConfig, DatabaseSchemaConfig};

/// Database schema
#[derive(Clone)]
pub struct DbSchema {
    config: DatabaseSchemaConfig,
    connection: DbConnection,
}

impl DbSchema {
    /// Connects to all databases specified in the provided config
    pub fn connect_all<'a>(config: &'a DatabaseConfig) -> HashMap<String, DbSchema> {
        // create database connections
        let connections: HashMap<&str, DbConnection> = config
            .connections
            .iter()
            .map(|(connection_name, connection_config)| {
                (
                    connection_name.as_str(),
                    DbConnection::new(connection_config),
                )
            })
            .collect();

        // map database connections to their schemas
        let schemas: HashMap<String, DbSchema> = config.schemas.iter().map(|(schema_name, schema_config)| {
            (
                schema_name.clone(),
                DbSchema {
                    config: schema_config.clone(),
                    connection:
                        (
                            connections
                                .get(schema_config.connection.as_str())
                                .expect(&format!("Database connection \"{}\" is not defined in the configuration. (Referenced by schema \"{}\")", schema_config.connection, schema_name))
                                .clone()
                        )
                }
            )
        }).collect();

        schemas
    }

    pub fn schema_create_key_value(&self, store_name: &str) {
        self.connection.schema_create_key_value(
            self.config
                .table_prefix
                .as_ref()
                .and_then(|x| Some(x.as_str())),
            store_name,
        )
    }

    /// Gets the value of the provided key
    pub fn key_value_get(&self, store_name: &str, key: &[&str]) -> Option<String> {
        self.connection.key_value_get(
            self.config
                .table_prefix
                .as_ref()
                .and_then(|x| Some(x.as_str())),
            store_name,
            key,
        )
    }

    /// Sets the value of the provided key
    pub fn key_value_set(&self, store_name: &str, key: &[&str], value: &str) {
        self.connection.key_value_set(
            self.config
                .table_prefix
                .as_ref()
                .and_then(|x| Some(x.as_str())),
            store_name,
            key,
            value,
        )
    }
}
