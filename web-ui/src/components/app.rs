use yew::prelude::*;
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use futures_util::{StreamExt, SinkExt};
use wasm_bindgen_futures::spawn_local;
use async_chat::{FromClient, FromServer};
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use futures::channel::mpsc;
use web_sys::HtmlInputElement;
use stylist::yew::styled_component;
use stylist::css;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use super::audio;

#[derive(Clone, PartialEq)]
enum MessageContent {
    Text(String),
    File { filename: String, data: String },
    Voice { duration: f64, data: String },
}

#[derive(Clone, PartialEq)]
struct ChatMessage {
    id: String,
    author: String,
    content: MessageContent,
    is_self: bool,
    is_error: bool,
    timestamp: chrono::DateTime<chrono::Utc>,
    reactions: Vec<(String, String)>, // (emoji, user_name)
    reply_to: Option<ReplyInfo>, // If this is a reply to another message
}

#[derive(Clone, PartialEq)]
struct ReplyInfo {
    message_id: String,
    author: String,
    preview: String,
}

#[derive(Clone, PartialEq)]
struct OnlineUser {
    username: String,
    status: String, // "online", "away", "offline"
}

enum ChatAction {
    AddMessage(ChatMessage),
    SetGroups(Vec<String>),
    Clear,
    AddReaction { msg_index: usize, emoji: String, user: String },
    SetTypingUsers(Vec<String>),
    SetOnlineUsers(Vec<OnlineUser>),
    UpdateUserPresence { username: String, status: String },
    SwitchGroup { group_name: String },
}

#[derive(Clone, PartialEq, Default)]
struct GroupData {
    messages: Vec<ChatMessage>,
    online_users: Vec<OnlineUser>,
}

struct ChatState {
    current_group: Option<String>,
    groups: Vec<String>,
    group_data: HashMap<String, GroupData>,
    typing_users: Vec<String>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            current_group: None,
            groups: Vec::new(),
            group_data: HashMap::new(),
            typing_users: Vec::new(),
        }
    }
}

impl ChatState {
    fn current_messages(&self) -> Vec<ChatMessage> {
        self.current_group.as_ref()
            .and_then(|g| self.group_data.get(g))
            .map(|d| d.messages.clone())
            .unwrap_or_default()
    }
    
    fn current_online_users(&self) -> Vec<OnlineUser> {
        self.current_group.as_ref()
            .and_then(|g| self.group_data.get(g))
            .map(|d| d.online_users.clone())
            .unwrap_or_default()
    }
}

impl Reducible for ChatState {
    type Action = ChatAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current_group = self.current_group.clone();
        let mut groups = self.groups.clone();
        let mut group_data = self.group_data.clone();
        let mut typing_users = self.typing_users.clone();
        
