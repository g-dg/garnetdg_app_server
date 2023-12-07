pub mod sqlite3;

use self::sqlite3::SQLite3Connection;
use crate::config::model::{DatabaseConnectionConfig, DatabaseSchemaConfig};

pub trait DbDriver {
    fn new(config: &DatabaseConnectionConfig) -> Self;
    fn key_value_get(&self, schema_config: &DatabaseSchemaConfig, key: &Vec<&str>) -> String;
    fn key_value_set(&self, schema_config: &DatabaseSchemaConfig, key: &Vec<&str>, value: &str);
}

#[derive(Debug)]
pub enum DbConnection {
    SQLite3(SQLite3Connection),
}
impl Clone for DbConnection {
    fn clone(&self) -> Self {
        match self {
            Self::SQLite3(driver) => Self::SQLite3(driver.clone()),
        }
    }
}

impl DbConnection {
    pub fn key_value_get(&self, schema_config: &DatabaseSchemaConfig, key: &Vec<&str>) -> String {
        match self {
            DbConnection::SQLite3(connection) => connection.key_value_get(schema_config, key),
        }
    }

    pub fn key_value_set(
        &self,
        schema_config: &DatabaseSchemaConfig,
        key: &Vec<&str>,
        value: &str,
    ) {
        match self {
            DbConnection::SQLite3(connection) => {
                connection.key_value_set(schema_config, key, value)
            }
        }
    }
}

pub fn connect_database(config: &DatabaseConnectionConfig) -> DbConnection {
    match config {
        DatabaseConnectionConfig::SQLite3 { database: _ } => {
            DbConnection::SQLite3(SQLite3Connection::new(config))
        }
        _ => panic!("Unknown database driver: {:?}", config),
    }
}
