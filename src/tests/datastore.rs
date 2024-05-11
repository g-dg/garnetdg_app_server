use crate::{config::DatastoreConfig, datastore::DataStore};

#[tokio::test]
async fn ping() {
    let datastore: DataStore<String> = DataStore::new(
        "test",
        DatastoreConfig {
            database_schema: None,
            keep_history: true,
            history_max_age: Some(3600),
            history_max_entries: Some(1000),
        },
        None,
    )
    .await;

    datastore.ping().await;

    let datastore2 = datastore.clone();

    datastore.ping().await;
    datastore2.ping().await;
}

#[tokio::test]
async fn basic_get_set() {
    let datastore: DataStore<String> = DataStore::new(
        "test",
        DatastoreConfig {
            database_schema: None,
            keep_history: false,
            history_max_age: None,
            history_max_entries: None,
        },
        None,
    )
    .await;

    datastore.set(&[], String::from("test1")).await;
    assert_eq!(
        datastore.get_current(&[]).await.value,
        Some(String::from("test1"))
    );

    datastore.set(&[], String::from("test2")).await;
    assert_eq!(
        datastore.get_current(&[]).await.value,
        Some(String::from("test2"))
    );
}
