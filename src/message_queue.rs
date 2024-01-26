//! Message queue

use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{de::Visitor, Deserialize, Serialize};
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::{config::MessageQueueConfig, database::DbSchema};

/// Message queue
#[derive(Clone)]
pub struct MessageQueue<T: Send + Sync + 'static> {
    /// Name of the message queue
    name: String,
    /// Configuration object
    config: MessageQueueConfig,
    /// Database schema used for storing data
    database: Option<DbSchema>,
    /// MPSC channel used to send requests to the message queue thread
    message_channel: UnboundedSender<MessageQueueRequest<T>>,
    /// Join handle of message queue thread
    join_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl<T: Send + Sync + 'static> MessageQueue<T> {
    /// Sets up a new message queue.
    /// Once the message queue is done being used, the `shutdown` function must be called.
    pub async fn new(name: &str, config: MessageQueueConfig, database: Option<DbSchema>) -> Self {
        let (spawn_tx, mut spawn_rx) = mpsc::unbounded_channel();

        let thread_builder = thread::Builder::new().name(String::from(name));

        let join_handle = thread_builder
            .spawn(move || {
                let (tx, mut rx) = mpsc::unbounded_channel();
                spawn_tx
                    .send(tx)
                    .expect("Error occurred while sending transmitter from message queue thread");

                let mut subscription_tree: SubscriptionTreeNode<T> = SubscriptionTreeNode::new();

                loop {
                    match rx.blocking_recv().expect("Channel for message queue closed") {

                        // handles requests to get any sent messages
                        MessageQueueRequest::Get {
                            path,
                            response_channel,
                            last_message_id,
                        } => {
                            let messages = subscription_tree.get_sent_messages(&path, &last_message_id);
                            response_channel.send(messages).expect("Error occurred while sending existing messages from message queue");
                        }

                        // handles request to send either send any missed messages or wait for new messages
                        MessageQueueRequest::Wait {
                            path,
                            response_channel,
                            last_message_id,
                        } => {
                            // get missed messages
                            let missed_messages = match last_message_id {
                                Some(last_message_id) => subscription_tree.get_sent_messages(&path, &Some(last_message_id)),
                                None => Vec::new()
                            };

                            // if any missed messages, return them
                            if missed_messages.len() > 0 {
                                response_channel.send(missed_messages).expect("Error occurred while sending missed messages from message queue");
                            } else {
                                // else subscribe to that channel
                                let node = subscription_tree.get_or_create_node(&path);

                                let subscription = Subscription {
                                    channel: response_channel,
                                };
                                node.subscriptions.push(subscription);
                            }
                        }

                        // handles requests to send a message
                        MessageQueueRequest::Send { path, message } => {
                            let message_id = MessageID::new();
                            let message = Message {
                                message,
                                path: path.clone(),
                                timestamp: Utc::now(),
                                message_id,
                            };

                            Self::send_message_recursive(&mut subscription_tree, &path, Arc::new(message));

                            //TODO: optionally save messages in database
                            //TODO: cleanup older messages according to config settings
                            //TODO: also clean up any empty nodes (i.e. nodes that have no subscriptions and no non-expired messages)
                        }

                        // handles ping requests
                        MessageQueueRequest::Ping { response_channel } => {
                            response_channel.send(()).expect("Error occurred while replying to message queue ping");
                        },

                        // handles shutdown requests
                        MessageQueueRequest::Shutdown { response_channel } => {
                            response_channel.send(()).expect("Error occurred while acknowledging message queue thread shutdown command");
                            break;
                        }
                    }
                }
            })
            .expect("Failed to spawn message thread");

        // recieve transmitter from new thread
        let tx = spawn_rx
            .recv()
            .await
            .expect("Error occurred while receiving transmitter from message queue thread");

        Self {
            name: String::from(name),
            config,
            database,
            message_channel: tx,
            join_handle: Arc::new(Mutex::new(Some(join_handle))),
        }
    }

    /// Sends messages recursively.
    fn send_message_recursive(
        subscription_tree: &mut SubscriptionTreeNode<T>,
        path: &[String],
        message: Arc<Message<T>>,
    ) {
        // depth-first recursion
        if path.len() > 0 {
            let descendent = subscription_tree
                .descendents
                .entry(path[0].clone())
                .or_insert_with(|| SubscriptionTreeNode::new());

            Self::send_message_recursive(descendent, &path[1..], message.clone());
        }

        // save message
        subscription_tree.messages.push_back(message.clone());

        // send messages to subscriptions
        // TODO: add support for non-recursive subscriptions
        while let Some(subscription) = subscription_tree.subscriptions.pop() {
            subscription
                .channel
                .send(vec![message.clone()])
                .expect("Error occurred while sending message from message queue");
        }
    }

    /// Sends a message.
    /// Returns once the message is sent and doesn't wait until the message is received.
    pub async fn send(&self, path: Vec<String>, message: T) {
        self.message_channel
            .send(MessageQueueRequest::Send { path, message })
            .expect("Error occurred while sending message to message queue");
    }

    /// Waits until a new message is received.
    /// If `last_message_id` is Some, sends only messages that were sent after that ID.
    /// If `last_message_id` is None, only waits for new messages.
    pub async fn recv(
        &self,
        path: Vec<String>,
        last_message_id: Option<MessageID>,
    ) -> Vec<Arc<Message<T>>> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        self.message_channel
            .send(MessageQueueRequest::Wait {
                path,
                response_channel: tx,
                last_message_id: last_message_id,
            })
            .expect("Error occurred while sending subscription request to message queue");

        let messages = rx
            .recv()
            .await
            .expect("Subscribed channel dropped from message queue");

        messages
    }

    /// Gets any available messages (useful for polling).
    /// If last_message_id is Some, only sends messages that were sent after that ID.
    /// if last_message_id is None, sends all sent messages.
    pub async fn get_messages(
        &self,
        path: Vec<String>,
        last_message_id: Option<MessageID>,
    ) -> Vec<Arc<Message<T>>> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        self.message_channel
            .send(MessageQueueRequest::Get {
                path,
                response_channel: tx,
                last_message_id,
            })
            .expect("Error occurred while requesting messages from message queue");

        let messages = rx
            .recv()
            .await
            .expect("Channel dropped while attempting to get messages from message queue");

        messages
    }

    /// Sends a ping and waits for a repsonse.
    /// Can be used to find current latency of message queue.
    pub async fn ping(&self) {
        let tx = self.message_channel.clone();

        let (ping_tx, mut ping_rx) = mpsc::unbounded_channel();

        tx.send(MessageQueueRequest::Ping {
            response_channel: ping_tx,
        })
        .expect("Error occurred while sending message queue ping");

        ping_rx
            .recv()
            .await
            .expect("Error occurred while receiving ping reply from message queue");
    }

    /// Requests a shutdown.
    /// Using the message queue after shutdown is called results in undefined behavior (likely panics).
    pub async fn shutdown(self) {
        let tx = self.message_channel.clone();

        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();

        // request shutdown
        tx.send(MessageQueueRequest::Shutdown {
            response_channel: shutdown_tx,
        })
        .expect("Error occurred while requesting message queue shutdown");

        // wait for shutdown
        shutdown_rx
            .recv()
            .await
            .expect("Error occurred while waiting for message queue shutdown");

        // wait for thread to exit
        let join_handle = self
            .join_handle
            .lock()
            .expect("Error occurred while locking message queue thread join handle")
            .take();
        if let Some(join_handle) = join_handle {
            join_handle.join().expect("Message queue thread panicked");
        }
    }
}

