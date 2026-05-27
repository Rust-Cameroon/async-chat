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
    /// Create a new group with optional password.
    CreateGroup {
        group_name: Arc<String>,
        password: Option<Arc<String>>,
    },
    /// List all available groups.
    ListGroups,
    /// Join a group by name.
    Join {
        group_name: Arc<String>,
        password: Option<Arc<String>>,
    },
    /// Post a message to a group.
    Post {
        group_name: Arc<String>,
        message: Arc<String>,
    },
}
/// Messages that the server sends back to clients.
#[derive(Debug, Deserialize, Serialize)]
pub enum FromServer {
    /// Confirmation that a group was created.
    GroupCreated {
        group_name: Arc<String>,
    },
    /// List of available groups.
    GroupList {
        groups: Vec<Arc<String>>,
    },
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
    use crate::{FromClient, FromServer};

    #[test]
    fn test_fromclient_post_json() {
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

    #[test]
    fn test_fromclient_create_group_json() {
        use std::sync::Arc;
        let from_client = FromClient::CreateGroup {
            group_name: Arc::new("Cats".to_string()),
            password: None,
        };
        let json = serde_json::to_string(&from_client).unwrap();
        assert_eq!(
            json,
            r#"{"CreateGroup":{"group_name":"Cats","password":null}}"#
        );
        assert_eq!(
            serde_json::from_str::<FromClient>(&json).unwrap(),
            from_client
        );
    }

    #[test]
    fn test_fromclient_create_group_with_password_json() {
        use std::sync::Arc;
        let from_client = FromClient::CreateGroup {
            group_name: Arc::new("Secure".to_string()),
            password: Some(Arc::new("secret123".to_string())),
        };
        let json = serde_json::to_string(&from_client).unwrap();
        assert_eq!(
            json,
            r#"{"CreateGroup":{"group_name":"Secure","password":"secret123"}}"#
        );
        assert_eq!(
            serde_json::from_str::<FromClient>(&json).unwrap(),
            from_client
        );
    }

    #[test]
    fn test_fromclient_join_with_password_json() {
        use std::sync::Arc;
        let from_client = FromClient::Join {
            group_name: Arc::new("Secure".to_string()),
            password: Some(Arc::new("secret123".to_string())),
        };
        let json = serde_json::to_string(&from_client).unwrap();
        assert_eq!(
            json,
            r#"{"Join":{"group_name":"Secure","password":"secret123"}}"#
        );
        assert_eq!(
            serde_json::from_str::<FromClient>(&json).unwrap(),
            from_client
        );
    }

    #[test]
    fn test_fromclient_list_groups_json() {
        let from_client = FromClient::ListGroups;
        let json = serde_json::to_string(&from_client).unwrap();
        assert_eq!(json, r#""ListGroups""#);
        assert_eq!(
            serde_json::from_str::<FromClient>(&json).unwrap(),
            from_client
        );
    }

    #[test]
    fn test_fromserver_group_created_json() {
        use std::sync::Arc;
        let from_server = FromServer::GroupCreated {
            group_name: Arc::new("Dogs".to_string()),
        };
        let json = serde_json::to_string(&from_server).unwrap();
        assert_eq!(
            json,
            r#"{"GroupCreated":{"group_name":"Dogs"}}"#
        );
        assert_eq!(
            serde_json::from_str::<FromServer>(&json).unwrap().to_string(),
            from_server.to_string()
        );
    }

    #[test]
    fn test_fromserver_group_list_json() {
        use std::sync::Arc;
        let from_server = FromServer::GroupList {
            groups: vec![Arc::new("Dogs".to_string()), Arc::new("Cats".to_string())],
        };
        let json = serde_json::to_string(&from_server).unwrap();
        assert_eq!(
            json,
            r#"{"GroupList":{"groups":["Dogs","Cats"]}}"#
        );
    }

    #[test]
    fn test_fromserver_error_json() {
        let from_server = FromServer::Error("Something went wrong".to_string());
        let json = serde_json::to_string(&from_server).unwrap();
        assert_eq!(json, r#"{"Error":"Something went wrong"}"#);
    }

    #[test]
    fn test_fromserver_message_json() {
        use std::sync::Arc;
        let from_server = FromServer::Message {
            group_name: Arc::new("Dogs".to_string()),
            message: Arc::new("Samoyeds rock!".to_string()),
        };
        let json = serde_json::to_string(&from_server).unwrap();
        assert_eq!(
            json,
            r#"{"Message":{"group_name":"Dogs","message":"Samoyeds rock!"}}"#
        );
    }
}
