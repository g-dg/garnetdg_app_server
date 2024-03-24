//! History-Tracking Change-Subscribable Tree-Based Key-Value Data Store

use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    rc::{Rc, Weak},
    sync::{
        mpsc::{self, RecvTimeoutError},
        Arc, Mutex, Weak as AWeak,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::{mpsc as mpsc_async, oneshot as oneshot_async};
use uuid::Uuid;

use crate::{
    config::DataStoreConfig,
    database::DbSchema,
    helpers::{sync_async::OneshotSender, tlru_cache::TLRUCache},
};

const ITEM_CACHE_MAX_ITEMS: usize = 1000;
const ITEM_CACHE_MAX_ACCESS_AGE: Duration = Duration::from_secs(3600);

/// Data store object
#[derive(Clone)]
pub struct DataStore<T> {
    name: String,
    config: DataStoreConfig,
    database: DbSchema,
    mpsc_channel_sender: mpsc::Sender<DataStoreRequest<T>>,
    join_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl<T: Serialize + DeserializeOwned + Send + Sync + 'static> DataStore<T> {
    /// Sets up a new data store.
    /// Once the data store is done being used, the `shutdown` function must be called.
    pub async fn new(name: &str, config: DataStoreConfig, database: Option<DbSchema>) -> Self {
        // we require a database for organizing the data
        // create an in-memory one if none is set
        let database = if let Some(database) = database {
            database
        } else {
            DbSchema::new_memory()
        };

        // create database schema
        database.schema_create_data_store(name);

        // oneshot used to get the request channel from the spawned thread
        let (spawn_tx, spawn_rx) = oneshot_async::channel();

        // customize thread name to datastore name
        let thread_builder = thread::Builder::new().name(String::from(name));

        // spawn datastore thread
        let join_handle = thread_builder
            .spawn(move || {
                // create MPSC channel that requests will be sent with
                let (tx, rx) = mpsc::channel();
                // send the request channel back to the parent thread
                spawn_tx
                    .send(tx)
                    .expect("Error occurred while sending transmitter from data store thread");

                let subscriptions_by_id: HashMap<Uuid, Rc<SubscriptionRecord<T>>> = HashMap::new();
                let subscriptions_by_path: HashMap<Vec<String>, Rc<SubscriptionRecord<T>>> = HashMap::new();

                let value_cache_by_change_id: TLRUCache<Uuid, Arc<Value<T>>> = TLRUCache::new(Some(ITEM_CACHE_MAX_ITEMS), None, Some(ITEM_CACHE_MAX_ACCESS_AGE));

                // thread loop
                loop {
                    // run loop tasks before loop
                    //TODO: occasionally commit changes to database

                    // contains the timeout to allow tasks to run again
                    let recv_timeout = Duration::from_millis(1000);

                    // wait for request
                    match rx.recv_timeout(recv_timeout){
                        Ok(request) => match request {
                            DataStoreRequest::Get { path, last_change_id, response_channel } => todo!(),
                            DataStoreRequest::GetCurrent { path, response_channel } => todo!(),
                            DataStoreRequest::List { path, response_channel } => todo!(),
                            DataStoreRequest::Set { path, value, response_channel } => todo!(),
                            DataStoreRequest::Delete { path, response_channel } => todo!(),
                            DataStoreRequest::Subscribe { path, timeout, response_channel } => todo!(),
                            DataStoreRequest::Unsubscribe { subscription_id, response_channel } => todo!(),

                            // handles ping requests
                            DataStoreRequest::Ping { response_channel } => {
                                if let Some(response_channel) = response_channel {
                                    response_channel.send(()).expect("Error occurred while replying to data store ping");
                                }
                            },

                            // handle shutdown requests
                            DataStoreRequest::Shutdown { response_channel } => {
                                if let Some(response_channel) = response_channel {
                                    response_channel.send(()).expect("Error occurred while acknowledging data store thread shutdown command");
                                }
                                break;
                            },
                        },

                        Err(recv_error) => match recv_error {
                            RecvTimeoutError::Timeout => {},
                            RecvTimeoutError::Disconnected => {
                                // all senders got dropped, shutdown thread
                                break;
                            },
                        },
                    }
                }

                // clean up
                //TODO: commit changes to database here

            })
            .expect("Failed to spawn data store thread");

        // recieve transmitter from new thread
        let tx = spawn_rx
            .await
            .expect("Error occurred while receiving transmitter from data store thread");

        // return datastore object
        Self {
            name: String::from(name),
            config,
            database,
            mpsc_channel_sender: tx,
            join_handle: Arc::new(Mutex::new(Some(join_handle))),
        }
    }

    pub async fn get_all(&self, path: &[&str], last_change_id: Option<Uuid>) -> Vec<Arc<Value<T>>> {
        let tx = self.mpsc_channel_sender.clone();

        let (response_tx, response_rx) = oneshot_async::channel();

        tx.send(DataStoreRequest::Get {
            path: path.iter().map(|x| String::from(*x)).collect(),
            last_change_id,
            response_channel: OneshotSender::Async(response_tx),
        })
        .expect("Error occurred while sending get all request to data store");

        response_rx
            .await
            .expect("Error occurred while receiving get all response from data store")
    }

    pub async fn get_current(&self, path: &[&str]) -> Arc<Value<T>> {
        let tx = self.mpsc_channel_sender.clone();

        let (response_tx, response_rx) = oneshot_async::channel();

        tx.send(DataStoreRequest::GetCurrent {
            path: path.iter().map(|x| String::from(*x)).collect(),
            response_channel: OneshotSender::Async(response_tx),
        })
        .expect("Error occurred while sending get current request to data store");

        response_rx
            .await
            .expect("Error occurred while receiving get current response from data store")
    }

    pub async fn list(&self, path: &[&str]) -> Vec<String> {
        let tx = self.mpsc_channel_sender.clone();

        let (response_tx, response_rx) = oneshot_async::channel();

        tx.send(DataStoreRequest::List {
            path: path.iter().map(|x| String::from(*x)).collect(),
            response_channel: OneshotSender::Async(response_tx),
        })
        .expect("Error occurred while sending list request to data store");

        response_rx
            .await
            .expect("Error occurred while receiving list response from data store")
    }

    pub async fn set(&self, path: &[&str], value: T) -> Uuid {
        let tx = self.mpsc_channel_sender.clone();

        let (response_tx, response_rx) = oneshot_async::channel();

        tx.send(DataStoreRequest::Set {
            path: path.iter().map(|x| String::from(*x)).collect(),
            value,
            response_channel: Some(OneshotSender::Async(response_tx)),
        })
        .expect("Error occurred while sending set request to data store");

        response_rx
            .await
            .expect("Error occurred while receiving set response from data store")
    }

    pub async fn delete(&self, path: &[&str]) -> Uuid {
        let tx = self.mpsc_channel_sender.clone();

        let (response_tx, response_rx) = oneshot_async::channel();

        tx.send(DataStoreRequest::Delete {
            path: path.iter().map(|x| String::from(*x)).collect(),
            response_channel: Some(OneshotSender::Async(response_tx)),
        })
        .expect("Error occurred while sending delete request to data store");

        response_rx
            .await
            .expect("Error occurred while receiving delete response from data store")
    }

    pub async fn subscribe(&self, path: &[&str]) -> Subscription<T> {
        todo!()
    }

    pub async fn subscribe_timeout(&self, path: &[&str], timeout: Duration) -> Subscription<T> {
        todo!()
    }

    pub async fn unsubscribe(&self, subscription_id: Uuid) {
        todo!()
    }

    /// Sends a ping and waits for a repsonse.
    /// Can be used to find current latency of the data store's request queue.
    pub async fn ping(&self) {
        let tx = self.mpsc_channel_sender.clone();

        let (ping_tx, ping_rx) = oneshot_async::channel();

        tx.send(DataStoreRequest::Ping {
            response_channel: Some(OneshotSender::Async(ping_tx)),
        })
        .expect("Error occurred while sending data store ping");

        ping_rx
            .await
            .expect("Error occurred while receiving ping reply from data store");
    }
}

impl<T> Drop for DataStore<T> {
    fn drop(&mut self) {
        // If we're the last clone, then we can shut down the thread
        if Arc::strong_count(&self.join_handle) == 1 {
            let tx = self.mpsc_channel_sender.clone();

            let (response_tx, response_rx) = mpsc::channel();

            tx.send(DataStoreRequest::Shutdown {
                response_channel: Some(OneshotSender::Sync(response_tx)),
            })
            .ok();

            // wait for shutdown
            response_rx.recv().ok();

            // wait for thread to exit
            self.join_handle
                .lock()
                .unwrap()
                .take()
                .unwrap()
                .join()
                .unwrap();
        }
    }
}

/// Requests for the data store thread.
enum DataStoreRequest<T> {
    /// Gets current and previous values
    Get {
        /// Path of the value to get
        path: Vec<String>,
        /// Sends all changes after this last change id
        last_change_id: Option<Uuid>,
        /// Response channel (sends the list of value entries)
        response_channel: OneshotSender<Vec<Arc<Value<T>>>>,
    },

    /// Gets the current value
    GetCurrent {
        /// Path of the value to get
        path: Vec<String>,
        /// Response channel (sends the value entry)
        response_channel: OneshotSender<Arc<Value<T>>>,
    },

    /// Lists sub-keys of a path that currently contain values
    List {
        /// Path to list sub-keys that contain values
        path: Vec<String>,
        /// Response channel (sends list of keys)
        response_channel: OneshotSender<Vec<String>>,
    },

    /// Inserts a value into the history, updating the current value
    Set {
        /// Path to set the value of
        path: Vec<String>,
        /// Value to set
        value: T,
        /// Response channel
        response_channel: Option<OneshotSender<Uuid>>,
    },

    /// Inserts a None value into the history, updating the current value
    Delete {
        /// Path to set a None value of
        path: Vec<String>,
        /// Response channel
        response_channel: Option<OneshotSender<Uuid>>,
    },

    /// Subscribes for a change notification on a path
    Subscribe {
        /// Path to subscribe to
        path: Vec<String>,
        /// Subscription timeout
        timeout: Option<Duration>,
        /// Response channel (sends subscription id)
        response_channel: OneshotSender<SubscriptionRecord<T>>,
    },

    /// Unsubscribes from changes for a path
    Unsubscribe {
        /// Subscription id to cancel
        subscription_id: Uuid,
        /// Response channel
        response_channel: Option<OneshotSender<()>>,
    },

    /// Pings the data store
    Ping {
        /// Response channel
        response_channel: Option<OneshotSender<()>>,
    },

    /// Request data store shutdown
    Shutdown {
        /// Response channel
        response_channel: Option<OneshotSender<()>>,
    },
}

struct SubscriptionRecord<T> {
    id: Uuid,
    path: Vec<String>,
    notification_channel: mpsc_async::Sender<Arc<Value<T>>>,
}

/// Object returned by the datastore api
pub struct Value<T> {
    /// The current value, None if not set or deleted
    pub value: Option<T>,
    /// The path of the value
    pub path: Vec<String>,
    /// The timestamp that this value was set (server time)
    pub timestamp: DateTime<Utc>,
    /// Change ID
    pub change_id: Uuid,
}

pub struct Subscription<T> {
    pub id: Uuid,
    notification_channel: mpsc_async::Receiver<Arc<Value<T>>>,
    data_store_channel: mpsc::Sender<DataStoreRequest<T>>,
}

impl<T> Subscription<T> {
    pub async fn recv(&self) -> Result<Arc<Value<T>>, ()> {
        todo!()
    }

    pub async fn recv_timeout(&self) -> Result<Option<Arc<Value<T>>>, ()> {
        todo!()
    }
}
impl<T> Drop for Subscription<T> {
    fn drop(&mut self) {
        // Once subscription is dropped, cancel it
        self.data_store_channel
            .send(DataStoreRequest::Unsubscribe {
                subscription_id: self.id,
                response_channel: None,
            })
            .ok();
    }
}
