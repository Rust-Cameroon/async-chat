#![allow(dead_code)] // Suppresses warnings about unused code

use crate::connection::Outbound;
use async_std::task;
use std::sync::Arc;
use tokio::sync::broadcast;

/// A packet sent over the group's broadcast channel.
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub author: Arc<String>,
    pub message: Arc<String>,
}

/// A named group that broadcasts messages to all connected subscribers.
pub struct Group {
    name: Arc<String>,
    sender: broadcast::Sender<ChatMessage>,
}

impl Group {
    /// Creates a new `Group` with a given name.
    pub fn new(name: Arc<String>) -> Group {
        let (sender, _receiver) = broadcast::channel(1000); // buffer size of 1000 messages
        Group { name, sender }
    }

    /// Adds a client connection to the group and starts sending messages to it.
    pub fn join(&self, outbound: Arc<Outbound>) {
        let receiver = self.sender.subscribe();
        task::spawn(handle_subscriber(self.name.clone(), receiver, outbound));
    }

    /// Posts a message to the group, broadcasting it to all subscribers.
    pub fn post(&self, author: Arc<String>, message: Arc<String>) {
        eprintln!("Server: Group '{}' broadcasting message from '{}'", self.name, author);
        let _ = self.sender.send(ChatMessage { author, message });
    }
}

/// Handles the lifecycle of a subscriber: receiving messages and sending them over their connection.
async fn handle_subscriber(
    group_name: Arc<String>,
    mut receiver: broadcast::Receiver<ChatMessage>,
    outbound: Arc<Outbound>,
) {
    use async_chat::FromServer;

    loop {
        match receiver.recv().await {
            Ok(packet) => {
                eprintln!("Server: Subscriber in '{}' received message from '{}'", group_name, packet.author);
                let server_message = FromServer::Message {
                    group_name: group_name.clone(),
                    author: packet.author,
                    message: packet.message,
                };

                // Send the message to the client
                if let Err(e) = outbound.send(server_message).await {
                    eprintln!(
                        "Failed to send message to client in group '{}': {}",
                        group_name, e
                    );
                    break; 
                }
            }
// ... (rest remains same)
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                // Client was too slow, some messages were skipped
                eprintln!(
                    "Client in group '{}' lagged behind, skipped {} messages",
                    group_name, skipped
                );

                let error_message = FromServer::Error(format!(
                    "You were lagging behind and missed {} messages",
                    skipped
                ));

                if let Err(e) = outbound.send(error_message).await {
                    eprintln!(
                        "Failed to send lag error to client in group '{}': {}",
                        group_name, e
                    );
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
