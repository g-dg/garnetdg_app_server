pub mod drivers;

use std::collections::HashMap;

use self::drivers::DbConnection;
use crate::config::{DatabaseConfig, DatabaseSchemaConfig};

#[derive(Clone)]
pub struct DbSchema {
    config: DatabaseSchemaConfig,
    connection: DbConnection,
}

impl DbSchema {
    pub fn key_value_get(&self, key: &Vec<&str>) -> String {
        String::new()
    }
    pub fn key_value_set(&self, key: &Vec<&str>, value: &str) {}
}

pub fn connect_schemas<'a>(config: &'a DatabaseConfig) -> HashMap<String, DbSchema> {
    // create database connections
    let connections: HashMap<&str, DbConnection> = config
        .connections
        .iter()
        .map(|(connection_name, connection_config)| {
            (
                connection_name.as_str(),
                drivers::connect_database(connection_config),
            )
        })
        .collect();

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
