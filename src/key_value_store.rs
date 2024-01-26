//! Key-value store

use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};

use crate::{config::KeyValueConfig, database::DbSchema};

/// Key-value store
#[derive(Clone)]
pub struct KeyValueStore<T> {
    /// Name of the key-value store
    name: String,
    /// Configuration object
    config: KeyValueConfig,
    /// Database schema used for storing data
    database: Option<DbSchema>,
    _phantom: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> KeyValueStore<T> {
    /// Creates a new instance of the key-value store
    pub fn new(store_name: &str, config: KeyValueConfig, database: Option<DbSchema>) -> Self {
        if let Some(database) = database.clone() {
            database.schema_create_key_value(store_name);
        }

        KeyValueStore {
            name: String::from(store_name),
            config,
            database,
            _phantom: PhantomData,
        }
    }

    /// Gets a the value of a key
    pub fn get(&self, key: &[&str]) -> Option<T> {
        if let Some(database) = self.database.clone() {
            database.key_value_get(&self.name, &key).and_then(|result| {
                serde_json::from_str(&result).expect("Failed to convert key value from JSON")
            })
        } else {
            //TODO: in-memory key-value store
            None
        }
    }

    /// Sets the value of a key
    pub fn set(&self, key: &[&str], value: T) {
        if let Some(database) = self.database.clone() {
            database.key_value_set(
                &self.name,
                &key,
                &serde_json::to_string(&value).expect("Failed to convert key value to JSON"),
            )
        } else {
            //TODO: in-memory key-value store
        }
    }
}