        match action {
            ChatAction::AddMessage(msg) => {
                if let Some(ref group) = current_group {
                    let data = group_data.entry(group.clone()).or_default();
                    data.messages.push(msg);
                }
            }
            ChatAction::SetGroups(g) => {
                groups = g;
            }
            ChatAction::Clear => {
                if let Some(ref group) = current_group {
                    if let Some(data) = group_data.get_mut(group) {
                        data.messages.clear();
                    }
                }
            }
            ChatAction::AddReaction { msg_index, emoji, user } => {
                if let Some(ref group) = current_group {
                    if let Some(data) = group_data.get_mut(group) {
                        if let Some(msg) = data.messages.get_mut(msg_index) {
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
                if let Some(ref group) = current_group {
                    let data = group_data.entry(group.clone()).or_default();
                    data.online_users = users;
                }
            }
            ChatAction::UpdateUserPresence { username, status } => {
                if let Some(ref group) = current_group {
                    let data = group_data.entry(group.clone()).or_default();
                    if let Some(user) = data.online_users.iter_mut().find(|u| u.username == username) {
                        user.status = status;
                    } else {
                        data.online_users.push(OnlineUser { username, status });
                    }
                }
            }
            ChatAction::SwitchGroup { group_name } => {
                current_group = Some(group_name.clone());
                group_data.entry(group_name).or_default();
            }
        }
        Self { current_group, groups, group_data, typing_users }.into()
    }
}

#[styled_component(App)]
pub fn app() -> Html {
    let chat_state = use_reducer(ChatState::default);
    let reply_to_message = use_state(|| None::<(String, String, String)>); // (id, author, preview)
    let favorites = use_state(|| Vec::<String>::new()); // List of favorite group names
    let input_ref = use_node_ref();
    let group_ref = use_node_ref();
    let name_ref = use_node_ref();
    let chat_box_ref = use_node_ref();
    let connected = use_state(|| false);
    
    let tx = use_state(|| None::<mpsc::UnboundedSender<FromClient>>);
    
    let left_sidebar_visible = use_state(|| true);
    let right_sidebar_visible = use_state(|| true);
    let is_recording = use_state(|| false);
    let my_name_state = use_state(|| "Me".to_string());
    let dark_mode = use_state(|| false);
    let is_typing = use_state(|| false);
    let notifications_enabled = use_state(|| false);
    let show_emoji_picker = use_state(|| false);
    let recording_state = use_state(|| None::<(web_sys::MediaRecorder, Vec<web_sys::Blob>, f64)>); // (recorder, chunks, start_time)
    let search_query = use_state(|| String::new());
    
    // Request notification permission on mount
    {
        let notifications_enabled = notifications_enabled.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Some(window) = web_sys::window() {
                    if let Ok(notification) = js_sys::Reflect::get(&window, &"Notification".into()) {
                        if !notification.is_undefined() {
                            let permission = js_sys::Reflect::get(&notification, &"permission".into())
                                .ok()
                                .and_then(|p| p.as_string())
                                .unwrap_or_default();
                            
                            if permission == "granted" {
                                notifications_enabled.set(true);
                            } else if permission == "default" {
                                // Request permission
                                if let Ok(request_fn) = js_sys::Reflect::get(&notification, &"requestPermission".into()) {
                                    if let Some(request) = request_fn.dyn_ref::<js_sys::Function>() {
                                        let _ = request.call0(&notification);
                                    }
                                }
                            }
                        }
                    }
                }
            });
            || ()
        });
    }

    let on_name_input = {
        let my_name_state = my_name_state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let val = input.value();
            my_name_state.set(if val.trim().is_empty() { "Me".to_string() } else { val });
        })
    };

    // Auto-scroll effect
    {
        let chat_box_ref = chat_box_ref.clone();
        let messages_len = chat_state.current_messages().len();
        use_effect_with(messages_len, move |_| {
            if let Some(div) = chat_box_ref.cast::<web_sys::HtmlElement>() {
                div.set_scroll_top(div.scroll_height());
            }
            || ()
        });
    }

    // Effect to request groups periodically
    {
        let tx = tx.clone();
        let connected = *connected;
        use_effect_with(connected, move |connected| {
            let mut interval = None;
            if *connected {
                let tx = tx.clone();
                let handle = gloo_timers::callback::Interval::new(5000, move || {
                    if let Some(sender) = &*tx {
                        let _ = sender.unbounded_send(FromClient::RequestGroups);
                    }
                });
                interval = Some(handle);
            }
            move || { drop(interval); }
        });
    }

    let on_join = {
        let group_ref = group_ref.clone();
        let name_ref = name_ref.clone();
        let chat_state = chat_state.clone();
        let connected = connected.clone();
        let tx = tx.clone();
        
        Callback::from(move |_: MouseEvent| {
            let group_name = group_ref.cast::<HtmlInputElement>().expect("input exists").value().trim().to_string();
            let user_name = name_ref.cast::<HtmlInputElement>().expect("name exists").value().trim().to_string();
            let my_name = if user_name.is_empty() { "Me".to_string() } else { user_name.clone() };

            if group_name.is_empty() { return; }
            
            // Switch to the new group (preserves history of other groups)
            chat_state.dispatch(ChatAction::SwitchGroup { 
                group_name: group_name.clone() 
            });
            
            // Add current user to online users list
            chat_state.dispatch(ChatAction::UpdateUserPresence {
                username: my_name.clone(),
                status: "Online".to_string(),
            });
            
            if let Some(sender) = &*tx {
                let _ = sender.unbounded_send(FromClient::Join { group_name: Arc::new(group_name) });
                return;
            }

            let chat_state = chat_state.clone();
            let connected = connected.clone();
            let tx_handle = tx.clone();
            let my_name_captured = my_name.clone();
            
            spawn_local(async move {
                let ws = match WebSocket::open("ws://127.0.0.1:8000") {
                    Ok(ws) => ws,
                    Err(e) => {
                        chat_state.dispatch(ChatAction::AddMessage(ChatMessage {
                            id: uuid::Uuid::new_v4().to_string(),
                            author: "System".to_string(),
                            content: MessageContent::Text(format!("Connection error: {:?}", e)),
                            is_self: false,
                            is_error: true,
                            timestamp: chrono::Utc::now(),
                            reactions: Vec::new(),
                                                reply_to: None,
                        }));
                        return;
                    }
                };
                
                connected.set(true);
                
                let (mut sink, mut stream) = ws.split();
                let (sender, mut receiver) = mpsc::unbounded::<FromClient>();
                tx_handle.set(Some(sender));

                let join_msg = FromClient::Join { group_name: Arc::new(group_name) };
                let _ = sink.send(Message::Text(serde_json::to_string(&join_msg).unwrap())).await;

                let chat_state_listener = chat_state.clone();
                let connected_listener = connected.clone();
                let tx_listener = tx_handle.clone();
                spawn_local(async move {
                    while let Some(msg_result) = stream.next().await {
                        match msg_result {
                            Ok(Message::Text(text)) => {
                                if let Ok(server_msg) = serde_json::from_str::<FromServer>(&text) {
                                    match server_msg {
                                        FromServer::Message { group_name: _, author, message } => {
                                            let is_self = author.to_string() == my_name_captured;
                                            let author_str = author.to_string();
                                            
                                            // Add/update user in online users list
                                            chat_state_listener.dispatch(ChatAction::UpdateUserPresence {
                                                username: author_str.clone(),
                                                status: "Online".to_string(),
                                            });
                                            
                                            chat_state_listener.dispatch(ChatAction::AddMessage(ChatMessage {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                is_self,
                                                author: author_str,
                                                content: MessageContent::Text(message.to_string()),
                                                is_error: false,
                                                timestamp: chrono::Utc::now(),
                                                reactions: Vec::new(),
                                                reply_to: None,
                                            }));
                                            
                                            // Play notification sound and show notification for other's messages
                                            if !is_self {
                                                // Play sound
                                                if let Some(window) = web_sys::window() {
                                                    if let Ok(audio) = web_sys::HtmlAudioElement::new() {
                                                        let _ = audio.set_src("data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1fdJivrJBhNjVgodDbq2EcBj+a2/LDciUFLIHO8tiJNwgZaLvt559NEAxQp+PwtmMcBjiR1/LMeSwFJHfH8N2QQAoUXrTp66hVFApGn+DyvmwhBTGH0fPTgjMGHm7A7+OZUQ0PVanm8LJeGAg+ltryy3k0Bip+zPLaizsKGGS57OShVhMJT6Lh8bllHAU2kdb0zHo1Bit+zPDbjDwLFmm57+idUQwPWKzn7LFjGgk9l9vyyXo1Byt9zPDdjTwLFmm37+meUgwPWKzm7LFjGgk+mNryx3w2CCt8y+/dkD0MFmq37uidUw0PWavl7LNkGQk9mNvxx342CCx8yu/ckT4MFmq47+idUw0PWqzm7LBiGgk9l9vxx302CCt8y+/dkT4MFmq47+idUw0PWqzm7LBiGgo9l9vxx302CCt8y+/dkT4MFmq47+idUw0PWqzm7LBiGgo9l9vxx302CCt8y+/dkT4MFmq47+idUw0PWqzm7LBiGgo9l9vxx302CCt8y+/dkT4MFmq47+idUw0PWqzm7LBiGgo9l9vxx302CCt8y+/dkT4MFmq47+idUw0PWqzm7LBiGgo9l9vxx302CCt8y+/dkT4MFmq47+idUw0PWqzm7LBiGgo9l9vxx302CCt8y+/dkT4MFmq47+idUw0PWqzm7LBiGgo9l9vxx302CCt8y+/dkT4MFmq47+idUw0PWqzm7LBiGgo9l9vxx302CCt8y+/dkT4MFmq47+idUw0PWqzm7LBiGgo9l9vxx302CCt8y+/dkT4MFmq47+idUw0PWqzm7LBiGgo9l9vxx302CCt8y+/dkT4M");
                                                        let _ = audio.play();
                                                    }
                                                    
                                                    // Show desktop notification
                                                    if let Ok(notification) = js_sys::Reflect::get(&window, &"Notification".into()) {
                                                        if !notification.is_undefined() {
                                                            let permission = js_sys::Reflect::get(&notification, &"permission".into())
                                                                .ok()
                                                                .and_then(|p| p.as_string())
                                                                .unwrap_or_default();
                                                            
                                                            if permission == "granted" {
                                                                // Simple notification without options for compatibility
                                                                let title = format!("New message from {}", author);
                                                                let body_text = format!("{}", message);
                                                                web_sys::console::log_1(&format!("Notification: {} - {}", title, body_text).into());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        FromServer::File { author, filename, data, .. } => {
                                            let author_str = author.to_string();
                                            chat_state_listener.dispatch(ChatAction::UpdateUserPresence {
                                                username: author_str.clone(),
                                                status: "Online".to_string(),
                                            });
                                            chat_state_listener.dispatch(ChatAction::AddMessage(ChatMessage {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                is_self: author_str == my_name_captured,
                                                author: author_str,
                                                content: MessageContent::File { filename, data },
                                                is_error: false,
                                                timestamp: chrono::Utc::now(),
                                                reactions: Vec::new(),
                                                reply_to: None,
                                            }));
                                        }
                                        FromServer::Voice { author, duration, data, .. } => {
                                            let author_str = author.to_string();
                                            chat_state_listener.dispatch(ChatAction::UpdateUserPresence {
                                                username: author_str.clone(),
                                                status: "Online".to_string(),
                                            });
                                            chat_state_listener.dispatch(ChatAction::AddMessage(ChatMessage {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                is_self: author_str == my_name_captured,
                                                author: author_str,
                                                content: MessageContent::Voice { duration, data },
                                                is_error: false,
                                                timestamp: chrono::Utc::now(),
                                                reactions: Vec::new(),
                                                reply_to: None,
                                            }));
                                        }
                                        FromServer::Reaction { message_id, emoji, author, .. } => {
                                            // Find the message and update its reactions
                                            chat_state_listener.dispatch(ChatAction::AddReaction {
                                                msg_index: chat_state_listener.messages.iter().position(|m| m.id == message_id).unwrap_or(0),
                                                emoji,
                                                user: author.to_string(),
                                            });
                                        }
                                        FromServer::GroupsList(list) => {
                                            chat_state_listener.dispatch(ChatAction::SetGroups(list));
                                        }
                                        FromServer::Error(err) => {
                                            chat_state_listener.dispatch(ChatAction::AddMessage(ChatMessage {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                author: "Error".to_string(),
                                                content: MessageContent::Text(err),
                                                is_self: false,
                                                is_error: true,
                                                timestamp: chrono::Utc::now(),
                                                reactions: Vec::new(),
                                                reply_to: None,
                                            }));
                                        }
                                        FromServer::Reply { author, message, reply_to_id, reply_to_author, reply_to_preview, .. } => {
                                            chat_state_listener.dispatch(ChatAction::AddMessage(ChatMessage {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                is_self: author.to_string() == my_name_captured,
                                                author: author.to_string(),
                                                content: MessageContent::Text(message.to_string()),
                                                is_error: false,
                                                timestamp: chrono::Utc::now(),
                                                reactions: Vec::new(),
                                                reply_to: Some(ReplyInfo {
                                                    message_id: reply_to_id,
                                                    author: reply_to_author,
                                                    preview: reply_to_preview,
                                                }),
                                            }));
                                        }
                                        FromServer::OnlineUsers { users, .. } => {
                                            let online: Vec<OnlineUser> = users.iter().map(|u| OnlineUser {
                                                username: u.username.clone(),
                                                status: format!("{:?}", u.status),
                                            }).collect();
                                            chat_state_listener.dispatch(ChatAction::SetOnlineUsers(online));
                                        }
                                        FromServer::PresenceUpdate { username, status } => {
                                            chat_state_listener.dispatch(ChatAction::UpdateUserPresence {
                                                username: username.to_string(),
                                                status: format!("{:?}", status),
                                            });
                                        }
                                    }
                                }
                            }
                            Ok(_) => (),
                            Err(_) => break,
                        }
                    }
                    connected_listener.set(false);
                    tx_listener.set(None);
                    chat_state_listener.dispatch(ChatAction::AddMessage(ChatMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        author: "System".to_string(),
                        content: MessageContent::Text("Connection lost.".to_string()),
                        is_self: false,
                        is_error: true,
                        timestamp: chrono::Utc::now(),
                        reactions: Vec::new(),
                                                reply_to: None,
                    }));
                });

                spawn_local(async move {
                    while let Some(msg) = receiver.next().await {
                        let json = serde_json::to_string(&msg).unwrap();
                        if let Err(_) = sink.send(Message::Text(json)).await {
                            break;
                        }
                    }
                });
            });
        })
    };

    let on_send = {
        let input_ref = input_ref.clone();
        let group_ref = group_ref.clone();
        let name_ref = name_ref.clone();
        let tx = tx.clone();
        Callback::from(move |_: MouseEvent| {
            let input_el = input_ref.cast::<HtmlInputElement>().expect("input exists");
            let group_el = group_ref.cast::<HtmlInputElement>().expect("group exists");
            let name_el = name_ref.cast::<HtmlInputElement>().expect("name exists");
            
            let message = input_el.value();
            let group_name = group_el.value().trim().to_string();
            let user_name = name_el.value().trim().to_string();
            
            if message.is_empty() || group_name.is_empty() { return; }
            
            if let Some(sender) = &*tx {
                let my_name = if user_name.is_empty() { "Me".to_string() } else { user_name };
                web_sys::console::log_1(&format!("UI: Sending Post to '{}' as '{}': {}", group_name, my_name, message).into());

                let post_msg = FromClient::Post { 
                    group_name: Arc::new(group_name),
                    author: Arc::new(my_name),
                    message: Arc::new(message)
                };
                if let Err(e) = sender.unbounded_send(post_msg) {
                    web_sys::console::error_1(&format!("UI Error: Failed to queue message: {:?}", e).into());
                } else {
                    web_sys::console::log_1(&"UI: Message queued successfully".into());
                    input_el.set_value("");
                }
            } else {
                web_sys::console::warn_1(&"UI Warning: Not connected (tx is None), cannot send".into());
            }
        })
    };

    let show_emojis = use_state(|| false);
    let file_input_ref = use_node_ref();

    let on_emoji_click = {
        let show_emojis = show_emojis.clone();
        Callback::from(move |_: MouseEvent| show_emojis.set(!*show_emojis))
    };

    let on_search_input = {
        let search_query = search_query.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            search_query.set(input.value().to_lowercase());
        })
    };

    let on_select_emoji = {
        let input_ref = input_ref.clone();
        let show_emojis = show_emojis.clone();
        Callback::from(move |emoji: &'static str| {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let curr = input.value();
                input.set_value(&format!("{}{}", curr, emoji));
                show_emojis.set(false);
            }
        })
    };

    // Chat button - focus input and scroll to bottom
    let on_chat_click = {
        let input_ref = input_ref.clone();
        let chat_box_ref = chat_box_ref.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let _ = input.focus();
            }
            // Scroll chat to bottom
            if let Some(chat_box) = chat_box_ref.cast::<web_sys::HtmlElement>() {
                chat_box.set_scroll_top(chat_box.scroll_height());
            }
        })
    };

    // View Friends - show left sidebar
    let on_view_friends_click = {
        let left_sidebar_visible = left_sidebar_visible.clone();
        Callback::from(move |_: MouseEvent| {
            left_sidebar_visible.set(true);
        })
    };

    // Add to Favorites
    let on_add_favorite_click = {
        let group_ref = group_ref.clone();
        let favorites = favorites.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(group_el) = group_ref.cast::<HtmlInputElement>() {
                let group_name = group_el.value().trim().to_string();
                if !group_name.is_empty() {
                    let mut current_favs = (*favorites).clone();
                    if current_favs.contains(&group_name) {
                        // Remove from favorites
                        current_favs.retain(|g| g != &group_name);
                        web_sys::window().unwrap().alert_with_message(&format!("Removed '{}' from favorites!", group_name)).ok();
                    } else {
                        // Add to favorites
                        current_favs.push(group_name.clone());
                        web_sys::window().unwrap().alert_with_message(&format!("Added '{}' to favorites!", group_name)).ok();
                    }
                    favorites.set(current_favs);
                } else {
                    web_sys::window().unwrap().alert_with_message("Please join a group first to add it to favorites!").ok();
                }
            }
        })
    };

    let on_file_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_file_change = {
        let file_input_ref = file_input_ref.clone();
        let group_ref = group_ref.clone();
        let name_ref = name_ref.clone();
        let tx = tx.clone();
        Callback::from(move |_: Event| {
            let file_input = file_input_ref.cast::<HtmlInputElement>().expect("file input exists");
            let group_el = group_ref.cast::<HtmlInputElement>().expect("group exists");
            let name_el = name_ref.cast::<HtmlInputElement>().expect("name exists");
            
            let group_name = group_el.value().trim().to_string();
            let user_name = name_el.value().trim().to_string();
            
            if group_name.is_empty() { return; }

            if let Some(files) = file_input.files() {
                if let Some(file) = files.get(0) {
                    let filename = file.name();
                    let tx = tx.clone();
                    let my_name = if user_name.is_empty() { "Me".to_string() } else { user_name };
                    
                    let reader = web_sys::FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let on_load = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                        let result = reader_clone.result().unwrap();
                        let data_url = result.as_string().unwrap();
                        
                        if let Some(sender) = &*tx {
                            let _ = sender.unbounded_send(FromClient::PostFile {
                                group_name: Arc::new(group_name.clone()),
                                author: Arc::new(my_name.clone()),
                                filename: filename.clone(),
                                data: data_url,
                            });
                        }
                    }) as Box<dyn FnMut(web_sys::Event)>);
                    
                    reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
                    reader.read_as_data_url(&file).unwrap();
                    on_load.forget();
                }
            }
        })
    };

    let toggle_left = {
        let left_sidebar_visible = left_sidebar_visible.clone();
        Callback::from(move |_: MouseEvent| left_sidebar_visible.set(!*left_sidebar_visible))
    };
    let toggle_right = {
        let right_sidebar_visible = right_sidebar_visible.clone();
        Callback::from(move |_: MouseEvent| right_sidebar_visible.set(!*right_sidebar_visible))
    };

    let toggle_recording = {
        let is_recording = is_recording.clone();
        let tx = tx.clone();
        let group_ref = group_ref.clone();
        let my_name_state = my_name_state.clone();
        let chat_state = chat_state.clone();
        
        Callback::from(move |_: MouseEvent| {
            let is_recording = is_recording.clone();
            let tx = tx.clone();
            let group_ref = group_ref.clone();
            let my_name_state = my_name_state.clone();
            let chat_state = chat_state.clone();
            
            let window = web_sys::window().unwrap();
            
            if *is_recording {
                let _ = js_sys::eval("if(window._stopRecording) window._stopRecording();");
                is_recording.set(false);
            } else {
                is_recording.set(true);
                
                let tx_clone = tx.clone();
                let group_ref_clone = group_ref.clone();
                let my_name_clone = my_name_state.clone();
                let is_recording_clone = is_recording.clone();
                let chat_state_clone = chat_state.clone();
                
                let on_done = Closure::wrap(Box::new(move |data_url: String, duration: f64| {
                    if !data_url.is_empty() {
                        if let Some(sender) = &*tx_clone {
                            if let Some(group_el) = group_ref_clone.cast::<HtmlInputElement>() {
                                let group_name = group_el.value().trim().to_string();
                                if !group_name.is_empty() {
                                    let _ = sender.unbounded_send(FromClient::PostVoice {
                                        group_name: Arc::new(group_name),
                                        author: Arc::new((*my_name_clone).clone()),
                                        duration,
                                        data: data_url.clone(),
                                    });
                                    chat_state_clone.dispatch(ChatAction::AddMessage(ChatMessage {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        author: (*my_name_clone).clone(),
                                        content: MessageContent::Voice { duration, data: data_url },
                                        is_self: true,
                                        is_error: false,
                                        timestamp: chrono::Utc::now(),
                                        reactions: Vec::new(),
                                                reply_to: None,
                                    }));
                                }
                            }
                        }
                    }
                    is_recording_clone.set(false);
                }) as Box<dyn FnMut(String, f64)>);
                
                let _ = js_sys::Reflect::set(&window, &"_onRecordingDone".into(), on_done.as_ref());
                on_done.forget();
                
                let _ = js_sys::eval(r#"
                    (async function() {
                        try {
                            const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
                            const rec = new MediaRecorder(stream);
                            const chunks = [];
                            const start = Date.now();
                            rec.ondataavailable = e => { if (e.data.size > 0) chunks.push(e.data); };
                            rec.onstop = () => {
                                const dur = (Date.now() - start) / 1000;
                                const blob = new Blob(chunks, { type: 'audio/webm' });
                                const reader = new FileReader();
                                reader.onloadend = () => {
                                    if (window._onRecordingDone) window._onRecordingDone(reader.result || '', dur);
                                    stream.getTracks().forEach(t => t.stop());
                                };
                                reader.readAsDataURL(blob);
                            };
                            window._stopRecording = () => { if (rec.state !== 'inactive') rec.stop(); };
                            rec.start();
                        } catch(e) {
                            alert('Microphone access denied. Please allow permissions.');
                            if (window._onRecordingDone) window._onRecordingDone('', 0);
                        }
                    })();
                "#);
            }
        })
    };

    let on_keypress = {
        let on_send = on_send.clone();
        let is_typing = is_typing.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                on_send.emit(MouseEvent::new("click").unwrap());
                is_typing.set(false);
            } else {
                is_typing.set(true);
            }
        })
    };

    let on_input_change = {
        let is_typing = is_typing.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            is_typing.set(!input.value().is_empty());
        })
    };

    let toggle_dark_mode = {
        let dark_mode = dark_mode.clone();
        Callback::from(move |_: MouseEvent| dark_mode.set(!*dark_mode))
    };

    // --- Styles ---

    let left_w = if *left_sidebar_visible { "300px" } else { "0" };
    let right_w = if *right_sidebar_visible { "350px" } else { "0px" };
    
    // Dark mode colors
    let bg_color = if *dark_mode { "#1a1a1a" } else { "white" };
    let text_color = if *dark_mode { "#e0e0e0" } else { "#1a1a1a" };
    let sidebar_bg = if *dark_mode { "#2d2d2d" } else { "#f7f9fa" };
    let border_color = if *dark_mode { "#3a3a3a" } else { "#e1e4e8" };
    let input_bg = if *dark_mode { "#2d2d2d" } else { "white" };
    let hover_bg = if *dark_mode { "#3a3a3a" } else { "#edf2f7" };

    let container_style = css!(r#"
        display: grid;
        grid-template-columns: ${left} 1fr ${right};
        height: 100vh;
        width: 100vw;
        font-family: 'Inter', sans-serif;
        background-color: ${bg};
        color: ${text};
        overflow: hidden;
        transition: all 0.3s ease;
        position: relative;

        @media (max-width: 1200px) {
            grid-template-columns: ${left} 1fr 0px;
        }
        @media (max-width: 800px) {
            grid-template-columns: 0px 1fr 0px;
        }
        @media (max-width: 600px) {
            font-size: 14px;
        }
    "#, left=left_w, right=right_w, bg=bg_color, text=text_color);

    // Sidebar Left Styles
    let sidebar_left_style = css!(r#"
        background-color: ${sidebar_bg};
        border-right: 1px solid ${border};
        display: flex;
        flex-direction: column;
        padding: 20px 0;
        overflow: hidden;
        transition: all 0.3s ease;
        min-width: 0;
    "#, sidebar_bg=sidebar_bg, border=border_color);

    let profile_small_style = css!(r#"
        display: flex;
        align-items: center;
        padding: 0 20px;
        gap: 12px;
        margin-bottom: 24px;
        position: relative;
    "#);

    let avatar_style = css!(r#"
        width: 48px;
        height: 48px;
        border-radius: 50%;
        background-color: #ddd;
        object-fit: cover;
    "#);

    let search_bar_container = css!(r#"
        margin: 0 20px 20px;
        position: relative;
        &::before {
            content: "🔍";
            position: absolute;
            left: 15px;
            top: 50%;
            transform: translateY(-50%);
            font-size: 0.8rem;
            opacity: 0.5;
        }
    "#);

    let search_input_style = css!(r#"
        width: 100%;
        padding: 10px 15px 10px 40px;
        border-radius: 20px;
        border: 1px solid ${border};
        background-color: ${input_bg};
        color: ${text};
        font-size: 0.9rem;
        outline: none;
        transition: all 0.2s ease;
        &:focus { border-color: #3498db; box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1); }
    "#, border=border_color, input_bg=input_bg, text=text_color);

    let contact_item_style = css!(r#"
        display: flex;
        align-items: center;
        padding: 12px 20px;
        gap: 15px;
        cursor: pointer;
        transition: all 0.2s ease;
        border-radius: 8px;
        margin: 0 10px;
        &:hover { 
            background-color: ${hover}; 
            transform: translateX(5px);
        }
        &.active { background-color: #e2e8f0; }
    "#, hover=hover_bg);

    // Main Chat Styles
    let chat_main_style = css!(r#"
        display: flex;
        flex-direction: column;
        background-color: ${bg};
        overflow: hidden;
        transition: all 0.3s ease;
    "#, bg=bg_color);

    let chat_header_style = css!(r#"
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 15px 25px;
        border-bottom: 1px solid ${border};
        transition: all 0.3s ease;
        
        @media (max-width: 600px) {
            padding: 12px 15px;
        }
    "#, border=border_color);

    let chat_messages_style = css!(r#"
        flex: 1;
        overflow-y: auto;
        padding: 20px 30px;
        display: flex;
        flex-direction: column;
        gap: 20px;
        background-color: ${bg};
        transition: all 0.3s ease;
        scroll-behavior: smooth;
        -webkit-overflow-scrolling: touch;
        
        @media (max-width: 600px) {
            padding: 15px;
            gap: 15px;
        }
    "#, bg=bg_color);

    let chat_footer_style = css!(r#"
        padding: 15px 25px 25px;
        background-color: ${footer_bg};
        transition: all 0.3s ease;
        position: relative;
        
        @media (max-width: 600px) {
            padding: 10px 15px 15px;
        }
    "#, footer_bg=if *dark_mode { "#2d2d2d" } else { "#e3f2fd" });

    let input_wrapper_style = css!(r#"
        background-color: ${input_bg};
        border-radius: 30px;
        display: flex;
        align-items: center;
        padding: 5px 10px 5px 20px;
        box-shadow: 0 2px 5px rgba(0,0,0,0.1);
        gap: 15px;
        transition: all 0.3s ease;
        input {
            flex: 1;
            border: none;
            outline: none;
            padding: 10px 0;
            font-size: 0.95rem;
            background: transparent;
            color: ${text};
        }
        
        @media (max-width: 600px) {
            padding: 5px 8px 5px 15px;
            gap: 10px;
            input {
                font-size: 0.9rem;
                padding: 8px 0;
            }
        }
    "#, input_bg=input_bg, text=text_color);

    let icon_btn_style = css!(r#"
        background: none;
        border: none;
        font-size: 1.2rem;
        cursor: pointer;
        opacity: 0.6;
        transition: opacity 0.2s;
        &:hover { opacity: 1; }
    "#);

    let send_circle_btn = css!(r#"
        width: 45px;
        height: 45px;
        background-color: #0084ff;
        color: white;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        border: none;
        cursor: pointer;
        box-shadow: 0 4px 10px rgba(0, 132, 255, 0.3);
        transition: transform 0.2s;
        &:hover { transform: scale(1.05); }
        &:active { transform: scale(0.95); }
        
        @media (max-width: 600px) {
            width: 40px;
            height: 40px;
            font-size: 0.9rem;
        }
    "#);

    // Sidebar Right Styles
    let sidebar_right_style = css!(r#"
        background-color: ${sidebar_bg};
        border-left: 1px solid ${border};
        display: flex;
        flex-direction: column;
        padding: 20px;
        overflow-y: auto;
        transition: all 0.3s ease;
        @media (max-width: 1200px) { display: none; }
    "#, sidebar_bg=sidebar_bg, border=border_color);

    let profile_large_style = css!(r#"
        display: flex;
        flex-direction: column;
        align-items: center;
        margin: 30px 0;
        text-align: center;
        h2 { margin: 15px 0 5px; font-size: 1.2rem; }
        span { opacity: 0.6; font-size: 0.85rem; }
    "#);

    let action_grid_style = css!(r#"
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 15px;
        margin: 20px 0;
    "#);

    let action_card_style = css!(r#"
        background: ${card_bg};
        padding: 15px;
        border-radius: 12px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        border: 1px solid ${border};
        cursor: pointer;
        transition: all 0.2s ease;
        &:hover { 
            box-shadow: 0 4px 12px rgba(0,0,0,0.15);
            transform: translateY(-2px);
        }
        .icon { font-size: 1.5rem; color: #0084ff; }
        .label { font-size: 0.8rem; font-weight: 500; color: ${text}; }
    "#, card_bg=input_bg, border=border_color, text=text_color);

    let attachments_section = css!(r#"
        margin-top: 30px;
    "#);

    let title_row_style = css!(r#"
        display: flex; 
        justify-content: space-between; 
        align-items: center; 
        margin-bottom: 15px; 
    "#);

    let attachment_grid = css!(r#"
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 10px;
    "#);

    let attachment_item = css!(r#"
        aspect-ratio: 1;
        background: white;
        border-radius: 8px;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 5px;
        border: 1px solid #edf2f7;
        font-size: 0.6rem;
        font-weight: 700;
        color: #0084ff;
        .icon { font-size: 1.2rem; }
        &.pdf { background-color: #eef2ff; color: #4f46e5; }
        &.video { background-color: #fff1f2; color: #e11d48; }
        &.audio { background-color: #f0fdf4; color: #16a34a; }
        &.image { background-color: #fefce8; color: #ca8a04; }
    "#);

    let bubble_base = css!(r#"
        max-width: 70%;
        padding: 12px 18px;
        border-radius: 20px;
        font-size: 0.95rem;
        line-height: 1.5;
        position: relative;
        animation: slideIn 0.3s ease;
        word-wrap: break-word;
        
        @keyframes slideIn {
            from {
                opacity: 0;
                transform: translateY(10px);
            }
            to {
                opacity: 1;
                transform: translateY(0);
            }
        }
        
        @media (max-width: 600px) {
            max-width: 85%;
            padding: 10px 14px;
            font-size: 0.9rem;
        }
    "#);

    let my_bubble = css!(r#"
        align-self: flex-end;
        background-color: #0084ff;
        color: white;
        border-bottom-right-radius: 4px;
    "#);

    let other_bubble = css!(r#"
        align-self: flex-start;
        background-color: ${other_bg};
        color: ${other_text};
        border-bottom-left-radius: 4px;
    "#, other_bg=if *dark_mode { "#3a3a3a" } else { "#f1f3f4" }, other_text=if *dark_mode { "#e0e0e0" } else { "#1a1a1a" });

    let connection_pill = css!(r#"
        font-size: 0.75rem;
        background: ${pill_bg};
        color: ${pill_text};
        padding: 4px 12px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        gap: 6px;
        transition: all 0.3s ease;
        .dot { 
            width: 8px; 
            height: 8px; 
            border-radius: 50%;
            animation: pulse 2s infinite;
        }
        .online { background-color: #2ecc71; }
        .offline { background-color: #e74c3c; }
        
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
    "#, pill_bg=if *dark_mode { "#3a3a3a" } else { "#edf2f7" }, pill_text=if *dark_mode { "#a0aec0" } else { "#4a5568" });

    html! {
        <div class={container_style}>
            <aside class={sidebar_left_style.clone()} style={format!("width: {}; {} transition: all 0.3s ease;", 
                if *left_sidebar_visible { "300px" } else { "0" },
                if *left_sidebar_visible { "" } else { "padding: 0; border: none; overflow: hidden;" }
            )}>
                <div class={profile_small_style}>
                    <img src={format!("https://ui-avatars.com/api/?name={}&background=3498db&color=fff", *my_name_state)} class={avatar_style.clone()} alt="Me" />
                    <div style="flex: 1;">
                        <div style="font-weight: 700; font-size: 0.95rem;">{ &*my_name_state }</div>
                        <div style="font-size: 0.75rem; opacity: 0.6;">{"Self Profile"}</div>
                    </div>
                    <span onclick={toggle_left.clone()} style="cursor: pointer; opacity: 0.5; font-size: 1.2rem;" title="Collapse Sidebar">{"⇠"}</span>
                </div>

                <div class={search_bar_container.clone()}>
                    <input 
                        class={search_input_style.clone()} 
                        placeholder="Search groups & contacts..." 
                        oninput={on_search_input}
                        value={(*search_query).clone()}
                    />
                </div>

                <div style="flex: 1; overflow-y: auto;">
                    { for chat_state.groups.iter()
                        .filter(|group| {
                            // Filter by search query
                            search_query.is_empty() || group.to_lowercase().contains(&*search_query)
                        })
                        .map(|group| {
                            let group_clone = group.clone();
                            let is_selected = chat_state.current_group.as_ref() == Some(group);
                            let on_group_click = {
                                let group_ref = group_ref.clone();
                                let on_join = on_join.clone();
                                Callback::from(move |_: MouseEvent| {
                                    if let Some(input) = group_ref.cast::<HtmlInputElement>() {
                                        input.set_value(&group_clone);
                                        on_join.emit(MouseEvent::new("click").unwrap());
                                    }
                                })
                            };
                            // Style for selected vs unselected group
                            let item_style = if is_selected {
                                format!("{} background: {}; border-left: 3px solid #0084ff;", 
                                    contact_item_style.to_string(),
                                    if *dark_mode { "rgba(0,132,255,0.2)" } else { "rgba(0,132,255,0.1)" }
                                )
                            } else {
                                contact_item_style.to_string()
                            };
                            html! {
                                <div onclick={on_group_click} class={item_style}>
                                    <div style="position: relative;">
                                        <img src={format!("https://ui-avatars.com/api/?name={}&background=random", group)} class={avatar_style.clone()} />
                                        { if is_selected {
                                            html! { <div style="position: absolute; bottom: 2px; right: 2px; width: 12px; height: 12px; background: #0084ff; border-radius: 50%; border: 2px solid white;"></div> }
                                        } else { html! {} }}
                                    </div>
                                    <div style="flex: 1;">
                                        <div style="display: flex; justify-content: space-between; align-items: center;">
                                            <span style={format!("font-weight: {}; font-size: 0.9rem; color: {};", 
                                                if is_selected { "700" } else { "600" },
                                                if is_selected { "#0084ff" } else { "inherit" }
                                            )}>{ group }</span>
                                            { if is_selected {
                                                html! { <span style="font-size: 0.7rem; background: #0084ff; color: white; padding: 2px 6px; border-radius: 10px;">{"Active"}</span> }
                                            } else { html! {} }}
                                        </div>
                                        <div style="font-size: 0.75rem; opacity: 0.6;">{"Public Group"}</div>
                                    </div>
                                </div>
                            }
                        })
                    }
                    
                    // Show "no results" message when searching
                    { if !search_query.is_empty() && chat_state.groups.iter().filter(|g| g.to_lowercase().contains(&*search_query)).count() == 0 {
                        html! {
                            <div style="padding: 20px; text-align: center; opacity: 0.6; font-size: 0.85rem;">
                                {"No groups found matching \""}{&*search_query}{"\""}
                            </div>
                        }
                    } else { html! {} }}
                    
                    <div style="padding: 15px 20px;">
                         <div style="font-size: 0.7rem; font-weight: 700; color: #a0aec0; margin-bottom: 10px;">{"SYSTEM CONTROLS"}</div>
                         <div style="display: flex; flex-direction: column; gap: 8px;">
                            <input ref={name_ref} oninput={on_name_input} class={css!("padding: 8px 12px; border-radius: 8px; border: 1px solid #e2e8f0; font-size: 0.8rem; outline: none;")} placeholder="Your Name" />
                            <input ref={group_ref.clone()} class={css!("padding: 8px 12px; border-radius: 8px; border: 1px solid #e2e8f0; font-size: 0.8rem; outline: none;")} placeholder="Group to Join" />
                            <button onclick={on_join} class={css!("background: #3498db; color: white; border: none; padding: 8px; border-radius: 8px; font-size: 0.8rem; cursor: pointer; font-weight: 600;")}>
                                { if *connected { "SWITCH GROUP" } else { "CONNECT" } }
                            </button>
                         </div>
                    </div>
                </div>
            </aside>

            <main class={chat_main_style}>
                <header class={chat_header_style}>
                    <div style="display: flex; align-items: center; gap: 15px;">
                        { if !*left_sidebar_visible {
                            html! { 
                                <span 
                                    onclick={toggle_left.clone()} 
                                    style="cursor: pointer; font-size: 1.3rem; margin-right: 10px; padding: 8px 12px; background: #0084ff; color: white; border-radius: 8px; display: flex; align-items: center; gap: 8px; font-weight: 500; box-shadow: 0 2px 8px rgba(0,132,255,0.3);" 
                                    title="Show Sidebar"
                                    class={css!("transition: all 0.2s; &:hover { transform: scale(1.05); background: #0073e6; }")}
                                >
                                    {"☰"}<span style="font-size: 0.85rem;">{"Menu"}</span>
                                </span> 
                            }
                        } else { html! {} }}
                        <img src="https://ui-avatars.com/api/?name=Group&background=2ecc71&color=fff" class={avatar_style.clone()} style="width: 40px; height: 40px;" />
                        <div>
                            <div style="font-weight: 700; font-size: 1rem;">{"General Chat"}</div>
                            <div class={connection_pill}>
                                <div class={classes!("dot", if *connected { "online" } else { "offline" })}></div>
                                { if *connected { "Live Connection" } else { "Disconnected" } }
                            </div>
                        </div>
                    </div>
                    <div style="display: flex; align-items: center; gap: 15px;">
                        <span 
                            onclick={toggle_dark_mode} 
                            style="cursor: pointer; font-size: 1.5rem; transition: transform 0.3s ease;"
                            title={if *dark_mode { "Light Mode" } else { "Dark Mode" }}
                        >
                            { if *dark_mode { "☀️" } else { "🌙" } }
                        </span>
                        { if !*right_sidebar_visible {
                            html! { <span onclick={toggle_right.clone()} style="cursor: pointer; font-size: 1.2rem; opacity: 0.5;" title="Show Info">{"⇠"}</span> }
                        } else { html! {} }}
                    </div>
                </header>

                <div ref={chat_box_ref} class={chat_messages_style}>
                    <div style="text-align: center; margin: 10px 0; position: relative;">
                        <hr style={format!("border: none; border-top: 1px solid {}; position: absolute; top: 50%; width: 100%; z-index: 1;", border_color)} />
                        <span style={format!("background: {}; padding: 0 15px; font-size: 0.75rem; color: #a0aec0; font-weight: 600; position: relative; z-index: 2;", bg_color)}>{"Async History"}</span>
                    </div>

                    { for chat_state.current_messages().iter().enumerate().map(|(idx, m)| {
                        let is_system = m.author == "System" || m.author == "Error" || m.is_error;
                        
                        if is_system {
                            let text = match &m.content {
                                MessageContent::Text(t) => t.clone(),
                                _ => "System error".to_string(),
                            };
                            return html! {
                                <div style="align-self: center; background: #fff5f5; color: #c53030; padding: 6px 15px; border-radius: 12px; font-size: 0.8rem; border: 1px solid #feb2b2;">
                                    { text }
                                </div>
                            };
                        }

                        let bubble_class = if m.is_self {
                            classes!(bubble_base.clone(), my_bubble.clone())
                        } else {
                            classes!(bubble_base.clone(), other_bubble.clone())
                        };

                        let formatted_time = m.timestamp.format("%H:%M").to_string();
                        let my_name = (*my_name_state).clone();
                        let msg_idx = idx;
                        
                        html! {
                            <div style={if m.is_self { "display: flex; flex-direction: row-reverse; gap: 12px;" } else { "display: flex; gap: 12px;" }}>
                                <img src={format!("https://ui-avatars.com/api/?name={}&background=random", m.author)} class={avatar_style.clone()} style="width: 32px; height: 32px;" />
                                <div style={if m.is_self { "display: flex; flex-direction: column; align-items: flex-end;" } else { "display: flex; flex-direction: column;" }}>
                                    <div style="font-size: 0.7rem; font-weight: 700; margin-bottom: 4px; opacity: 0.6;">
                                        { &m.author }
                                        <span style="margin-left: 8px; font-weight: 400; opacity: 0.8;">{ formatted_time }</span>
                                    </div>
                                    <div class={bubble_class}>
                                        // Show reply preview if this is a reply
                                        { if let Some(ref reply) = m.reply_to {
                                            html! {
                                                <div style={format!("background: {}; padding: 8px 12px; border-radius: 8px; margin-bottom: 8px; border-left: 3px solid #0084ff; font-size: 0.85rem;",
                                                    if m.is_self { "rgba(255,255,255,0.2)" } else { "rgba(0,0,0,0.05)" })}>
                                                    <div style="font-weight: 600; font-size: 0.75rem; opacity: 0.8; margin-bottom: 2px;">
                                                        {"↩ Replying to "}{&reply.author}
                                                    </div>
                                                    <div style="opacity: 0.7; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 200px;">
                                                        {&reply.preview}
                                                    </div>
                                                </div>
                                            }
                                        } else { html! {} }}
                                        { match &m.content {
                                            MessageContent::Text(text) => html! { <span>{ text }</span> },
                                            MessageContent::File { filename, data } => {
                                                if filename.ends_with(".png") || filename.ends_with(".jpg") || filename.ends_with(".jpeg") || filename.ends_with(".gif") {
                                                    html! {
                                                        <div style="display: flex; flex-direction: column; gap: 5px;">
                                                            <img src={data.clone()} style="max-width: 100%; border-radius: 8px; border: 1px solid rgba(0,0,0,0.1);" />
                                                            <span style="font-size: 0.7rem; opacity: 0.7; font-style: italic;">{ filename }</span>
                                                        </div>
                                                    }
                                                } else {
                                                    html! {
                                                        <div style="display: flex; align-items: center; gap: 10px;">
                                                            <span style="font-size: 1.5rem;">{"📄"}</span>
                                                            <div style="display: flex; flex-direction: column;">
                                                                <span style="font-weight: 600;">{ filename }</span>
                                                                <a href={data.clone()} download={filename.clone()} style="font-size: 0.75rem; color: inherit; text-decoration: underline;">{"Download"}</a>
                                                            </div>
                                                        </div>
                                                    }
                                                }
                                            }
                                            MessageContent::Voice { duration, data } => {
                                                html! {
                                                    <div style="display: flex; align-items: center; gap: 10px; min-width: 200px;">
                                                        <span style="font-size: 1.5rem;">{"🎤"}</span>
                                                        <div style="flex: 1; display: flex; flex-direction: column; gap: 5px;">
                                                            <audio controls=true src={data.clone()} style="width: 100%; height: 30px;" />
                                                            <span style="font-size: 0.7rem; opacity: 0.7;">
                                                                {format!("Voice message ({:.1}s)", duration)}
                                                            </span>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        }}
                                    </div>
                                    
                                    // Reactions display
                                    { if !m.reactions.is_empty() {
                                        html! {
                                            <div style="display: flex; gap: 5px; margin-top: 5px; flex-wrap: wrap;">
                                                { for m.reactions.iter().map(|(emoji, _user)| {
                                                    html! {
                                                        <span style={format!(
                                                            "background: {}; padding: 3px 8px; border-radius: 12px; font-size: 0.85rem; border: 1px solid {};",
                                                            if *dark_mode { "#3a3a3a" } else { "#f0f0f0" },
                                                            border_color
                                                        )}>
                                                            { emoji }
                                                        </span>
                                                    }
                                                })}
                                            </div>
                                        }
                                    } else { html! {} }}
                                    
                                    // Quick reactions
                                    { if !m.is_error {
                                        let tx_clone = tx.clone();
                                        let group_ref_clone = group_ref.clone();
                                        let my_name_clone = my_name.clone();
                                        let msg_id = m.id.clone();
                                        html! {
                                            <div style="display: flex; gap: 8px; margin-top: 5px; opacity: 0.6; font-size: 0.9rem;">
                                                { for ["❤️", "👍", "😂", "🎉"].iter().map(|&emoji| {
                                                    let chat_state_inner = chat_state.clone();
                                                    let tx_inner = tx_clone.clone();
                                                    let group_ref_inner = group_ref_clone.clone();
                                                    let my_name_inner = my_name_clone.clone();
                                                    let msg_id_inner = msg_id.clone();
                                                    let emoji_str = emoji.to_string();
                                                    let msg_idx_inner = msg_idx;
                                                    let on_react = Callback::from(move |_: MouseEvent| {
                                                        // Update local state immediately for instant feedback
                                                        chat_state_inner.dispatch(ChatAction::AddReaction {
                                                            msg_index: msg_idx_inner,
                                                            emoji: emoji_str.clone(),
                                                            user: my_name_inner.clone(),
                                                        });
                                                        
                                                        // Send reaction to server for broadcast to other clients
                                                        if let Some(sender) = &*tx_inner {
                                                            if let Some(group_el) = group_ref_inner.cast::<HtmlInputElement>() {
                                                                let group_name = group_el.value().trim().to_string();
                                                                if !group_name.is_empty() {
                                                                    let _ = sender.unbounded_send(FromClient::PostReaction {
                                                                        group_name: Arc::new(group_name),
                                                                        author: Arc::new(my_name_inner.clone()),
                                                                        message_id: msg_id_inner.clone(),
                                                                        emoji: emoji_str.clone(),
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    });
                                                    html! {
                                                        <span 
                                                            onclick={on_react}
                                                            class={css!(r#"
                                                                cursor: pointer;
                                                                transition: transform 0.2s;
                                                                &:hover {
                                                                    transform: scale(1.3);
                                                                }
                                                            "#)}
                                                        >
                                                            { emoji }
                                                        </span>
                                                    }
                                                })}
                                            </div>
                                        }
                                    } else { html! {} }}
                                </div>
                            </div>
                        }
                    })}
                </div>

                <footer class={chat_footer_style}>
                    <input 
                        type="file" 
                        ref={file_input_ref} 
                        style="display: none;" 
                        onchange={on_file_change} 
                    />

                    { if *show_emojis {
                        let emoji_picker_bg = if *dark_mode { "#2d2d2d" } else { "white" };
                        let emoji_border = if *dark_mode { "#3a3a3a" } else { "#e1e4e8" };
                        html! {
                            <div style={format!("position: absolute; bottom: 80px; left: 20px; background: {}; border: 1px solid {}; border-radius: 16px; padding: 15px; box-shadow: 0 8px 25px rgba(0,0,0,0.15); z-index: 100; max-width: 350px; max-height: 400px; overflow-y: auto;", emoji_picker_bg, emoji_border)}>
                                // Smileys & Emotion
                                <div style="margin-bottom: 10px;">
                                    <div style="font-size: 0.7rem; font-weight: 600; margin-bottom: 8px; opacity: 0.6;">{"😀 Smileys"}</div>
                                    <div style="display: grid; grid-template-columns: repeat(8, 1fr); gap: 5px;">
                                        { for ["😀", "😃", "😄", "😁", "😆", "😅", "🤣", "😂", "🙂", "😊", "😇", "🥰", "😍", "🤩", "😘", "😗", "😚", "😙", "🥲", "😋", "😛", "😜", "🤪", "😝", "🤑", "🤗", "🤭", "🤫", "🤔", "🤐", "🤨", "😐", "😑", "😶", "😏", "😒", "🙄", "😬", "🤥", "😌", "😔", "😪", "🤤", "😴", "😷", "🤒", "🤕", "🤢", "🤮", "🥴", "😵", "🤯", "🤠", "🥳", "🥸", "😎", "🤓", "🧐", "😕", "😟", "🙁", "☹️", "😮", "😯", "😲", "😳", "🥺", "😦", "😧", "😨", "😰", "😥", "😢", "😭", "😱", "😖", "😣", "😞", "😓", "😩", "😫", "🥱", "😤", "😡", "😠", "🤬", "😈", "👿", "💀", "☠️", "💩", "🤡", "👹", "👺", "👻", "👽", "👾", "🤖"].iter().map(|&e| {
                                            let on_click = {
                                                let on_select_emoji = on_select_emoji.clone();
                                                Callback::from(move |_: MouseEvent| on_select_emoji.emit(e))
                                            };
                                            html! { <span onclick={on_click} class={css!("cursor: pointer; font-size: 1.4rem; padding: 3px; border-radius: 5px; transition: all 0.15s; &:hover { background: rgba(0,0,0,0.1); transform: scale(1.2); }")}>{ e }</span> }
                                        })}
                                    </div>
                                </div>
                                // Gestures & People
                                <div style="margin-bottom: 10px;">
                                    <div style="font-size: 0.7rem; font-weight: 600; margin-bottom: 8px; opacity: 0.6;">{"👋 Gestures"}</div>
                                    <div style="display: grid; grid-template-columns: repeat(8, 1fr); gap: 5px;">
                                        { for ["👋", "🤚", "🖐️", "✋", "🖖", "👌", "🤌", "🤏", "✌️", "🤞", "🤟", "🤘", "🤙", "👈", "👉", "👆", "🖕", "👇", "☝️", "👍", "👎", "✊", "👊", "🤛", "🤜", "👏", "🙌", "👐", "🤲", "🤝", "🙏", "✍️", "💅", "🤳", "💪", "🦾", "🦿", "🦵", "🦶", "👂", "🦻", "👃", "🧠", "🫀", "🫁", "🦷", "🦴", "👀", "👁️", "👅", "👄"].iter().map(|&e| {
                                            let on_click = {
                                                let on_select_emoji = on_select_emoji.clone();
                                                Callback::from(move |_: MouseEvent| on_select_emoji.emit(e))
                                            };
                                            html! { <span onclick={on_click} class={css!("cursor: pointer; font-size: 1.4rem; padding: 3px; border-radius: 5px; transition: all 0.15s; &:hover { background: rgba(0,0,0,0.1); transform: scale(1.2); }")}>{ e }</span> }
                                        })}
                                    </div>
                                </div>
                                // Hearts & Love
                                <div style="margin-bottom: 10px;">
                                    <div style="font-size: 0.7rem; font-weight: 600; margin-bottom: 8px; opacity: 0.6;">{"❤️ Hearts"}</div>
                                    <div style="display: grid; grid-template-columns: repeat(8, 1fr); gap: 5px;">
                                        { for ["❤️", "🧡", "💛", "💚", "💙", "💜", "🖤", "🤍", "🤎", "💔", "❣️", "💕", "💞", "💓", "💗", "💖", "💘", "💝", "💟", "♥️", "💋", "💯", "💢", "💥", "💫", "💦", "💨", "🕳️", "💣", "💬", "👁️‍🗨️", "🗨️", "🗯️", "💭", "💤"].iter().map(|&e| {
                                            let on_click = {
                                                let on_select_emoji = on_select_emoji.clone();
                                                Callback::from(move |_: MouseEvent| on_select_emoji.emit(e))
                                            };
                                            html! { <span onclick={on_click} class={css!("cursor: pointer; font-size: 1.4rem; padding: 3px; border-radius: 5px; transition: all 0.15s; &:hover { background: rgba(0,0,0,0.1); transform: scale(1.2); }")}>{ e }</span> }
                                        })}
                                    </div>
                                </div>
                                // Animals & Nature
                                <div style="margin-bottom: 10px;">
                                    <div style="font-size: 0.7rem; font-weight: 600; margin-bottom: 8px; opacity: 0.6;">{"🐶 Animals"}</div>
                                    <div style="display: grid; grid-template-columns: repeat(8, 1fr); gap: 5px;">
                                        { for ["🐶", "🐱", "🐭", "🐹", "🐰", "🦊", "🐻", "🐼", "🐨", "🐯", "🦁", "🐮", "🐷", "🐸", "🐵", "🙈", "🙉", "🙊", "🐒", "🐔", "🐧", "🐦", "🐤", "🐣", "🐥", "🦆", "🦅", "🦉", "🦇", "🐺", "🐗", "🐴", "🦄", "🐝", "🐛", "🦋", "🐌", "🐞", "🐜", "🦟", "🦗", "🦂", "🐢", "🐍", "🦎", "🦖", "🦕", "🐙", "🦑", "🦐", "🦞", "🦀", "🐡", "🐠", "🐟", "🐬", "🐳", "🐋", "🦈", "🐊", "🐅", "🐆", "🦓", "🦍", "🦧", "🐘", "🦛", "🦏", "🐪", "🐫", "🦒", "🦘", "🐃", "🐂", "🐄", "🐎", "🐖", "🐏", "🐑", "🦙", "🐐", "🦌", "🐕", "🐩", "🦮", "🐈", "🐓", "🦃", "🦚", "🦜", "🦢", "🦩", "🐇", "🦝", "🦨", "🦡", "🦦", "🦥", "🐁", "🐀", "🐿️", "🦔"].iter().map(|&e| {
                                            let on_click = {
                                                let on_select_emoji = on_select_emoji.clone();
                                                Callback::from(move |_: MouseEvent| on_select_emoji.emit(e))
                                            };
                                            html! { <span onclick={on_click} class={css!("cursor: pointer; font-size: 1.4rem; padding: 3px; border-radius: 5px; transition: all 0.15s; &:hover { background: rgba(0,0,0,0.1); transform: scale(1.2); }")}>{ e }</span> }
                                        })}
                                    </div>
                                </div>
                                // Food & Drink
                                <div style="margin-bottom: 10px;">
                                    <div style="font-size: 0.7rem; font-weight: 600; margin-bottom: 8px; opacity: 0.6;">{"🍕 Food"}</div>
                                    <div style="display: grid; grid-template-columns: repeat(8, 1fr); gap: 5px;">
                                        { for ["🍇", "🍈", "🍉", "🍊", "🍋", "🍌", "🍍", "🥭", "🍎", "🍏", "🍐", "🍑", "🍒", "🍓", "🫐", "🥝", "🍅", "🫒", "🥥", "🥑", "🍆", "🥔", "🥕", "🌽", "🌶️", "🫑", "🥒", "🥬", "🥦", "🧄", "🧅", "🍄", "🥜", "🌰", "🍞", "🥐", "🥖", "🫓", "🥨", "🥯", "🥞", "🧇", "🧀", "🍖", "🍗", "🥩", "🥓", "🍔", "🍟", "🍕", "🌭", "🥪", "🌮", "🌯", "🫔", "🥙", "🧆", "🥚", "🍳", "🥘", "🍲", "🫕", "🥣", "🥗", "🍿", "🧈", "🧂", "🥫", "🍱", "🍘", "🍙", "🍚", "🍛", "🍜", "🍝", "🍠", "🍢", "🍣", "🍤", "🍥", "🥮", "🍡", "🥟", "🥠", "🥡", "🦀", "🦞", "🦐", "🦑", "🦪", "🍦", "🍧", "🍨", "🍩", "🍪", "🎂", "🍰", "🧁", "🥧", "🍫", "🍬", "🍭", "🍮", "🍯", "🍼", "🥛", "☕", "🫖", "🍵", "🍶", "🍾", "🍷", "🍸", "🍹", "🍺", "🍻", "🥂", "🥃", "🥤", "🧋", "🧃", "🧉", "🧊"].iter().map(|&e| {
                                            let on_click = {
                                                let on_select_emoji = on_select_emoji.clone();
                                                Callback::from(move |_: MouseEvent| on_select_emoji.emit(e))
                                            };
                                            html! { <span onclick={on_click} class={css!("cursor: pointer; font-size: 1.4rem; padding: 3px; border-radius: 5px; transition: all 0.15s; &:hover { background: rgba(0,0,0,0.1); transform: scale(1.2); }")}>{ e }</span> }
                                        })}
                                    </div>
                                </div>
                                // Activities & Celebrations
                                <div style="margin-bottom: 10px;">
                                    <div style="font-size: 0.7rem; font-weight: 600; margin-bottom: 8px; opacity: 0.6;">{"🎉 Activities"}</div>
                                    <div style="display: grid; grid-template-columns: repeat(8, 1fr); gap: 5px;">
                                        { for ["⚽", "🏀", "🏈", "⚾", "🥎", "🎾", "🏐", "🏉", "🥏", "🎱", "🪀", "🏓", "🏸", "🏒", "🏑", "🥍", "🏏", "🪃", "🥅", "⛳", "🪁", "🏹", "🎣", "🤿", "🥊", "🥋", "🎽", "🛹", "🛷", "⛸️", "🥌", "🎿", "⛷️", "🏂", "🪂", "🎪", "🎭", "🎨", "🎬", "🎤", "🎧", "🎼", "🎹", "🥁", "🎷", "🎺", "🎸", "🪕", "🎻", "🎲", "♟️", "🎯", "🎳", "🎮", "🎰", "🧩", "🎁", "🎀", "🎊", "🎉", "🎈", "🪅", "🪆", "🧨", "✨", "🎇", "🎆", "🌟", "⭐", "🔥", "💫", "🌈", "☀️", "🌤️", "⛅", "🌥️", "☁️", "🌦️", "🌧️", "⛈️", "🌩️", "🌨️", "❄️", "☃️", "⛄", "🌬️", "💨", "🌪️", "🌫️", "🌊", "💧", "💦", "☔", "🔆", "🔅"].iter().map(|&e| {
                                            let on_click = {
                                                let on_select_emoji = on_select_emoji.clone();
                                                Callback::from(move |_: MouseEvent| on_select_emoji.emit(e))
                                            };
                                            html! { <span onclick={on_click} class={css!("cursor: pointer; font-size: 1.4rem; padding: 3px; border-radius: 5px; transition: all 0.15s; &:hover { background: rgba(0,0,0,0.1); transform: scale(1.2); }")}>{ e }</span> }
                                        })}
                                    </div>
                                </div>
                                // Objects & Symbols
                                <div style="margin-bottom: 5px;">
                                    <div style="font-size: 0.7rem; font-weight: 600; margin-bottom: 8px; opacity: 0.6;">{"💻 Objects"}</div>
                                    <div style="display: grid; grid-template-columns: repeat(8, 1fr); gap: 5px;">
                                        { for ["⌚", "📱", "💻", "⌨️", "🖥️", "🖨️", "🖱️", "🖲️", "💽", "💾", "💿", "📀", "📼", "📷", "📸", "📹", "🎥", "📽️", "🎞️", "📞", "☎️", "📟", "📠", "📺", "📻", "🎙️", "🎚️", "🎛️", "🧭", "⏱️", "⏲️", "⏰", "🕰️", "⌛", "⏳", "📡", "🔋", "🔌", "💡", "🔦", "🕯️", "🪔", "🧯", "🛢️", "💸", "💵", "💴", "💶", "💷", "🪙", "💰", "💳", "💎", "⚖️", "🪜", "🧰", "🔧", "🔨", "⚒️", "🛠️", "⛏️", "🪓", "🔩", "⚙️", "🪤", "🧱", "⛓️", "🧲", "🔫", "💣", "🧨", "🪚", "🔪", "🗡️", "⚔️", "🛡️", "🚬", "⚰️", "🪦", "⚱️", "🏺", "🔮", "📿", "🧿", "💈", "⚗️", "🔭", "🔬", "🕳️", "🩹", "🩺", "💊", "💉", "🩸", "🧬", "🦠", "🧫", "🧪", "🌡️", "🧹", "🪠", "🧺", "🧻", "🚽", "🚰", "🚿", "🛁", "🛀", "🧼", "🪥", "🪒", "🧽", "🪣", "🧴", "🛎️", "🔑", "🗝️", "🚪", "🪑", "🛋️", "🛏️", "🛌", "🧸", "🖼️", "🪞", "🪟", "🛒", "🎁", "🎈", "🎏", "🎀", "🪄", "🧧", "🎐", "🎑", "🧧", "✉️", "📩", "📨", "📧", "💌", "📥", "📤", "📦", "🏷️", "📪", "📫", "📬", "📭", "📮", "📯", "📜", "📃", "📄", "📑", "🧾", "📊", "📈", "📉", "🗒️", "🗓️", "📆", "📅", "🗑️", "📇", "🗃️", "🗳️", "🗄️", "📋", "📁", "📂", "🗂️", "🗞️", "📰", "📓", "📔", "📒", "📕", "📗", "📘", "📙", "📚", "📖", "🔖", "🧷", "🔗", "📎", "🖇️", "📐", "📏", "🧮", "📌", "📍", "✂️", "🖊️", "🖋️", "✒️", "🖌️", "🖍️", "📝", "✏️", "🔍", "🔎", "🔏", "🔐", "🔒", "🔓", "✅", "❎", "✔️", "❌", "❓", "❔", "❕", "❗", "〰️", "➕", "➖", "➗", "✖️", "💲", "💱", "™️", "©️", "®️", "🔚", "🔙", "🔛", "🔝", "🔜", "☑️", "🔘", "🔴", "🟠", "🟡", "🟢", "🔵", "🟣", "⚫", "⚪", "🟤", "🔺", "🔻", "🔸", "🔹", "🔶", "🔷", "🔳", "🔲", "▪️", "▫️", "◾", "◽", "◼️", "◻️", "🟥", "🟧", "🟨", "🟩", "🟦", "🟪", "⬛", "⬜", "🟫"].iter().map(|&e| {
                                            let on_click = {
                                                let on_select_emoji = on_select_emoji.clone();
                                                Callback::from(move |_: MouseEvent| on_select_emoji.emit(e))
                                            };
                                            html! { <span onclick={on_click} class={css!("cursor: pointer; font-size: 1.4rem; padding: 3px; border-radius: 5px; transition: all 0.15s; &:hover { background: rgba(0,0,0,0.1); transform: scale(1.2); }")}>{ e }</span> }
                                        })}
                                    </div>
                                </div>
                            </div>
                        }
                    } else { html! {} }}

                    <div class={input_wrapper_style}>
                        <div style="display: flex; gap: 12px; padding: 0 10px; border-right: 1px solid #f0f0f0;">
                            <span onclick={on_file_click.clone()} class={icon_btn_style.clone()} title="Attach File">{"📎"}</span>
                            <span onclick={on_emoji_click} class={icon_btn_style.clone()} title="Insert Emoji">{"😊"}</span>
                        </div>
                        <input 
                            ref={input_ref.clone()}
                            class={css!("flex: 1; border: none; padding: 10px 15px; outline: none; font-size: 0.95rem;")} 
                            placeholder={if *is_recording { "Recording voice..." } else { "Type progress here..." }}
                            onkeypress={on_keypress.clone()}
                            oninput={on_input_change}
                            disabled={*is_recording}
                        />
                        <div style="display: flex; gap: 12px; padding: 0 10px; border-left: 1px solid #f0f0f0;">
                            <span onclick={on_file_click.clone()} class={icon_btn_style.clone()}>{"📷"}</span>
                            <span onclick={toggle_recording} class={classes!(icon_btn_style.clone(), if *is_recording { css!(r#"
                                color: #e74c3c; 
                                animation: pulse 1.5s infinite;
                                background: rgba(231, 76, 60, 0.1);
                                border-radius: 50%;
                                padding: 8px;
                                @keyframes pulse {
                                    0%, 100% { transform: scale(1); opacity: 1; }
                                    50% { transform: scale(1.1); opacity: 0.8; }
                                }
                            "#) } else { css!(r#"
                                &:hover { color: #e74c3c; }
                            "#) })}>
                                { if *is_recording { "⏹️" } else { "🎙️" } }
                            </span>
                            { if *is_recording {
                                html! {
                                    <span style="font-size: 0.75rem; color: #e74c3c; margin-left: 5px; font-weight: 600;">
                                        {"Recording... Click to stop & send"}
                                    </span>
                                }
                            } else { html! {} }}
                        </div>
                    </div>
                    <button onclick={on_send} class={send_circle_btn}>
                        {"➔"}
                    </button>
                    
                    // Typing indicator
                    { if *is_typing && chat_state.typing_users.len() > 0 {
                        let typing_bg = if *dark_mode { "#3a3a3a" } else { "#f1f3f4" };
                        let typing_text = if *dark_mode { "#a0aec0" } else { "#6b7280" };
                        html! {
                            <div style={format!(
                                "position: absolute; bottom: 100%; left: 30px; background: {}; padding: 8px 15px; border-radius: 20px; font-size: 0.8rem; color: {}; margin-bottom: 5px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                                typing_bg, typing_text
                            )}>
                                <span style="opacity: 0.7;">{"Someone is typing"}</span>
                                <span class={css!(r#"
                                    margin-left: 5px;
                                    &::after {
                                        content: '...';
                                        animation: typing 1.4s infinite;
                                    }
                                    @keyframes typing {
                                        0%, 100% { opacity: 0; }
                                        50% { opacity: 1; }
                                    }
                                "#)}>
                                </span>
                            </div>
                        }
                    } else { html! {} }}
                </footer>
            </main>

            <aside class={if *right_sidebar_visible { sidebar_right_style.clone() } else { css!("display: none;") }}>
                <div class={title_row_style.clone()} style="padding: 0 10px;">
                    <span onclick={toggle_right} style="cursor: pointer; opacity: 0.5; font-size: 1.2rem;" title="Collapse Panel">{"⇢"}</span>
                    <div style="font-weight: 700; font-size: 0.8rem; color: #a0aec0;">{"ROOM INFO"}</div>
                    <div style="width: 20px;"></div>
                </div>

                <div class={profile_large_style}>
                    <img src={format!("https://ui-avatars.com/api/?name={}&background=3498db&color=fff", chat_state.current_group.as_deref().unwrap_or("User"))} class={avatar_style.clone()} style="width: 100px; height: 100px; margin-bottom: 20px;" />
                    <h2 style="margin: 0; font-size: 1.25rem;">{ chat_state.current_group.as_deref().unwrap_or("Select a Group") }</h2>
                    <div style="opacity: 0.6; font-size: 0.85rem; margin-top: 5px;">{"Active Member"}</div>
                </div>

                <div class={action_grid_style}>
                    <div onclick={on_chat_click.clone()} class={action_card_style.clone()} style="cursor: pointer;">
                        <span class="icon">{"💬"}</span>
                        <span class="label">{"Chat"}</span>
                    </div>
                    <div onclick={on_file_click.clone()} class={action_card_style.clone()}>
                        <span class="icon" style="color: #4a5568;">{"📁"}</span>
                        <span class="label">{"Send File"}</span>
                    </div>
                </div>

                // Online Users Section
                <div style="margin-top: 20px; padding-top: 15px; border-top: 1px solid rgba(0,0,0,0.1);">
                    <div style="font-weight: 700; font-size: 0.75rem; color: #a0aec0; margin-bottom: 10px;">
                        {"👥 ONLINE USERS ("}{chat_state.current_online_users().len()}{")"}
                    </div>
                    { if chat_state.current_online_users().is_empty() {
                        html! {
                            <div style="font-size: 0.8rem; opacity: 0.6; padding: 10px 0;">
                                {"No users online yet"}
                            </div>
                        }
                    } else {
                        html! {
                            <div style="display: flex; flex-direction: column; gap: 8px; max-height: 150px; overflow-y: auto;">
                                { for chat_state.current_online_users().iter().map(|user| {
                                    let status_color = match user.status.as_str() {
                                        "Online" => "#2ecc71",
                                        "Away" => "#f39c12",
                                        _ => "#95a5a6",
                                    };
                                    html! {
                                        <div style="display: flex; align-items: center; gap: 10px; padding: 5px;">
                                            <div style="position: relative;">
                                                <img src={format!("https://ui-avatars.com/api/?name={}&background=random&size=32", user.username)} 
                                                     style="width: 32px; height: 32px; border-radius: 50%;" />
                                                <div style={format!("position: absolute; bottom: 0; right: 0; width: 10px; height: 10px; border-radius: 50%; background: {}; border: 2px solid white;", status_color)}></div>
                                            </div>
                                            <div>
                                                <div style="font-size: 0.85rem; font-weight: 500;">{ &user.username }</div>
                                                <div style="font-size: 0.7rem; opacity: 0.6;">{ &user.status }</div>
                                            </div>
                                        </div>
                                    }
                                })}
                            </div>
                        }
                    }}
                </div>

                <div style="display: flex; flex-direction: column; gap: 15px; margin-top: 10px;">
                    <div onclick={on_view_friends_click.clone()} style="display: flex; align-items: center; gap: 10px; font-size: 0.9rem; cursor: pointer; padding: 8px; border-radius: 8px; transition: background 0.2s;" class={css!("&:hover { background: rgba(0,0,0,0.05); }")}>
                        <span>{"👥"}</span> {"View Friends / Groups"}
                    </div>
                    <div onclick={on_add_favorite_click.clone()} style="display: flex; align-items: center; gap: 10px; font-size: 0.9rem; cursor: pointer; padding: 8px; border-radius: 8px; transition: background 0.2s;" class={css!("&:hover { background: rgba(0,0,0,0.05); }")}>
                        <span>{"⭐"}</span> {"Add to Favorites"}
                    </div>
                </div>

                <div class={attachments_section}>
                    <div class={title_row_style}>
                        <div style="font-weight: 700; font-size: 0.8rem; color: #a0aec0;">{"ATTACHMENTS"}</div>
                        <div style="font-size: 0.7rem; color: #3182ce; cursor: pointer;">{"View All"}</div>
                    </div>
                    <div class={attachment_grid}>
                        { for chat_state.current_messages().iter().filter_map(|m| {
                            if let MessageContent::File { filename, .. } = &m.content {
                                let (icon, class) = if filename.to_lowercase().ends_with(".pdf") { ("📄", "pdf") }
                                               else if filename.to_lowercase().ends_with(".mp3") || filename.to_lowercase().ends_with(".wav") { ("♫", "audio") }
                                               else if filename.to_lowercase().ends_with(".png") || filename.to_lowercase().ends_with(".jpg") || filename.to_lowercase().ends_with(".jpeg") { ("🖼", "image") }
                                               else { ("📁", "") };
                                Some(html! {
                                    <div class={classes!(attachment_item.clone(), class)}>
                                        <span class="icon">{ icon }</span>
                                    </div>
                                })
                            } else { None }
                        }).take(8) }
                    </div>
                </div>

                <style>
                    {r#"
                        @keyframes pulse {
                            0% { transform: scale(1); }
                            50% { transform: scale(1.2); }
                            100% { transform: scale(1); }
                        }
                    "#}
                </style>
            </aside>
        </div>
    }
}
