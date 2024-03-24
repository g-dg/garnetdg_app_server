use std::sync::mpsc;

use tokio::sync::{mpsc as mpsc_async, oneshot as oneshot_async};

pub enum MPSCSender<T> {
    Sync(mpsc::Sender<T>),
    Async(mpsc_async::UnboundedSender<T>),
}
impl<T> MPSCSender<T> {
    pub fn send(&self, value: T) -> Result<(), T> {
        match self {
            Self::Sync(channel) => channel.send(value).map_err(|x| x.0),
            Self::Async(channel) => channel.send(value).map_err(|x| x.0),
        }
    }
}

pub enum OneshotSender<T> {
    Sync(mpsc::Sender<T>),
    Async(oneshot_async::Sender<T>),
}
impl<T> OneshotSender<T> {
    pub fn send(self, value: T) -> Result<(), T> {
        match self {
            Self::Sync(channel) => channel.send(value).map_err(|x| x.0),
            Self::Async(channel) => channel.send(value),
        }
    }
}
