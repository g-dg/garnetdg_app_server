//! Key-value database operations

use super::SQLite3Connection;

use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{named_params, OptionalExtension};

impl SQLite3Connection {
    pub fn schema_create_key_value(&self, namespace: Option<&str>, store_name: &str) {
        let table_prefix = Self::get_table_prefix(namespace, Some(store_name));
        let conn = self
            .pool
            .get()
            .expect("Could not get database connection for key-value table creation");

        conn.execute_batch(&format!(
            "CREATE TABLE IF NOT EXISTS \"{0}key_value_tree\" ( \
                \"id\" INTEGER PRIMARY KEY NOT NULL, \
                \"parent_id\" INTEGER REFERENCES \"{0}key_value_tree\" (\"id\"), \
                \"key\" TEXT NOT NULL \
            ); \
            CREATE UNIQUE INDEX IF NOT EXISTS \"{0}index_key_value_tree__ifnull_parent_id__key\" ON \"{0}key_value_tree\" (IFNULL(\"parent_id\", 0), \"key\"); \
            CREATE INDEX IF NOT EXISTS \"{0}index_key_value_tree__parent_id__key\" ON \"{0}key_value_tree\" (\"parent_id\", \"key\"); \
            \
            CREATE TABLE IF NOT EXISTS \"{0}key_value\" ( \
                \"id\" INTEGER PRIMARY KEY NOT NULL, \
                \"tree_node_id\" INTEGER REFERENCES \"{0}key_value_tree\" (\"id\"), \
                \"value\" TEXT \
            ); \
            CREATE UNIQUE INDEX IF NOT EXISTS \"{0}index_key_value__ifnull_tree_node_id\" ON \"{0}key_value\" (IFNULL(\"tree_node_id\", 0)); \
            CREATE INDEX IF NOT EXISTS \"{0}index_key_value__tree_node_id\" ON \"{0}key_value\" (\"tree_node_id\");",
            table_prefix))
        .expect(&format!("An error occurred while creating database tables \"{0}key_value_tree\", \"{0}key_value_values\"", table_prefix));
    }

    pub fn key_value_get(
        &self,
        namespace: Option<&str>,
        store_name: &str,
        key: &[&str],
    ) -> Option<String> {
        let conn = self
            .pool
            .get()
            .expect("Could not get database connection from connection pool");

        let table_prefix = Self::get_table_prefix(namespace, Some(store_name));

        todo!();
    }

    pub fn key_value_set(
        &self,
        namespace: Option<&str>,
        store_name: &str,
        key: &[&str],
        value: Option<&str>,
    ) {
        let conn = self
            .pool
            .get()
            .expect("Could not get database connection from connection pool");

        let table_prefix = Self::get_table_prefix(namespace, Some(store_name));

        todo!();
    }

    pub fn key_value_list(
        &self,
        namespace: Option<&str>,
        store_name: &str,
        key: &[&str],
    ) -> Vec<String> {
        let conn = self
            .pool
            .get()
            .expect("Could not get database connection from connection pool");

        let table_prefix = Self::get_table_prefix(namespace, Some(store_name));

        todo!();
    }

    /// Gets the node id from a path.
    /// If `create` is set to true, creates all nodes up to that path
    fn get_node_id(
        conn: &PooledConnection<SqliteConnectionManager>,
        table_prefix: &str,
        key: &[&str],
        create: bool,
    ) -> Option<i64> {
        Self::get_node_id_recursive(conn, table_prefix, key, None, create)
    }

    /// Recursively gets the node id from the next part of the path.
    fn get_node_id_recursive(
        conn: &PooledConnection<SqliteConnectionManager>,
        table_prefix: &str,
        key: &[&str],
        parent_id: Option<i64>,
        create: bool,
    ) -> Option<i64> {
        if key.len() > 0 {
            let id_result: Option<i64> = conn
            .prepare_cached(&format!(
                "SELECT \"id\" FROM \"{0}key_value_tree\" WHERE \"parent_id\" = :id AND \"key\" = :key;",
                table_prefix
            ))
            .expect("Error occurred while preparing database query")
            .query_row(named_params! {":id": parent_id, ":key": key[0]}, |row| {
                row.get(0)
            })
            .optional()
            .expect("Error occurred while querying database");

            if let Some(id) = id_result {
                Self::get_node_id_recursive(conn, table_prefix, &key[1..], Some(id), create)
            } else {
                // if create flag is set and the node is not found, create a new node
                if create {
                    conn
                    .prepare_cached(&format!("INSERT INTO \"{0}key_value_tree\" (\"parent_id\", \"key\") VALUES (:id, :key);", table_prefix))
                    .expect("Error occurred while preparing database query")
                    .execute(named_params! {":id": parent_id, ":key": key[0]})
                    .expect("Error occurred while updating database");
                    let created_id = conn.last_insert_rowid();

                    Self::get_node_id_recursive(
                        conn,
                        table_prefix,
                        &key[1..],
                        Some(created_id),
                        create,
                    )
                } else {
                    None
                }
            }
        } else {
            parent_id
        }
    }
}
