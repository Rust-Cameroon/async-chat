#![allow(dead_code)] // Suppresses warnings about unused code

use crate::connection::Outbound;
use async_std::task;
use std::sync::Arc;
use tokio::sync::broadcast;

/// A packet sent over the group's broadcast channel.
#[derive(Clone, Debug)]
pub enum BroadcastPacket {
    Message {
        author: Arc<String>,
        message: Arc<String>,
    },
    File {
        author: Arc<String>,
        filename: String,
        data: String,
    },
    Voice {
        author: Arc<String>,
        duration: f64,
        data: String,
    },
    Reaction {
        author: Arc<String>,
        message_id: String,
        emoji: String,
    },
    Reply {
        author: Arc<String>,
        message: Arc<String>,
        reply_to_id: String,
        reply_to_author: String,
        reply_to_preview: String,
    },
    PresenceUpdate {
        username: Arc<String>,
        status: async_chat::UserStatus,
    },
}

/// A named group that broadcasts messages to all connected subscribers.
pub struct Group {
    name: Arc<String>,
    sender: broadcast::Sender<BroadcastPacket>,
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
        let _ = self.sender.send(BroadcastPacket::Message { author, message });
    }

    pub fn post_file(&self, author: Arc<String>, filename: String, data: String) {
        eprintln!("Server: Group '{}' broadcasting file '{}' from '{}'", self.name, filename, author);
        let _ = self.sender.send(BroadcastPacket::File { author, filename, data });
    }

    pub fn post_voice(&self, author: Arc<String>, duration: f64, data: String) {
        eprintln!("Server: Group '{}' broadcasting voice message ({:.1}s) from '{}'", self.name, duration, author);
        let _ = self.sender.send(BroadcastPacket::Voice { author, duration, data });
    }

    pub fn post_reaction(&self, author: Arc<String>, message_id: String, emoji: String) {
        eprintln!("Server: Group '{}' broadcasting reaction '{}' from '{}' to message '{}'", self.name, emoji, author, message_id);
        let _ = self.sender.send(BroadcastPacket::Reaction { author, message_id, emoji });
    }

    pub fn post_reply(&self, author: Arc<String>, message: Arc<String>, reply_to_id: String, reply_to_author: String, reply_to_preview: String) {
        eprintln!("Server: Group '{}' broadcasting reply from '{}' to message by '{}'", self.name, author, reply_to_author);
        let _ = self.sender.send(BroadcastPacket::Reply { author, message, reply_to_id, reply_to_author, reply_to_preview });
    }

    pub fn broadcast_presence(&self, username: Arc<String>, status: async_chat::UserStatus) {
        eprintln!("Server: Group '{}' broadcasting presence update for '{}': {:?}", self.name, username, status);
        let _ = self.sender.send(BroadcastPacket::PresenceUpdate { username, status });
    }
}

/// Handles the lifecycle of a subscriber: receiving messages and sending them over their connection.
async fn handle_subscriber(
    group_name: Arc<String>,
    mut receiver: broadcast::Receiver<BroadcastPacket>,
    outbound: Arc<Outbound>,
) {
    use async_chat::FromServer;

    loop {
        match receiver.recv().await {
            Ok(packet) => {
                let server_message = match packet {
                    BroadcastPacket::Message { author, message } => FromServer::Message {
                        group_name: group_name.clone(),
                        author,
                        message,
                    },
                    BroadcastPacket::File { author, filename, data } => FromServer::File {
                        group_name: group_name.clone(),
                        author,
                        filename,
                        data,
                    },
                    BroadcastPacket::Voice { author, duration, data } => FromServer::Voice {
                        group_name: group_name.clone(),
                        author,
                        duration,
                        data,
                    },
                    BroadcastPacket::Reaction { author, message_id, emoji } => FromServer::Reaction {
                        group_name: group_name.clone(),
                        author,
                        message_id,
                        emoji,
                    },
                    BroadcastPacket::Reply { author, message, reply_to_id, reply_to_author, reply_to_preview } => FromServer::Reply {
                        group_name: group_name.clone(),
                        author,
                        message,
                        reply_to_id,
                        reply_to_author,
                        reply_to_preview,
                    },
                    BroadcastPacket::PresenceUpdate { username, status } => FromServer::PresenceUpdate {
                        username,
                        status,
                    },
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
