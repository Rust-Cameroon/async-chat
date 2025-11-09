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
    /// Adds a client connection to the group and starts sending messages to it.
    ///
    /// # Arguments
    ///
    /// * `outbound` - The client connection to receive messages.
    ///
    /// This function spawns a background task to handle receiving messages from the
    /// broadcast channel and forwarding them to the client. A task is used so that
    /// the message receiving loop can run asynchronously without blocking the caller.
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
/// Receives messages from the broadcast channel and forwards them to the client connection.
/// Exits when the client disconnects or an error occurs.
async fn handle_subscriber(
    group_name: Arc<String>,
    mut receiver: broadcast::Receiver<Arc<String>>,
    outbound: Arc<Outbound>,
) {
    use async_chat::FromServer;
    
    loop {
        match receiver.recv().await {
            Ok(message) => {
                let server_message = FromServer::Message {
                    group_name: group_name.clone(),
                    message,
                };
                
                // Send the message to the client
                if let Err(e) = outbound.send(server_message).await {
                    eprintln!("Failed to send message to client in group '{}': {}", group_name, e);
                    break; // Exit the loop if we can't send to the client
                }
            }
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                // Client was too slow, some messages were skipped
                eprintln!("Client in group '{}' lagged behind, skipped {} messages", group_name, skipped);
                
                let error_message = FromServer::Error(
                    format!("You were lagging behind and missed {} messages", skipped)
                );
                
                if let Err(e) = outbound.send(error_message).await {
                    eprintln!("Failed to send lag error to client in group '{}': {}", group_name, e);
                    break;
                }
            }
            Err(broadcast::error::RecvError::Closed) => {
                // The broadcast channel was closed (group was deleted)
                eprintln!("Broadcast channel for group '{}' was closed", group_name);
                break;
            }
        }
    }
    
    eprintln!("Subscriber handler for group '{}' exited", group_name);
}
