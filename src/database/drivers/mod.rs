pub mod sqlite3;

use self::sqlite3::SQLite3Connection;
use crate::config::{DatabaseConnectionConfig, DatabaseSchemaConfig};

pub trait DbDriver {
    fn new(config: &DatabaseConnectionConfig) -> Self;
    fn key_value_get(&self, schema_config: &DatabaseSchemaConfig, key: &Vec<&str>) -> Option<String>;
    fn key_value_set(&self, schema_config: &DatabaseSchemaConfig, key: &Vec<&str>, value: &str);
}

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
    // gets the value for a key
    pub fn key_value_get(&self, schema_config: &DatabaseSchemaConfig, key: &Vec<&str>) -> Option<String> {
        match self {
            DbConnection::SQLite3(connection) => connection.key_value_get(schema_config, key),
        }
    }

    // Sets the value on a key
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
    }
}
