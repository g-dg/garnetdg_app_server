use crate::{config::DataStoreConfig, datastore::DataStore};

#[tokio::test]
async fn ping() {
    let datastore: DataStore<String> = DataStore::new(
        "test",
        DataStoreConfig {
            database_schema: None,
            keep_history: Some(true),
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
