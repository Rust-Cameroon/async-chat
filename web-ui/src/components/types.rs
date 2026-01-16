use std::collections::HashMap;
use std::rc::Rc;
use yew::prelude::*;

/// Content types for chat messages
#[derive(Clone, PartialEq)]
pub enum MessageContent {
    Text(String),
    File { filename: String, data: String },
    Voice { duration: f64, data: String },
}

/// Reply information for threaded messages
#[derive(Clone, PartialEq)]
pub struct ReplyInfo {
    pub message_id: String,
    pub author: String,
    pub preview: String,
}

/// A single chat message
#[derive(Clone, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub author: String,
    pub content: MessageContent,
    pub is_self: bool,
    pub is_error: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub reactions: Vec<(String, String)>, // (emoji, user_name)
    pub reply_to: Option<ReplyInfo>,
}

/// Online user information
#[derive(Clone, PartialEq)]
pub struct OnlineUser {
    pub username: String,
    pub status: String, // "Online", "Away", "Offline"
}

/// Actions that can modify chat state
pub enum ChatAction {
    AddMessage(ChatMessage),
    SetGroups(Vec<String>),
    Clear,
    AddReaction { msg_index: usize, emoji: String, user: String },
    SetTypingUsers(Vec<String>),
    SetOnlineUsers(Vec<OnlineUser>),
    UpdateUserPresence { username: String, status: String },
    SwitchGroup { group_name: String },
    AddMessageToGroup { group_name: String, message: ChatMessage },
}

/// Messages for a specific group
#[derive(Clone, PartialEq, Default)]
pub struct GroupMessages {
    pub messages: Vec<ChatMessage>,
    pub online_users: Vec<OnlineUser>,
}

/// Main chat state
#[derive(Clone, PartialEq)]
pub struct ChatState {
    pub current_group: Option<String>,
    pub groups: Vec<String>,
    pub group_messages: HashMap<String, GroupMessages>,
    pub typing_users: Vec<String>,
    pub online_users: Vec<OnlineUser>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            current_group: None,
            groups: Vec::new(),
            group_messages: HashMap::new(),
            typing_users: Vec::new(),
            online_users: Vec::new(),
        }
    }
}

impl Reducible for ChatState {
    type Action = ChatAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current_group = self.current_group.clone();
        let mut groups = self.groups.clone();
        let mut group_messages = self.group_messages.clone();
        let mut typing_users = self.typing_users.clone();
        let mut online_users = self.online_users.clone();
        
        match action {
            ChatAction::AddMessage(msg) => {
                // Add to current group's messages
                if let Some(ref group) = current_group {
                    let entry = group_messages.entry(group.clone()).or_default();
                    entry.messages.push(msg);
                }
            }
            ChatAction::SetGroups(g) => {
                groups = g;
            }
            ChatAction::Clear => {
                if let Some(ref group) = current_group {
                    if let Some(entry) = group_messages.get_mut(group) {
                        entry.messages.clear();
                    }
                }
            }
            ChatAction::AddReaction { msg_index, emoji, user } => {
                if let Some(ref group) = current_group {
                    if let Some(entry) = group_messages.get_mut(group) {
                        if let Some(msg) = entry.messages.get_mut(msg_index) {
                            if let Some(pos) = msg.reactions.iter().position(|(e, u)| e == &emoji && u == &user) {
                                msg.reactions.remove(pos);
                            } else {
                                msg.reactions.push((emoji, user));
                            }
                        }
                    }
                }
            }
            ChatAction::SetTypingUsers(users) => {
                typing_users = users;
            }
            ChatAction::SetOnlineUsers(users) => {
                online_users = users;
            }
            ChatAction::UpdateUserPresence { username, status } => {
                if let Some(user) = online_users.iter_mut().find(|u| u.username == username) {
                    user.status = status;
                } else {
                    online_users.push(OnlineUser { username, status });
                }
            }
            ChatAction::SwitchGroup { group_name } => {
                current_group = Some(group_name.clone());
                // Initialize group if not exists
                group_messages.entry(group_name).or_default();
            }
            ChatAction::AddMessageToGroup { group_name, message } => {
                let entry = group_messages.entry(group_name).or_default();
                entry.messages.push(message);
            }
        }
        
        Self { 
            current_group,
            groups, 
            group_messages,
            typing_users, 
            online_users 
        }.into()
    }
}

/// Get messages for the current group
impl ChatState {
    pub fn current_messages(&self) -> &[ChatMessage] {
        if let Some(ref group) = self.current_group {
            if let Some(entry) = self.group_messages.get(group) {
                return &entry.messages;
            }
        }
        &[]
    }
}
