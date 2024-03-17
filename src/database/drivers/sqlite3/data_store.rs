//! Key-value database operations

use super::SQLite3Connection;

use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{named_params, OptionalExtension};

impl SQLite3Connection {
    pub fn schema_create_data_store(&self, namespace: Option<&str>, store_name: &str) {
        let table_prefix = Self::get_table_prefix(namespace, Some(store_name));
        let conn = self
            .pool
            .get()
            .expect("Could not get database connection for key-value table creation");

        conn.execute_batch(&format!(
            "{0}",
            table_prefix))
        .expect(&format!("An error occurred while creating database tables \"{0}\"", table_prefix));
    }

    
}