/// Used to store a message and its associated metadata
#[derive(Debug)]
pub struct Message<T> {
    /// The message
    pub message: T,
    /// The path of the message
    pub path: Vec<String>,
    /// The timestamp that the message was sent (server time)
    pub timestamp: DateTime<Utc>,
    /// Message ID
    pub message_id: MessageID,
}

/// Enum of requests sent to the message queue thread
enum MessageQueueRequest<T> {
    /// Request to get any sent messages (does not wait)
    Get {
        path: Vec<String>,
        response_channel: UnboundedSender<Vec<Arc<Message<T>>>>,
        last_message_id: Option<MessageID>,
    },
    /// Request to wait for a new message or return missed messages
    Wait {
        path: Vec<String>,
        response_channel: UnboundedSender<Vec<Arc<Message<T>>>>,
        last_message_id: Option<MessageID>,
    },
    /// Request to send a message
    Send { path: Vec<String>, message: T },
    /// Request to ping
    Ping {
        response_channel: UnboundedSender<()>,
    },
    /// Request to shut down
    Shutdown {
        response_channel: UnboundedSender<()>,
    },
}

/// Tree of subscriptions and sent messages
#[derive(Debug)]
struct SubscriptionTreeNode<T> {
    /// Hashmap of path descendents
    descendents: HashMap<String, SubscriptionTreeNode<T>>,
    /// List of subscriptions to this path
    subscriptions: Vec<Subscription<T>>,
    /// Queue of the most recently sent messages
    messages: VecDeque<Arc<Message<T>>>,
}
impl<T> SubscriptionTreeNode<T> {
    /// Creates a new subscription tree node.
    fn new() -> SubscriptionTreeNode<T> {
        SubscriptionTreeNode::<T> {
            subscriptions: Vec::new(),
            descendents: HashMap::new(),
            messages: VecDeque::new(),
        }
    }

