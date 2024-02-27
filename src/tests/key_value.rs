//! Key-value store tests

use std::collections::HashMap;

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    config::{DatabaseConfig, DatabaseConnectionConfig, DatabaseSchemaConfig, KeyValueConfig},
    database::DbSchema,
    key_value_store::KeyValueStore,
};

async fn create_key_value_store<T: Send + Sync + Serialize + DeserializeOwned>(
    key_value_config: Option<KeyValueConfig>,
) -> KeyValueStore<T> {
    let db_config = DatabaseConfig {
        connections: HashMap::from([(
            String::from("memory"),
            DatabaseConnectionConfig::SQLite3 {
                database: String::from(":memory:"),
            },
        )]),
        schemas: HashMap::from([(
            String::from("memory"),
            DatabaseSchemaConfig {
                connection: String::from("memory"),
                table_prefix: None,
            },
        )]),
    };

    let all_db_schemas = &DbSchema::connect_all(&db_config);
    let db_schema = all_db_schemas
        .get("memory")
        .expect("Could not find \"memory\" database schema in testing schemas");

    let key_value_config = match key_value_config {
        Some(config) => config,
        None => KeyValueConfig {
            database_schema: None,
        },
    };

    let key_value_store =
        KeyValueStore::<T>::new("TestStore", key_value_config, Some(db_schema.clone()));

    key_value_store
}

#[tokio::test]
pub async fn store_initialization() {
    // create key-value store
    create_key_value_store::<String>(None).await;
}

#[tokio::test]
pub async fn newly_created_store_is_empty() {
    let store = create_key_value_store::<String>(None).await;

    let result = store.get(&[]);
    assert_eq!(
        result, None,
        "New key-value store must have a root with no value"
    );

    let result = store.get(&["test"]);
    assert_eq!(
        result, None,
        "New key-value store must not return a value for a child"
    );

    let result = store.list(&[]);
    assert_eq!(
        result.len(),
        0,
        "New key-value store must have not list any children"
    );
}
