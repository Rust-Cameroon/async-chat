//! # async-chat
//!
//! A simple async group chat system implemented in Rust, using `async-std` for concurrency.
//! This crate defines the message formats and utility functions used by both the client and server.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
pub mod utils;

/// Messages that clients can send to the server.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FromClient {
    /// Join a group by name.
    Join { group_name: Arc<String> },
    /// Post a message to a group.
    Post {
        group_name: Arc<String>,
        author: Arc<String>,
        message: Arc<String>,
    },
    RequestGroups,
    PostFile {
        group_name: Arc<String>,
        author: Arc<String>,
        filename: String,
        data: String, // Base64
    },
    PostVoice {
        group_name: Arc<String>,
        author: Arc<String>,
        duration: f64, // Duration in seconds
        data: String, // Base64 encoded audio
    },
    PostReaction {
        group_name: Arc<String>,
        author: Arc<String>,
        message_id: String,
        emoji: String,
    },
    /// Reply to a specific message
    PostReply {
        group_name: Arc<String>,
        author: Arc<String>,
        message: Arc<String>,
        reply_to_id: String,
        reply_to_author: String,
        reply_to_preview: String,
    },
    /// Set user's online status
    SetPresence {
        username: Arc<String>,
        status: UserStatus,
    },
    /// Request list of online users in a group
    RequestOnlineUsers { group_name: Arc<String> },
}

/// User online status
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UserStatus {
    Online,
    Away,
    Offline,
}
/// Messages that the server sends back to clients.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FromServer {
    /// A message has been posted to a group.
    Message {
        group_name: Arc<String>,
        author: Arc<String>,
        message: Arc<String>,
    },
    /// The server encountered an error.
    Error(String),
    File {
        group_name: Arc<String>,
        author: Arc<String>,
        filename: String,
        data: String, // Base64
    },
    Voice {
        group_name: Arc<String>,
        author: Arc<String>,
        duration: f64,
        data: String, // Base64 encoded audio
    },
    Reaction {
        group_name: Arc<String>,
        author: Arc<String>,
        message_id: String,
        emoji: String,
    },
    /// A reply to a specific message
    Reply {
        group_name: Arc<String>,
        author: Arc<String>,
        message: Arc<String>,
        reply_to_id: String,
        reply_to_author: String,
        reply_to_preview: String,
    },
    GroupsList(Vec<String>),
    /// List of online users in a group
    OnlineUsers {
        group_name: Arc<String>,
        users: Vec<OnlineUser>,
    },
    /// User presence update
    PresenceUpdate {
        username: Arc<String>,
        status: UserStatus,
    },
}

/// Online user info
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OnlineUser {
    pub username: String,
    pub status: UserStatus,
}

#[cfg(test)]
mod test {
    use crate::FromClient;

    #[test]
    fn test_fromclient_json() {
        use std::sync::Arc;
        let from_client = FromClient::Post {
            group_name: Arc::new("Dogs".to_string()),
            message: Arc::new("Samoyeds rock!".to_string()),
        };
        let json = serde_json::to_string(&from_client).unwrap();
        assert_eq!(
            json,
            r#"{"Post":{"group_name":"Dogs","message":"Samoyeds rock!"}}"#
        );
        assert_eq!(
            serde_json::from_str::<FromClient>(&json).unwrap(),
            from_client
        );
    }
}
