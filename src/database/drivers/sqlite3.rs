use super::DbDriver;
use crate::config::{DatabaseConnectionConfig, DatabaseSchemaConfig};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub struct SQLite3Connection {
    pool: Pool<SqliteConnectionManager>,
}

impl DbDriver for SQLite3Connection {
    fn new(config: &DatabaseConnectionConfig) -> Self {
        match config {
            DatabaseConnectionConfig::SQLite3 { database } => {
                let manager = SqliteConnectionManager::file(database);
                let pool = r2d2::Pool::new(manager).unwrap();
                Self { pool }
            }
        }
    }

    fn key_value_get(
        &self,
        schema_config: &DatabaseSchemaConfig,
        key: &Vec<&str>,
    ) -> Option<String> {
        todo!()
    }

    fn key_value_set(
        &self,
        schema_config: &DatabaseSchemaConfig,
        key: &Vec<&str>,
        value: &str,
    ) {
        todo!()
    }
}

impl Clone for SQLite3Connection {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
