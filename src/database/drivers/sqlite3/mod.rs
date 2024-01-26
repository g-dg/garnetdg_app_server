//! SQLite3 database driver

mod key_value;

use super::DbDriver;
use crate::config::DatabaseConnectionConfig;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

/// SQLite3 database connection
#[derive(Clone)]
pub struct SQLite3Connection {
    pool: Pool<SqliteConnectionManager>,
}

impl DbDriver for SQLite3Connection {
    fn new(config: &DatabaseConnectionConfig) -> Self {
        match config {
            DatabaseConnectionConfig::SQLite3 { database } => {
                let manager = SqliteConnectionManager::file(database)
                    .with_init(|c| c.execute_batch("PRAGMA busy_timeout = 60000;"));
                let pool = r2d2::Pool::new(manager).expect("Could not connect to SQLite3 database");
                pool.get()
				.expect("Could not get database connection for initialization")
				.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA foreign_keys = 1; PRAGMA auto_vacuum = INCREMENTAL; PRAGMA recursive_triggers = 1;")
				.expect("Could not run database initialization commands");
                Self { pool }
            }
        }
    }

    fn schema_create_key_value(&self, table_prefix: Option<&str>, store_name: &str) {
        SQLite3Connection::schema_create_key_value(&self, table_prefix, store_name)
    }

    fn key_value_get(
        &self,
        table_prefix: Option<&str>,
        store_name: &str,
        key: &[&str],
    ) -> Option<String> {
        SQLite3Connection::key_value_get(self, table_prefix, store_name, key)
    }

    fn key_value_set(
        &self,
        table_prefix: Option<&str>,
        store_name: &str,
        key: &[&str],
        value: &str,
    ) {
        SQLite3Connection::key_value_set(self, table_prefix, store_name, key, value)
    }
}

impl SQLite3Connection {
    const TABLE_NAME_SEPARATOR: &'static str = "__";
    const TABLE_NAME_ALLOWED_CHARACTERS: &'static str =
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_";

    fn sanitize_table_name(name: &str) -> String {
        name.chars()
            .filter(|c| Self::TABLE_NAME_ALLOWED_CHARACTERS.chars().any(|x| x == *c))
            .collect()
    }

    fn get_table_prefix(table_prefix: Option<&str>, store_name: Option<&str>) -> String {
        let mut table_name = String::new();
        if let Some(prefix) = table_prefix {
            table_name.push_str(&Self::sanitize_table_name(prefix));
            table_name.push_str(Self::TABLE_NAME_SEPARATOR);
        }
        if let Some(name) = store_name {
            table_name.push_str(&Self::sanitize_table_name(name));
            table_name.push_str(Self::TABLE_NAME_SEPARATOR);
        }
        table_name
    }
}
