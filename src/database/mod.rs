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

    /// Gets the value of the provided key
    pub fn key_value_get(&self, key: &Vec<&str>) -> Option<String> {
        self.connection.key_value_get(&self.config, key)
    }

    /// Sets the value of the provided key
    pub fn key_value_set(&self, key: &Vec<&str>, value: &str) {
        self.connection.key_value_set(&self.config, key, value)
    }
}
