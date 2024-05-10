//! SQLite3 database driver

pub mod datastore;

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
                    .with_init(|c| c.execute_batch("PRAGMA busy_timeout = 60000; PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA foreign_keys = 1; PRAGMA auto_vacuum = INCREMENTAL; PRAGMA recursive_triggers = 1;"));
                let pool = r2d2::Pool::new(manager).expect("Could not connect to SQLite3 database");
                Self { pool }
            }
        }
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
