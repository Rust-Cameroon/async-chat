use crate::types::{ChatMessage, Theme};
use stylist::yew::styled_component;
use yew::prelude::*;

use super::message_input::MessageInput;
use super::message_list::MessageList;

#[derive(Properties, PartialEq)]
pub struct ChatRoomProps {
    pub messages: Vec<ChatMessage>,
    pub current_group: Option<String>,
    pub current_user: String,
    pub on_send_message: Callback<(String, String)>,
    pub theme: Theme,
}

#[styled_component(ChatRoom)]
pub fn chat_room(props: &ChatRoomProps) -> Html {
    let chat_room_style = stylist::Style::new(
        r#"
        .chat-room {
            flex: 1;
            display: flex;
            flex-direction: column;
            background-color: var(--color-bg-primary);
            height: 100%;
            overflow: hidden;
        }
        
        .chat-room-header {
            background-color: var(--color-bg-secondary);
            border-bottom: 1px solid var(--color-border-primary);
            padding: var(--spacing-md) var(--spacing-lg);
            display: flex;
            align-items: center;
            justify-content: space-between;
            min-height: 56px;
            flex-shrink: 0;
        }
        
        .chat-room-title {
            display: flex;
            flex-direction: column;
            gap: 2px;
        }
        
        .chat-room-name {
            font-size: var(--font-size-md);
            font-weight: var(--font-weight-semibold);
            color: var(--color-text-primary);
        }
        
        .chat-room-status {
            font-size: var(--font-size-xs);
            color: var(--color-text-muted);
        }
        
        .chat-room-actions {
            display: flex;
            align-items: center;
            gap: var(--spacing-sm);
        }
        
        .action-button {
            background: none;
            border: none;
            color: var(--color-text-secondary);
            cursor: pointer;
            padding: var(--spacing-xs);
            border-radius: var(--border-radius-sm);
            transition: var(--transition-fast);
            width: 32px;
            height: 32px;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        
        .action-button:hover {
            background-color: var(--color-bg-elevated);
            color: var(--color-text-primary);
        }
        
        .chat-room-content {
            flex: 1;
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }
        
        .messages-container {
            flex: 1;
            overflow: hidden;
        }
        
        .no-group-selected {
            flex: 1;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            color: var(--color-text-muted);
            text-align: center;
            padding: var(--spacing-xl);
        }
        
        .no-group-icon {
            font-size: 64px;
            margin-bottom: var(--spacing-lg);
            opacity: 0.3;
        }
        
        .no-group-title {
            font-size: var(--font-size-lg);
            font-weight: var(--font-weight-medium);
            margin-bottom: var(--spacing-sm);
            color: var(--color-text-secondary);
        }
        
        .no-group-text {
            font-size: var(--font-size-sm);
            line-height: 1.5;
            max-width: 400px;
        }
    "#,
    )
    .expect("Failed to create chat room styles");

    let current_group_display = props.current_group.clone();
    let member_count = 1; // This would come from props or state in a real app

    html! {
        <div class={chat_room_style}>
            <div class="chat-room">
                if let Some(group_name) = &props.current_group {
                    <div class="chat-room-header">
                        <div class="chat-room-title">
                            <div class="chat-room-name">{format!("# {}", group_name)}</div>
                            <div class="chat-room-status">{format!("{} member{}", member_count, if member_count == 1 { "" } else { "s" })}</div>
                        </div>
                        <div class="chat-room-actions">
                            <button class="action-button" title="Search messages">
                                <span>{"🔍"}</span>
                            </button>
                            <button class="action-button" title="Group settings">
                                <span>{"⚙️"}</span>
                            </button>
                            <button class="action-button" title="Group info">
                                <span>{"ℹ️"}</span>
                            </button>
                        </div>
                    </div>

                    <div class="chat-room-content">
                        <div class="messages-container">
                            <MessageList
                                messages={props.messages.clone()}
                                theme={props.theme}
                            />
                        </div>

                        <MessageInput
                            current_group={props.current_group.clone()}
                            current_user={props.current_user.clone()}
                            on_send_message={props.on_send_message.clone()}
                            disabled={false}
                        />
                    </div>
                } else {
                    <div class="no-group-selected">
                        <div class="no-group-icon">{"📱"}</div>
                        <div class="no-group-title">{"Welcome to Async Chat!"}</div>
                        <div class="no-group-text">
                            {"Select a group from the sidebar to start chatting, or create a new group to begin your conversation."}
                        </div>
                    </div>
                }
            </div>
        </div>
    }
}
