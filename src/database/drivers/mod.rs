//! Database driver abstraction

pub mod sqlite3;

use self::sqlite3::SQLite3Connection;
use crate::config::DatabaseConnectionConfig;

/// Trait for a database driver
pub trait DbDriver {
    fn new(config: &DatabaseConnectionConfig) -> Self;
    fn schema_create_key_value(&self, namespace: Option<&str>, store_name: &str);
    fn key_value_get(
        &self,
        namespace: Option<&str>,
        store_name: &str,
        key: &[&str],
    ) -> Option<String>;
    fn key_value_set(
        &self,
        namespace: Option<&str>,
        store_name: &str,
        key: &[&str],
        value: Option<&str>,
    );
    fn key_value_list(
        &self,
        namespace: Option<&str>,
        store_name: &str,
        key: &[&str],
    ) -> Vec<String>;
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
    pub fn schema_create_key_value(&self, namespace: Option<&str>, name: &str) {
        match self {
            DbConnection::SQLite3(connection) => {
                connection.schema_create_key_value(namespace, name)
            }
        }
    }

    /// Gets the value for a key
    pub fn key_value_get(
        &self,
        namespace: Option<&str>,
        store_name: &str,
        key: &[&str],
    ) -> Option<String> {
        match self {
            DbConnection::SQLite3(connection) => {
                connection.key_value_get(namespace, store_name, key)
            }
        }
    }

    /// Sets the value on a key
    pub fn key_value_set(
        &self,
        namespace: Option<&str>,
        store_name: &str,
        key: &[&str],
        value: Option<&str>,
    ) {
        match self {
            DbConnection::SQLite3(connection) => {
                connection.key_value_set(namespace, store_name, key, value)
            }
        }
    }

    /// Lists the child keys of a key
    pub fn key_value_list(
        &self,
        namespace: Option<&str>,
        store_name: &str,
        key: &[&str],
    ) -> Vec<String> {
        match self {
            DbConnection::SQLite3(connection) => {
                connection.key_value_list(namespace, store_name, key)
            }
        }
    }
}
