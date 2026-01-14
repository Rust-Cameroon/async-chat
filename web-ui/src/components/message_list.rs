use crate::types::{ChatMessage, Theme};
use chrono::{DateTime, Utc};
use stylist::yew::styled_component;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MessageListProps {
    pub messages: Vec<ChatMessage>,
    pub theme: Theme,
}

#[styled_component(MessageList)]
pub fn message_list(props: &MessageListProps) -> Html {
    let messages = props.messages.clone();
    let theme = props.theme;

    let message_list_style = stylist::Style::new(
        r#"
        .message-list {
            flex: 1;
            overflow-y: auto;
            padding: var(--spacing-md);
            display: flex;
            flex-direction: column;
            gap: var(--spacing-md);
        }
        
        .message-container {
            display: flex;
            flex-direction: column;
            gap: var(--spacing-xs);
        }
        
        .message-own {
            align-items: flex-end;
        }
        
        .message-other {
            align-items: flex-start;
        }
        
        .message-bubble {
            max-width: 70%;
            padding: var(--spacing-sm) var(--spacing-md);
            border-radius: var(--border-radius-lg);
            word-wrap: break-word;
            position: relative;
        }
        
        .message-own .message-bubble {
            background-color: var(--color-primary);
            color: white;
            border-bottom-right-radius: var(--border-radius-sm);
        }
        
        .message-other .message-bubble {
            background-color: var(--color-bg-elevated);
            color: var(--color-text-primary);
            border-bottom-left-radius: var(--border-radius-sm);
            border: 1px solid var(--color-border-secondary);
        }
        
        .message-header {
            display: flex;
            align-items: baseline;
            gap: var(--spacing-xs);
            margin-bottom: 2px;
            font-size: var(--font-size-xs);
            color: var(--color-text-muted);
        }
        
        .message-own .message-header {
            justify-content: flex-end;
        }
        
        .message-other .message-header {
            justify-content: flex-start;
        }
        
        .message-sender {
            font-weight: var(--font-weight-semibold);
            color: var(--color-text-secondary);
        }
        
        .message-own .message-sender {
            color: rgba(255, 255, 255, 0.8);
        }
        
        .message-timestamp {
            opacity: 0.7;
        }
        
        .message-content {
            font-size: var(--font-size-md);
            line-height: 1.4;
            white-space: pre-wrap;
        }
        
        .message-own .message-content {
            color: white;
        }
        
        .message-divider {
            display: flex;
            align-items: center;
            margin: var(--spacing-lg) 0;
            color: var(--color-text-muted);
        }
        
        .message-divider-line {
            flex: 1;
            height: 1px;
            background-color: var(--color-border-secondary);
        }
        
        .message-divider-text {
            padding: 0 var(--spacing-md);
            font-size: var(--font-size-xs);
            font-weight: var(--font-weight-medium);
        }
        
        .typing-indicator {
            display: flex;
            align-items: center;
            gap: var(--spacing-xs);
            padding: var(--spacing-sm) var(--spacing-md);
            background-color: var(--color-bg-elevated);
            border-radius: var(--border-radius-lg);
            border-bottom-left-radius: var(--border-radius-sm);
            border: 1px solid var(--color-border-secondary);
            max-width: 70%;
        }
        
        .typing-dots {
            display: flex;
            gap: 4px;
        }
        
        .typing-dot {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            background-color: var(--color-text-muted);
            animation: typingBounce 1.4s infinite ease-in-out both;
        }
        
        .typing-dot:nth-child(1) { animation-delay: -0.32s; }
        .typing-dot:nth-child(2) { animation-delay: -0.16s; }
        .typing-dot:nth-child(3) { animation-delay: 0s; }
        
        @keyframes typingBounce {
            0%, 80%, 100% {
                transform: scale(0);
            }
            40% {
                transform: scale(1);
            }
        }
        
        .empty-state {
            flex: 1;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            color: var(--color-text-muted);
            text-align: center;
            padding: var(--spacing-xl);
        }
        
        .empty-icon {
            font-size: 64px;
            margin-bottom: var(--spacing-md);
            opacity: 0.3;
        }
        
        .empty-title {
            font-size: var(--font-size-lg);
            font-weight: var(--font-weight-medium);
            margin-bottom: var(--spacing-sm);
            color: var(--color-text-secondary);
        }
        
        .empty-text {
            font-size: var(--font-size-sm);
            line-height: 1.5;
        }
        
        .date-divider {
            text-align: center;
            margin: var(--spacing-md) 0;
            position: relative;
        }
        
        .date-divider span {
            background-color: var(--color-bg-secondary);
            color: var(--color-text-muted);
            padding: var(--spacing-xs) var(--spacing-sm);
            border-radius: var(--border-radius-sm);
            font-size: var(--font-size-xs);
            font-weight: var(--font-weight-medium);
            border: 1px solid var(--color-border-secondary);
        }
    "#,
    )
    .expect("Failed to create message list styles");

    let format_timestamp = |timestamp: DateTime<Utc>| {
        let now = Utc::now();
        let diff = now.signed_duration_since(timestamp);

        if diff.num_minutes() < 1 {
            "now".to_string()
        } else if diff.num_hours() < 1 {
            format!("{}m ago", diff.num_minutes())
        } else if diff.num_days() < 1 {
            format!("{}h ago", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("{}d ago", diff.num_days())
        } else {
            timestamp.format("%b %d").to_string()
        }
    };

    // Group messages by date for date dividers
    let mut grouped_messages: Vec<(String, Vec<&ChatMessage>)> = Vec::new();
    let mut current_date = String::new();
    let mut current_group: Vec<&ChatMessage> = Vec::new();

    for message in &messages {
        let message_date = message.timestamp.format("%Y-%m-%d").to_string();

        if current_date.is_empty() {
            current_date = message_date;
            current_group.push(message);
        } else if current_date == message_date {
            current_group.push(message);
        } else {
            grouped_messages.push((current_date, current_group));
            current_date = message_date;
            current_group = vec![message];
        }
    }

    if !current_group.is_empty() {
        grouped_messages.push((current_date, current_group));
    }

    html! {
        <div class={message_list_style}>
            <div class="message-list">
                if messages.is_empty() {
                    <div class="empty-state">
                        <div class="empty-icon">{"💬"}</div>
                        <div class="empty-title">{"No messages yet"}</div>
                        <div class="empty-text">
                            {"Start a conversation by sending a message to a group!"}
                        </div>
                    </div>
                } else {
                    {for grouped_messages.into_iter().map(|(date, group_messages)| {
                        let formatted_date = if date == Utc::now().format("%Y-%m-%d").to_string() {
                            "Today".to_string()
                        } else if date == (Utc::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string() {
                            "Yesterday".to_string()
                        } else {
                            let parsed_date = chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", date))
                                .unwrap_or_else(|_| chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap());
                            parsed_date.format("%B %d, %Y").to_string()
                        };

                        html! {
                            <>
                                <div class="date-divider">
                                    <span>{formatted_date}</span>
                                </div>
                                {for group_messages.into_iter().map(|message| {
                                    let is_own = message.is_own;
                                    let message_class = classes!(
                                        "message-container",
                                        if is_own { "message-own" } else { "message-other" }
                                    );

                                    html! {
                                        <div key={message.id.clone()} class={message_class}>
                                            <div class="message-bubble">
                                                {if !is_own {
                                                    html! {
                                                        <div class="message-header">
                                                            <span class="message-sender">{&message.sender}</span>
                                                            <span class="message-timestamp">{format_timestamp(message.timestamp)}</span>
                                                        </div>
                                                    }
                                                } else {
                                                    html! {}
                                                }}
                                                <div class="message-content">{&message.content}</div>
                                                {if is_own {
                                                    html! {
                                                        <div class="message-header">
                                                            <span class="message-timestamp">{format_timestamp(message.timestamp)}</span>
                                                        </div>
                                                    }
                                                } else {
                                                    html! {}
                                                }}
                                            </div>
                                        </div>
                                    }
                                }).collect::<Html>()}
                            </>
                        }
                    }).collect::<Html>()}
                }
            </div>
        </div>
    }
}
