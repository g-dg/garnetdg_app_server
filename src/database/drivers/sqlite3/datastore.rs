//! SQLite3 Datastore database driver

use std::time::Duration;

use crate::{
    config::DatastoreConfig,
    database::{api::datastore::DatastoreDatabaseConfig, models::datastore::DatastoreValueMeta},
};

use super::SQLite3Connection;

use chrono::{DateTime, Utc};
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{named_params, OptionalExtension};
use uuid::Uuid;

type DBConnection = PooledConnection<SqliteConnectionManager>;

impl SQLite3Connection {
    pub fn datastore_create(&self, config: &DatastoreDatabaseConfig) {
        let table_prefix = Self::datastore_get_table_prefix(config);
        let conn = self.get_connection();

        conn.execute_batch(&format!(
            "
CREATE TABLE IF NOT EXISTS \"{0}datastore_tree\" (
    \"id\" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    \"parent_id\" INTEGER REFERENCES \"{0}datastore_tree\" (\"id\"),
    \"key\" TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS \"{0}index_datastore_tree__ifnull_parent_id__key\" ON \"{0}datastore_tree\" (IFNULL(\"parent_id\", 0), \"key\");
CREATE INDEX IF NOT EXISTS \"{0}index_datastore_tree__parent_id__key\" ON \"{0}datastore_tree\" (\"parent_id\", \"key\");

CREATE TABLE IF NOT EXISTS \"{0}datastore_values\" (
    \"id\" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    \"tree_node_id\" INTEGER REFERENCES \"{0}datastore_tree\",
    \"change_id\" TEXT NOT NULL UNIQUE,
    \"timestamp\" TEXT NOT NULL,
    \"value\" TEXT
);
CREATE INDEX IF NOT EXISTS \"{0}index_datastore_values__tree_node_id\" ON \"{0}datastore_values\" (\"tree_node_id\");
CREATE INDEX IF NOT EXISTS \"{0}index_datastore_values__timestamp\" ON \"{0}datastore_values\" (\"timestamp\");
            ",
            table_prefix))
        .unwrap_or_else(|_| panic!("An error occurred while creating database tables \"{0}\"", table_prefix));
    }

    pub fn datastore_get_current(
        &self,
        config: &DatastoreDatabaseConfig,
        path: &[&str],
    ) -> Option<DatastoreValueMeta> {
        let table_prefix = Self::datastore_get_table_prefix(config);
        let conn = self.get_connection();

        todo!()
    }

    pub fn datastore_get_history(
        &self,
        config: &DatastoreDatabaseConfig,
        path: &[&str],
        last_change_id: Option<Uuid>,
    ) -> Vec<DatastoreValueMeta> {
        let table_prefix = Self::datastore_get_table_prefix(config);
        let conn = self.get_connection();

        todo!()
    }

    pub fn datastore_get_value(
        &self,
        config: &DatastoreDatabaseConfig,
        change_id: Uuid,
    ) -> Option<String> {
        let table_prefix = Self::datastore_get_table_prefix(config);
        let conn = self.get_connection();

        let mut select_stmt = conn
            .prepare_cached(&format!(
                "SELECT \"value\" FROM \"{0}datastore_values\" WHERE \"change_id\" = :change_id;",
                table_prefix
            ))
            .expect("Error occurred while preparing database query");
        let result: Option<Option<String>> = select_stmt
            .query_row(named_params! {":change_id": change_id}, |row| row.get(0))
            .optional()
            .expect("Error occurred while querying database");

        result.flatten()
    }

    pub fn datastore_list(&self, config: &DatastoreDatabaseConfig, path: &[&str]) -> Vec<String> {
        let table_prefix = Self::datastore_get_table_prefix(config);
        let conn = self.get_connection();

        todo!()
    }

    pub fn datastore_set(
        &self,
        config: &DatastoreDatabaseConfig,
        path: &[&str],
        value: Option<&str>,
    ) -> Uuid {
        let timestamp = Utc::now();
        let change_id = Uuid::new_v4();

        let table_prefix = Self::datastore_get_table_prefix(config);
        let conn = self.get_connection();

        let node_id = self.datastore_key_get_or_create(&conn, &table_prefix, path, None);

        let mut insert_stmt = conn
            .prepare_cached(&format!(
                "INSERT INTO \"{0}datastore_values\" (\"tree_node_id\", \"change_id\", \"timestamp\", \"value\") VALUES (:node_id, :change_id, :timestamp, :value));",
                table_prefix
            ))
            .expect("Error occurred while preparing database query");
        insert_stmt
            .execute(named_params! {":node_id": node_id, ":change_id": change_id, ":timestamp": timestamp, "value": value})
            .expect("Error occurred while inserting into database");

        change_id
    }

    pub fn datastore_cleanup(&self, config: &DatastoreDatabaseConfig, path: &[&str]) {
        let timestamp = Utc::now();

        let table_prefix = Self::datastore_get_table_prefix(config);
        let conn = self.get_connection();

        let node_id = self.datastore_key_get(&conn, &table_prefix, path, None);
        if let Some(node_id) = node_id {
            if config.keep_history {
                // delete all but the latest entry older than the max age
            } else {
                // delete all but latest value
            }
        }
    }

    fn datastore_get_table_prefix(config: &DatastoreDatabaseConfig) -> String {
        Self::get_table_prefix(
            config.namespace.as_deref(),
            Some(config.store_name.as_str()),
        )
    }

    fn datastore_key_get(
        &self,
        conn: &DBConnection,
        table_prefix: &str,
        path: &[&str],
        parent_id: Option<i64>,
    ) -> Option<Option<i64>> {
        if !path.is_empty() {
            let mut select_stmt = conn
                .prepare_cached(&format!("SELECT \"id\" FROM \"{0}datastore_tree\" WHERE \"parent_id\" = :parent_id AND \"key\" = :key;", table_prefix))
                .expect("Error occurred while preparing database query");
            let id_result: Option<i64> = select_stmt
                .query_row(
                    named_params! {":parent_id": parent_id, ":key": path[0]},
                    |row| row.get(0),
                )
                .optional()
                .expect("Error occurred while querying database");

            if let Some(id) = id_result {
                self.datastore_key_get(conn, table_prefix, &path[1..], Some(id))
            } else {
                None
            }
        } else {
            Some(parent_id)
        }
    }

    fn datastore_key_get_or_create(
        &self,
        conn: &DBConnection,
        table_prefix: &str,
        path: &[&str],
        parent_id: Option<i64>,
    ) -> Option<i64> {
        if !path.is_empty() {
            let mut select_stmt = conn
                .prepare_cached(&format!("SELECT \"id\" FROM \"{0}datastore_tree\" WHERE \"parent_id\" = :parent_id AND \"key\" = :key;", table_prefix))
                .expect("Error occurred while preparing database query");
            let id_result: Option<i64> = select_stmt
                .query_row(
                    named_params! {":parent_id": parent_id, ":key": path[0]},
                    |row| row.get(0),
                )
                .optional()
                .expect("Error occurred while querying database");

            let id = if let Some(id) = id_result {
                id
            } else {
                let mut insert_stmt = conn
                    .prepare_cached(&format!("INSERT INTO \"{0}datastore_tree\" (\"parent_id\", \"key\") VALUES (:parent_id, :key);", table_prefix))
                    .expect("Error occurred while preparing database query");
                insert_stmt
                    .execute(named_params! {":parent_id": parent_id, ":key": path[0]})
                    .expect("Error occurred while updating database");
                conn.last_insert_rowid()
            };

            self.datastore_key_get_or_create(conn, table_prefix, &path[1..], Some(id))
        } else {
            parent_id
        }
    }
}