    /// Gets a subscription tree node recursively based on the provided path.
    fn get_node(&self, path: &[String]) -> Option<&SubscriptionTreeNode<T>> {
        if path.len() > 0 {
            let descendent = &self.descendents.get(&path[0]);
            if let Some(descendent) = descendent {
                descendent.get_node(&path[1..])
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    /// Gets or creates a subscription tree node recursively based on the provided path.
    fn get_or_create_node(&mut self, path: &[String]) -> &mut SubscriptionTreeNode<T> {
        if path.len() > 0 {
            self.descendents
                .entry(path[0].clone())
                .or_insert_with(|| SubscriptionTreeNode::new())
                .get_or_create_node(&path[1..])
        } else {
            self
        }
    }

    /// Gets any sent messages for the provided path.
    fn get_sent_messages(
        &self,
        path: &Vec<String>,
        last_message_id: &Option<MessageID>,
    ) -> Vec<Arc<Message<T>>> {
        let empty = VecDeque::new();
        let all_messages = match self.get_node(&path) {
            Some(node) => &node.messages,
            None => &empty,
        };

        let messages: Vec<Arc<Message<T>>> = if let Some(last_message_id) = last_message_id {
            let mut found_message = false;

            // message list is a queue with more recent messages at the back
            all_messages
                .iter()
                .rev()
                .filter(|msg| {
                    if found_message {
                        // we've found the last message, don't return earlier messages
                        false
                    } else {
                        if msg.message_id == *last_message_id {
                            found_message = true;
                            // we're on the last message, don't return it
                            false
                        } else {
                            // return most recent messages
                            true
                        }
                    }
                })
                .collect::<Vec<_>>() // need to collect into new vec since otherwise .rev() won't actually reverse the order the filter is called
                .into_iter()
                .rev()
                .map(|msg| msg.clone())
                .collect()
        } else {
            // return all messages
            all_messages.iter().map(|msg| msg.clone()).collect()
        };

        messages
    }
}

/// Contains a MPSC channel to reply when a message is sent
#[derive(Clone, Debug)]
struct Subscription<T> {
    channel: UnboundedSender<Vec<Arc<Message<T>>>>,
}

/// Handles message ID creation, storage and formatting
#[derive(Copy, Clone)]
pub struct MessageID {
    id: u128,
}
impl MessageID {
    pub const NONE: MessageID = MessageID { id: 0 };

    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self { id: rng.gen() }
    }

    pub fn to_string(&self) -> String {
        format!("{:032x}", self.id)
    }
}
impl PartialEq for MessageID {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Display for MessageID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl Debug for MessageID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageID")
            .field("id", &self.to_string())
            .finish()
    }
}
impl Serialize for MessageID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
impl<'de> Deserialize<'de> for MessageID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(MessageIDVisitor)
    }
}
/// Used to deserialize a message ID
struct MessageIDVisitor;
impl<'de> Visitor<'de> for MessageIDVisitor {
    type Value = MessageID;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a hexadecimal string up to 32 characters")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match u128::from_str_radix(v, 16) {
            Ok(id) => Ok(MessageID { id }),
            Err(err) => Err(E::custom(format!("{}", err.to_string()))),
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match u128::from_str_radix(v.as_str(), 16) {
            Ok(id) => Ok(MessageID { id }),
            Err(err) => Err(E::custom(format!(
                "Failed to convert message id: {}",
                err.to_string()
            ))),
        }
    }
}
