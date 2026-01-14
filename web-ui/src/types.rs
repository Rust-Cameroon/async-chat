use async_chat::{FromClient, FromServer};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Enhanced chat message with additional UI information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub group_name: String,
    pub content: String,
    pub sender: String,
    pub timestamp: DateTime<Utc>,
    pub is_own: bool,
}

impl ChatMessage {
    pub fn new(group_name: String, content: String, sender: String, is_own: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            group_name,
            content,
            sender,
            timestamp: Utc::now(),
            is_own,
        }
    }

    pub fn from_server_message(
        group_name: String,
        message: String,
        sender: String,
        current_user: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            group_name,
            content: message,
            sender,
            timestamp: Utc::now(),
            is_own: sender == current_user,
        }
    }
}

/// Chat group information for UI display
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatGroup {
    pub name: String,
    pub member_count: usize,
    pub last_message: Option<String>,
    pub last_activity: DateTime<Utc>,
    pub unread_count: usize,
}

impl ChatGroup {
    pub fn new(name: String) -> Self {
        Self {
            name,
            member_count: 1,
            last_message: None,
            last_activity: Utc::now(),
            unread_count: 0,
        }
    }

    pub fn update_activity(&mut self, message: &str) {
        self.last_message = Some(message.to_string());
        self.last_activity = Utc::now();
    }

    pub fn increment_unread(&mut self) {
        self.unread_count += 1;
    }

    pub fn clear_unread(&mut self) {
        self.unread_count = 0;
    }
}

/// WebSocket connection status
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
    Error(String),
}

impl ConnectionStatus {
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionStatus::Connected)
    }

    pub fn to_string(&self) -> String {
        match self {
            ConnectionStatus::Connecting => "Connecting...".to_string(),
            ConnectionStatus::Connected => "Connected".to_string(),
            ConnectionStatus::Disconnected => "Disconnected".to_string(),
            ConnectionStatus::Error(msg) => format!("Error: {}", msg),
        }
    }
}

/// Application-wide state
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_user: String,
    pub current_group: Option<String>,
    pub connection_status: ConnectionStatus,
    pub groups: Vec<ChatGroup>,
    pub messages: Vec<ChatMessage>,
    pub theme: Theme,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_user: format!("User_{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            current_group: None,
            connection_status: ConnectionStatus::Disconnected,
            groups: Vec::new(),
            messages: Vec::new(),
            theme: Theme::Dark,
        }
    }
}

/// Theme settings
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn toggle(&self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }
}

/// Application actions for state management
#[derive(Debug, Clone)]
pub enum AppAction {
    SetUser(String),
    SetConnectionStatus(ConnectionStatus),
    JoinGroup(String),
    LeaveGroup(String),
    AddMessage(ChatMessage),
    SetCurrentGroup(Option<String>),
    UpdateGroup(ChatGroup),
    ClearUnread(String),
    SetTheme(Theme),
}

/// Message types for WebSocket communication with enhanced data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnhancedFromClient {
    /// Join a group with user identification
    Join {
        group_name: String,
        user_id: String,
        username: String,
    },
    /// Post a message to a group
    Post {
        group_name: String,
        message: String,
        user_id: String,
        username: String,
    },
    /// Leave a group
    Leave { group_name: String, user_id: String },
    /// Get list of active groups
    GetGroups,
    /// Typing indicator
    Typing {
        group_name: String,
        username: String,
        is_typing: bool,
    },
}

impl From<EnhancedFromClient> for FromClient {
    fn from(enhanced: EnhancedFromClient) -> Self {
        match enhanced {
            EnhancedFromClient::Join { group_name, .. } => FromClient::Join {
                group_name: std::sync::Arc::new(group_name),
            },
            EnhancedFromClient::Post {
                group_name,
                message,
                ..
            } => FromClient::Post {
                group_name: std::sync::Arc::new(group_name),
                message: std::sync::Arc::new(message),
            },
            _ => FromClient::Join {
                group_name: std::sync::Arc::new("unknown".to_string()),
            }, // Fallback
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnhancedFromServer {
    /// Enhanced message with sender information
    Message {
        group_name: String,
        message: String,
        sender: String,
        timestamp: DateTime<Utc>,
        message_id: String,
    },
    /// User joined a group
    UserJoined {
        group_name: String,
        username: String,
    },
    /// User left a group
    UserLeft {
        group_name: String,
        username: String,
    },
    /// List of available groups
    GroupsList(Vec<ChatGroup>),
    /// Group member count update
    GroupUpdate {
        group_name: String,
        member_count: usize,
    },
    /// User is typing
    UserTyping {
        group_name: String,
        username: String,
    },
    /// Server error
    Error(String),
    /// Connection successful
    Connected { user_id: String, message: String },
}

impl From<FromServer> for EnhancedFromServer {
    fn from(server: FromServer) -> Self {
        match server {
            FromServer::Message {
                group_name,
                message,
            } => {
                EnhancedFromServer::Message {
                    group_name: (*group_name).clone(),
                    message: (*message).clone(),
                    sender: "Unknown".to_string(), // Server doesn't send sender info yet
                    timestamp: Utc::now(),
                    message_id: Uuid::new_v4().to_string(),
                }
            }
            FromServer::Error(msg) => EnhancedFromServer::Error(msg),
        }
    }
}
