//! Key-value database operations

use super::SQLite3Connection;

use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{named_params, OptionalExtension};

impl SQLite3Connection {
    pub fn schema_create_key_value(&self, table_prefix: Option<&str>, store_name: &str) {
        let table_prefix = Self::get_table_prefix(table_prefix, Some(store_name));
        let conn = self
            .pool
            .get()
            .expect("Could not get database connection for key-value table creation");
        conn.execute_batch(&format!(
            "CREATE TABLE IF NOT EXISTS \"{0}key_values\" ( \
                \"id\" INTEGER PRIMARY KEY NOT NULL, \
                \"parent_id\" INTEGER REFERENCES \"{0}key_values\" (\"id\"), \
                \"key\" TEXT NOT NULL, \
                \"value\" TEXT \
            ); \
            CREATE UNIQUE INDEX IF NOT EXISTS \"{0}index_key_values__ifnull_parent_id__key\" ON \"{0}key_values\" (IFNULL(\"parent_id\", 0), \"key\"); \
            CREATE INDEX IF NOT EXISTS \"{0}index_key_values__parent_id__key\" ON \"{0}key_values\" (\"parent_id\", \"key\");",
            table_prefix))
        .expect(&format!("An error occurred while creating table {}key_values", table_prefix));
    }

    pub fn key_value_get(
        &self,
        table_prefix: Option<&str>,
        store_name: &str,
        key: &[&str],
    ) -> Option<String> {
        let conn = self
            .pool
            .get()
            .expect("Could not get database connection from connection pool");

        let table_prefix = Self::get_table_prefix(table_prefix, Some(store_name));

        fn get_value_recursive(
            conn: &PooledConnection<SqliteConnectionManager>,
            current_id: Option<i64>,
            key: &[&str],
            table_prefix: &str,
        ) -> Option<String> {
            if key.len() > 0 {
                // get child id
                let mut stmt = conn
                    .prepare_cached(&format!(
                        "SELECT \"id\" FROM \"{}key_values\" WHERE \"id\" = :id AND \"key\" = :key;",
                        table_prefix
                    ))
                    .expect("Error occurred while preparing SQL statement");
                let new_id: Option<i64> = stmt
                    .query_row(named_params! {":id": current_id, ":key": key[0]}, |row| {
                        row.get(0)
                    })
                    .optional()
                    .expect("Error occurred while querying database");
                // if returned the child id, then recurse
                if let Some(new_id) = new_id {
                    get_value_recursive(conn, Some(new_id), key, table_prefix)
                } else {
                    None
                }
            } else {
                // get value from id
                let mut stmt = conn
                    .prepare_cached(&format!(
                        "SELECT \"value\" FROM \"{}key_values\" WHERE \"id\" = :id",
                        table_prefix
                    ))
                    .expect("Error occurred while preparing SQL statement");
                stmt.query_row(named_params! {":id": current_id}, |row| row.get(0))
                    .optional()
                    .expect("Error occurred while querying database")
            }
        }

        get_value_recursive(&conn, None, key, &table_prefix)
    }

    pub fn key_value_set(
        &self,
        table_prefix: Option<&str>,
        store_name: &str,
        key: &[&str],
        value: &str,
    ) {
        let conn = self
            .pool
            .get()
            .expect("Could not get database connection from connection pool");

        todo!()
    }
}
