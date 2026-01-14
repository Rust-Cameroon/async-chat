use crate::types::{ConnectionStatus, Theme};
use stylist::yew::styled_component;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub current_user: String,
    pub connection_status: ConnectionStatus,
    pub theme: Theme,
    pub on_theme_change: Callback<Theme>,
}

#[styled_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    let theme = props.theme;

    let header_style = stylist::Style::new(
        r#"
        .header {
            background-color: var(--color-bg-secondary);
            border-bottom: 1px solid var(--color-border-primary);
            padding: var(--spacing-sm) var(--spacing-md);
            display: flex;
            align-items: center;
            justify-content: space-between;
            height: 48px;
            min-height: 48px;
            flex-shrink: 0;
        }
        
        .header-left {
            display: flex;
            align-items: center;
            gap: var(--spacing-md);
        }
        
        .header-title {
            font-size: var(--font-size-lg);
            font-weight: var(--font-weight-semibold);
            color: var(--color-text-primary);
        }
        
        .header-right {
            display: flex;
            align-items: center;
            gap: var(--spacing-md);
        }
        
        .user-info {
            display: flex;
            align-items: center;
            gap: var(--spacing-sm);
            color: var(--color-text-secondary);
            font-size: var(--font-size-sm);
        }
        
        .user-avatar {
            width: 24px;
            height: 24px;
            border-radius: 50%;
            background-color: var(--color-primary);
            color: white;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: var(--font-size-xs);
            font-weight: var(--font-weight-semibold);
        }
        
        .connection-status {
            display: flex;
            align-items: center;
            gap: var(--spacing-xs);
            padding: var(--spacing-xs) var(--spacing-sm);
            border-radius: var(--border-radius-sm);
            font-size: var(--font-size-xs);
            font-weight: var(--font-weight-medium);
        }
        
        .status-connected {
            background-color: rgba(59, 165, 92, 0.1);
            color: var(--color-success);
        }
        
        .status-connecting {
            background-color: rgba(250, 166, 26, 0.1);
            color: var(--color-warning);
        }
        
        .status-disconnected {
            background-color: rgba(237, 66, 69, 0.1);
            color: var(--color-danger);
        }
        
        .status-error {
            background-color: rgba(237, 66, 69, 0.1);
            color: var(--color-danger);
        }
        
        .status-indicator {
            width: 8px;
            height: 8px;
            border-radius: 50%;
        }
        
        .indicator-connected {
            background-color: var(--color-success);
        }
        
        .indicator-connecting {
            background-color: var(--color-warning);
            animation: pulse 1.5s infinite;
        }
        
        .indicator-disconnected {
            background-color: var(--color-danger);
        }
        
        .indicator-error {
            background-color: var(--color-danger);
        }
        
        .theme-toggle {
            background: none;
            border: none;
            color: var(--color-text-secondary);
            cursor: pointer;
            padding: var(--spacing-xs);
            border-radius: var(--border-radius-sm);
            transition: var(--transition-fast);
        }
        
        .theme-toggle:hover {
            background-color: var(--color-bg-elevated);
            color: var(--color-text-primary);
        }
        
        .theme-icon {
            font-size: var(--font-size-lg);
        }
    "#,
    )
    .expect("Failed to create header styles");

    let toggle_theme = {
        let on_theme_change = props.on_theme_change.clone();
        Callback::from(move |_| {
            on_theme_change.emit(theme.toggle());
        })
    };

    let connection_class = match props.connection_status {
        ConnectionStatus::Connected => "status-connected",
        ConnectionStatus::Connecting => "status-connecting",
        ConnectionStatus::Disconnected => "status-disconnected",
        ConnectionStatus::Error(_) => "status-error",
    };

    let indicator_class = match props.connection_status {
        ConnectionStatus::Connected => "indicator-connected",
        ConnectionStatus::Connecting => "indicator-connecting",
        ConnectionStatus::Disconnected => "indicator-disconnected",
        ConnectionStatus::Error(_) => "indicator-error",
    };

    let status_text = props.connection_status.to_string();
    let user_initial = props
        .current_user
        .chars()
        .next()
        .unwrap_or('U')
        .to_uppercase();
    let theme_icon = match theme {
        Theme::Dark => "🌙",
        Theme::Light => "☀️",
    };

    html! {
        <header class={header_style}>
            <div class="header">
                <div class="header-left">
                    <div class="header-title">{"🔥 Async Chat"}</div>
                </div>

                <div class="header-right">
                    <div class="user-info">
                        <div class="user-avatar">{user_initial}</div>
                        <span>{&props.current_user}</span>
                    </div>

                    <div class={format!("connection-status {}", connection_class)}>
                        <div class={format!("status-indicator {}", indicator_class)}></div>
                        <span>{status_text}</span>
                    </div>

                    <button class="theme-toggle" onclick={toggle_theme} title={format!("Switch to {} theme", match theme { Theme::Dark => "light", Theme::Light => "dark" })}>
                        <span class="theme-icon">{theme_icon}</span>
                    </button>
                </div>
            </div>
        </header>
    }
}
