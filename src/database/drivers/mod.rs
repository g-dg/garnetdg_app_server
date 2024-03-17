//! Database driver abstraction

pub mod sqlite3;

use self::sqlite3::SQLite3Connection;
use crate::config::DatabaseConnectionConfig;

/// Trait for a database driver
pub trait DbDriver {
    fn new(config: &DatabaseConnectionConfig) -> Self;
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
}
