//! Message queue tests

use std::time::Duration;

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    config::MessageQueueConfig,
    message_queue::{MessageID, MessageQueue},
};

async fn start_message_queue<T: Send + Sync + Serialize + DeserializeOwned>(
    config: Option<MessageQueueConfig>,
) -> MessageQueue<T> {
    let config = match config {
        Some(config) => config,
        None => MessageQueueConfig {
            database_schema: None,
            message_expiry: None,
            message_limit: None,
        },
    };

    let message_queue = MessageQueue::<T>::new("Test MsgQueue", config, None).await;

    message_queue
}

async fn stop_message_queue<T: Send + Sync + Serialize + DeserializeOwned>(
    message_queue: MessageQueue<T>,
) {
    message_queue.shutdown().await;
}

#[tokio::test]
pub async fn basic_startup_shutdown() {
    // create message queue
    let message_queue = start_message_queue::<String>(None).await;

    // stop message queue
    stop_message_queue(message_queue).await;
}

#[tokio::test]
pub async fn unique_message_ids() {
    let message_ids = (0..256)
        .map(|_| MessageID::new())
        .collect::<Vec<MessageID>>();
    for i in &message_ids {
        assert_eq!(message_ids.iter().filter(|x| *x == i).count(), 1);
    }
}

#[tokio::test]
pub async fn empty_message_queue() {
    let message_queue = start_message_queue::<String>(None).await;

    // newly created message queue must be empty
    let result = message_queue.get_messages(vec![], None).await;
    assert_eq!(result.len(), 0, "New message queue must be empty");

    // child element of message queue must be empty
    let result = message_queue
        .get_messages(vec![String::from("test")], None)
        .await;
    assert_eq!(
        result.len(),
        0,
        "Child element of message queue must be empty"
    );

    stop_message_queue(message_queue).await;
}

#[tokio::test]
pub async fn empty_message_queue_child() {
    let message_queue = start_message_queue::<String>(None).await;

    // child element of message queue must be empty
    let result = message_queue
        .get_messages(vec![String::from("test")], None)
        .await;
    assert_eq!(
        result.len(),
        0,
        "Child of message queue must be empty with no messages"
    );

    stop_message_queue(message_queue).await;
}

#[tokio::test]
pub async fn empty_message_queue_previous_message() {
    let message_queue = start_message_queue::<String>(None).await;

    // no messages returned with no messages and random previous message id
    let result = message_queue
        .get_messages(vec![], Some(MessageID::new()))
        .await;
    assert_eq!(
        result.len(),
        0,
        "Messages returned with no messages in queue and random previous message id"
    );

    stop_message_queue(message_queue).await;
}

#[tokio::test]
pub async fn message_queue_root() {
    let message_queue = start_message_queue(None).await;

    // send message to root
    message_queue.send(vec![], String::from("test1")).await;

    // ensure message sent at root gets returned at root
    let result = message_queue.get_messages(vec![], None).await;
    assert_eq!(result.len(), 1, "Only 1 message should be in root");
    assert_eq!(
        result[0].message,
        String::from("test1"),
        "Message \"test1\" expected in index 0"
    );

    // ensure no messages are returned after the last message
    let result = message_queue
        .get_messages(vec![], Some(result[0].message_id))
        .await;
    assert_eq!(result.len(), 0, "No message must be after the last message");

    // ensure message sent at root is not visible to child
    let result = message_queue
        .get_messages(vec![String::from("test1")], None)
        .await;
    assert_eq!(result.len(), 0, "Message sent to root is visible to child");

    stop_message_queue(message_queue).await;
}

