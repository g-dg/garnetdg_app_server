//! History-Tracking Change-Subscribable Tree-Based Key-Value Data Store

use std::{
    collections::{HashMap, VecDeque},
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

use crate::{config::DataStoreConfig, database::DbSchema};

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

                let subscriptions_by_id: HashMap<Uuid, Rc<Subscription<T>>> = HashMap::new();
                let subscriptions_by_path: HashMap<Vec<String>, Rc<Subscription<T>>> = HashMap::new();

                //TODO: make a doublely-linked-list hashmap for LRU cache
                let value_cache_by_change_id: HashMap<Uuid, Arc<Value<T>>> = HashMap::new();

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
                            DataStoreRequest::Subscribe { path, timeout, notification_response_channel, subscription_response_channel } => todo!(),
                            DataStoreRequest::Unsubscribe { subscription_id, response_channel } => todo!(),

                            // handles ping requests
                            DataStoreRequest::Ping { response_channel } => {
                                response_channel.send(()).expect("Error occurred while replying to data store ping");
                            },

                            // handle shutdown requests
                            DataStoreRequest::Shutdown { response_channel } => {
                                response_channel.send(()).expect("Error occurred while acknowledging data store thread shutdown command");
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

    /// Sends a ping and waits for a repsonse.
    /// Can be used to find current latency of the data store's request queue.
    pub async fn ping(&self) {
        let tx = self.mpsc_channel_sender.clone();

        let (ping_tx, ping_rx) = oneshot_async::channel();

        tx.send(DataStoreRequest::Ping {
            response_channel: ping_tx,
        })
        .expect("Error occurred while sending data store ping");

        ping_rx
            .await
            .expect("Error occurred while receiving ping reply from data store");
    }

    /// Requests data store shutdown.
    /// This gracefully shuts down the data store and waits for the thread to exit.
    /// Using the data store after shutdown is called results in undefined behavior.
    pub async fn shutdown(self) {
        let tx = self.mpsc_channel_sender.clone();

        let (shutdown_tx, shutdown_rx) = oneshot_async::channel();

        // request shutdown
        tx.send(DataStoreRequest::Shutdown {
            response_channel: shutdown_tx,
        })
        .expect("Error occurred while requesting data store shutdown");

        // wait for shutdown
        shutdown_rx
            .await
            .expect("Error occurred while waiting for data store shutdown");

        // wait for thread to exit
        let join_handle = self
            .join_handle
            .lock()
            .expect("Error occurred while locking data store thread handle for joining")
            .take();
        if let Some(join_handle) = join_handle {
            join_handle.join().expect("Data store thread panicked");
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
        response_channel: oneshot_async::Sender<Vec<Arc<Value<T>>>>,
    },

    /// Gets the current value
    GetCurrent {
        /// Path of the value to get
        path: Vec<String>,
        /// Response channel (sends the value entry)
        response_channel: oneshot_async::Sender<Arc<Value<T>>>,
    },

    /// Lists sub-keys of a path that currently contain values
    List {
        /// Path to list sub-keys that contain values
        path: Vec<String>,
        /// Response channel (sends list of keys)
        response_channel: oneshot_async::Sender<Vec<String>>,
    },

    /// Inserts a value into the history, updating the current value
    Set {
        /// Path to set the value of
        path: Vec<String>,
        /// Value to set
        value: T,
        /// Response channel
        response_channel: oneshot_async::Sender<()>,
    },

    /// Inserts a None value into the history, updating the current value
    Delete {
        /// Path to set a None value of
        path: Vec<String>,
        /// Response channel
        response_channel: oneshot_async::Sender<()>,
    },

    /// Subscribes for a change notification on a path
    Subscribe {
        /// Path to subscribe to
        path: Vec<String>,
        /// Subscription timeout
        timeout: Option<Duration>,
        /// Change notification channel
        notification_response_channel: SubscriptionNotificationChannel<T>,
        /// Response channel (sends subscription id)
        subscription_response_channel: oneshot_async::Sender<Uuid>,
    },

    /// Unsubscribes from changes for a path
    Unsubscribe {
        /// Subscription id to cancel
        subscription_id: Uuid,
        /// Response channel
        response_channel: oneshot_async::Sender<()>,
    },

    /// Pings the data store
    Ping {
        /// Response channel
        response_channel: oneshot_async::Sender<()>,
    },

    /// Request data store shutdown
    Shutdown {
        /// Response channel
        response_channel: oneshot_async::Sender<()>,
    },
}

struct Subscription<T> {
    id: Uuid,
    path: Vec<String>,
    channel: SubscriptionNotificationChannel<T>,
}

enum SubscriptionNotificationChannel<T> {
    Single(oneshot_async::Sender<Vec<Option<Arc<Value<T>>>>>),
    Multiple(mpsc_async::UnboundedSender<Vec<Option<Arc<Value<T>>>>>),
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
