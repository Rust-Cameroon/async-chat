#![allow(dead_code)] // Suppresses warnings about unused code

use crate::connection::Outbound;
use async_std::task;
use std::sync::Arc;
use tokio::sync::broadcast;

/// A named group that broadcasts messages to all connected subscribers.
pub struct Group {
    name: Arc<String>,
    sender: broadcast::Sender<Arc<String>>,
}

impl Group {
    /// Creates a new `Group` with a given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the group.
    pub fn new(name: Arc<String>) -> Group {
        let (sender, _receiver) = broadcast::channel(1000); // buffer size of 1000 messages
        Group { name, sender }
    }
    /// Adds a new outbound client to the group, subscribing them to messages.
    ///
    /// # Arguments
    ///
    /// * `outbound` - The client connection to receive messages.
    pub fn join(&self, outbound: Arc<Outbound>) {
        let receiver = self.sender.subscribe();
        task::spawn(handle_subscriber(self.name.clone(), receiver, outbound));
    }
    /// Posts a message to the group, broadcasting it to all subscribers.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to broadcast.
    pub fn post(&self, message: Arc<String>) {
        let _ = self.sender.send(message); // Ignoring the result to suppress warning
    }
}

/// Handles the lifecycle of a subscriber: receiving messages and sending them over their connection.
///
/// This is a stub — should be implemented to read from the `receiver` and forward messages to `outbound`.
async fn handle_subscriber(
    _group_name: Arc<String>,
    _receiver: broadcast::Receiver<Arc<String>>,
    _outbound: Arc<Outbound>,
) {
    todo!()
}
