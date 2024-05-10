//! Database module

pub mod api;
pub mod drivers;
pub mod models;

use std::collections::HashMap;

use self::drivers::DbConnection;
use crate::config::{DatabaseConfig, DatabaseConnectionConfig, DatabaseSchemaConfig};

/// Database schema
#[derive(Clone)]
pub struct DbSchema {
    config: DatabaseSchemaConfig,
    connection: DbConnection,
}

impl DbSchema {
    /// Connects to all databases specified in the provided config
    pub fn connect_all(config: &DatabaseConfig) -> HashMap<String, DbSchema> {
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
                                .unwrap_or_else(|| panic!("Database connection \"{}\" is not defined in the configuration. (Referenced by schema \"{}\")", schema_config.connection, schema_name))
                                .clone()
                        )
                }
            )
        }).collect();

        schemas
    }

    /// Creates an in-memory database schema.
    /// Uses SQLite3 in-memory database as the backend
    pub fn new_memory() -> Self {
        let connection_config = DatabaseConnectionConfig::SQLite3 {
            database: String::from(":memory:"),
        };
        let connection = DbConnection::new(&connection_config);

        let schema_config = DatabaseSchemaConfig {
            connection: String::new(),
            table_prefix: None,
        };

        DbSchema {
            config: schema_config,
            connection,
        }
    }
}
