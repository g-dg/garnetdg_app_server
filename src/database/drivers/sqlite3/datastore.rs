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
            "
CREATE TABLE IF NOT EXISTS \"{0}datastore_tree\" (
    \"id\" INTEGER PRIMARY KEY NOT NULL,
    \"parent_id\" INTEGER REFERENCES \"{0}datastore_tree\" (\"id\"),
    \"key\" TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS \"{0}index_datastore_tree__ifnull_parent_id__key\" ON \"{0}datastore_tree\" (IFNULL(\"parent_id\", 0), \"key\");
CREATE INDEX IF NOT EXISTS \"{0}index_datastore_tree__parent_id__key\" ON \"{0}datastore_tree\" (\"parent_id\", \"key\");

CREATE TABLE IF NOT EXISTS \"{0}datastore_values\" (
    \"id\" INTEGER PRIMARY KEY NOT NULL,
    \"tree_node_id\" INTEGER REFERENCES \"{0}datastore_tree\",
    \"change_id\" TEXT NOT NULL UNIQUE,
    \"timestamp\" TEXT NOT NULL,
    \"value\" TEXT
);
CREATE INDEX IF NOT EXISTS \"{0}index_datastore_values__tree_node_id\" ON \"{0}datastore_values\" (\"tree_node_id\");
CREATE INDEX IF NOT EXISTS \"{0}index_datastore_values__timestamp\" ON \"{0}datastore_values\" (\"timestamp\");
            ",
            table_prefix))
        .expect(&format!("An error occurred while creating database tables \"{0}\"", table_prefix));
    }
}
