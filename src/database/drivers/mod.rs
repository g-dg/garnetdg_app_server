//! Database driver abstraction

pub mod sqlite3;

use self::sqlite3::SQLite3Connection;
use crate::config::DatabaseConnectionConfig;

/// Trait for a database driver
pub trait DbDriver {
    fn new(config: &DatabaseConnectionConfig) -> Self;
    fn schema_create_key_value(&self, table_prefix: Option<&str>, store_name: &str);
    fn key_value_get(
        &self,
        table_prefix: Option<&str>,
        store_name: &str,
        key: &[&str],
    ) -> Option<String>;
    fn key_value_set(
        &self,
        table_prefix: Option<&str>,
        store_name: &str,
        key: &[&str],
        value: &str,
    );
}

/// A database connection
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
    /// Connects to the database specified in the provided database connection config
    pub fn new(config: &DatabaseConnectionConfig) -> DbConnection {
        match config {
            DatabaseConnectionConfig::SQLite3 { database: _ } => {
                DbConnection::SQLite3(SQLite3Connection::new(config))
            }
        }
    }

    /// Creates a key-value schema
    pub fn schema_create_key_value(&self, table_prefix: Option<&str>, name: &str) {
        match self {
            DbConnection::SQLite3(connection) => {
                connection.schema_create_key_value(table_prefix, name)
            }
        }
    }

    /// Gets the value for a key
    pub fn key_value_get(
        &self,
        table_prefix: Option<&str>,
        store_name: &str,
        key: &[&str],
    ) -> Option<String> {
        match self {
            DbConnection::SQLite3(connection) => {
                connection.key_value_get(table_prefix, store_name, key)
            }
        }
    }

    /// Sets the value on a key
    pub fn key_value_set(
        &self,
        table_prefix: Option<&str>,
        store_name: &str,
        key: &[&str],
        value: &str,
    ) {
        match self {
            DbConnection::SQLite3(connection) => {
                connection.key_value_set(table_prefix, store_name, key, value)
            }
        }
    }
}
