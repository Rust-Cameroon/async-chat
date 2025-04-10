//! # async-chat
//!
//! A simple async group chat system implemented in Rust, using `async-std` for concurrency.
//! This crate defines the message formats and utility functions used by both the client and server.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
pub mod utils;

/// Messages that clients can send to the server.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum FromClient {
    /// Join a group by name.
    Join { group_name: Arc<String> },
    /// Post a message to a group.
    Post {
        group_name: Arc<String>,
        message: Arc<String>,
    },
}
/// Messages that the server sends back to clients.
#[derive(Debug, Deserialize, Serialize)]
pub enum FromServer {
    /// A message has been posted to a group.
    Message {
        group_name: Arc<String>,
        message: Arc<String>,
    },
    /// The server encountered an error.
    Error(String),
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
