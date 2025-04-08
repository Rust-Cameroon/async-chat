use std::sync::Arc;

use serde::{Deserialize, Serialize};
pub mod utils;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub username: Arc<String>,
    pub id: Arc<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum FromClient {
    Join {
        group_name: Arc<String>,
    },
    Post {
        group_name: Arc<String>,
        message: Arc<String>,
    },
    Register {
        username: Arc<String>,
        password: Arc<String>,
    },
    Login {
        username: Arc<String>,
        password: Arc<String>,
    },
    Logout,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FromServer {
    Message {
        group_name: Arc<String>,
        message: Arc<String>,
        sender: User,
    },
    Error(String),
    AuthSuccess {
        user: User,
    },
    AuthError(String),
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