#[tokio::test]
pub async fn message_queue_child() {
    let message_queue = start_message_queue(None).await;

    // send message to child
    message_queue
        .send(vec![String::from("test")], String::from("test1"))
        .await;

    // ensure message sent at child is visible to root
    let result = message_queue
        .get_messages(vec![String::from("test")], None)
        .await;
    assert_eq!(
        result.len(),
        1,
        "1 message sent to child must be visible to child"
    );
    assert_eq!(
        result[0].message,
        String::from("test1"),
        "Message \"test1\" sent to child must be visible to child"
    );

    // ensure message sent at child gets returned at child
    let result = message_queue.get_messages(vec![], None).await;
    assert_eq!(
        result.len(),
        1,
        "1 message sent to child must be visible to root"
    );
    assert_eq!(
        result[0].message,
        String::from("test1"),
        "Message \"test1\" sent to child must be visible to root"
    );

    let message_id = result[0].message_id;

    // ensure no messages are returned after the last message for child
    let result = message_queue.get_messages(vec![], Some(message_id)).await;
    assert_eq!(
        result.len(),
        0,
        "Must return no messages from root after last sent message"
    );

    // ensure no messages are returned after the last message for child
    let result = message_queue
        .get_messages(vec![String::from("test")], Some(message_id))
        .await;
    assert_eq!(
        result.len(),
        0,
        "Must return no messages from child after last sent message"
    );

    stop_message_queue(message_queue).await;
}

#[tokio::test]
pub async fn message_queue_multiple() {
    let message_queue = start_message_queue(None).await;

    // send first message to root
    message_queue.send(vec![], String::from("test1")).await;

    // ensure message is received
    let result = message_queue.get_messages(vec![], None).await;
    assert_eq!(result.len(), 1, "1 message must be received");
    assert_eq!(
        result[0].message,
        String::from("test1"),
        "Message \"test1\" must be received"
    );

    let message_1_id = result[0].message_id;

    // send second message to root
    message_queue.send(vec![], String::from("test2")).await;

    // ensure both messages are in queue in order
    let result = message_queue.get_messages(vec![], None).await;
    assert_eq!(result.len(), 2, "1 message must be received");
    assert_eq!(
        result[0].message,
        String::from("test1"),
        "Message \"test1\" must be received first"
    );
    assert_eq!(
        result[1].message,
        String::from("test2"),
        "Message \"test2\" must be received second"
    );

    let message_2_id = result[1].message_id;

    // ensure both messages are sent when passing random message id (simulating missed message)
    let result = message_queue
        .get_messages(vec![], Some(MessageID::new()))
        .await;
    assert_eq!(result.len(), 2, "2 messages must be received");
    assert_eq!(
        result[0].message,
        String::from("test1"),
        "Message \"test1\" must be received first"
    );
    assert_eq!(
        result[1].message,
        String::from("test2"),
        "Message \"test2\" must be received second"
    );

    // ensure only second message is present when passing first message id
    let result = message_queue.get_messages(vec![], Some(message_1_id)).await;
    assert_eq!(
        result.len(),
        1,
        "1 message must be present when getting messages after first message"
    );
    assert_eq!(
        result[0].message,
        String::from("test2"),
        "Message \"test2\" must be first message when getting messages after first message"
    );

    // ensure no message is present when passing second message id
    let result = message_queue.get_messages(vec![], Some(message_2_id)).await;
    assert_eq!(
        result.len(),
        0,
        "0 messages must be present when getting messages after second message"
    );

    stop_message_queue(message_queue).await;
}

#[tokio::test]
pub async fn message_queue_wait() {
    let message_queue = start_message_queue::<String>(None).await;

    let thread_message_queue = message_queue.clone();
    let send_thread = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;

        thread_message_queue
            .send(vec![], String::from("test1"))
            .await;
    });

    let thread_message_queue = message_queue.clone();
    let recv_thread = tokio::spawn(async move {
        let result = thread_message_queue.recv(vec![], None).await;

        assert_eq!(result.len(), 1, "Did not receive any message");
        assert_eq!(
            result[0].message,
            String::from("test1"),
            "Received incorrect message"
        );
    });

    let result = tokio::join!(send_thread, recv_thread);
    assert!(result.0.is_ok(), "Send future unsuccessful");
    assert!(result.1.is_ok(), "Recv future unsuccessful");

    stop_message_queue(message_queue).await;
}
