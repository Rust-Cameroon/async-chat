use crate::services::use_connection_service;
use crate::types::*;
use async_chat::{FromClient, FromServer};
use stylist::{Style, yew::styled_component};
use web_sys::console;
use yew::prelude::*;
use yew::{Component, Context, Html, html};

// Import child components
use super::chat_room::chat_room;
use super::group_sidebar::group_sidebar;
use super::header::header;

/// Main application styles
const APP_STYLE: &str = include_str!("../styles/global.css");

#[styled_component(App)]
pub fn app_component() -> Html {
    let state = use_state(|| AppState::default());
    let connection_service = use_connection_service(
        Callback::from({
            let state = state.clone();
            move |message: FromServer| {
                handle_server_message(message, &state);
            }
        }),
        Callback::from({
            let state = state.clone();
            move |status: ConnectionStatus| {
                handle_connection_status_change(status, &state);
            }
        }),
    );

    // Auto-connect on component mount
    {
        let mut service = (*connection_service).clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Try WebSocket first, fall back to HTTP simulation
                if let Err(e) = service.connect("ws://127.0.0.1:8081").await {
                    console::log_1(
                        &format!("WebSocket connection failed: {}. Using HTTP simulation.", e)
                            .into(),
                    );
                    service.set_http_mode();
                    if let Err(e) = service.connect("http://127.0.0.1:8080").await {
                        console::log_1(&format!("HTTP simulation failed: {}", e).into());
                    }
                }
            });
            || ()
        });
    }

    let current_state = (*state).clone();
    let service = (*connection_service).clone();

    html! {
                <div class="app-container">
                    <style>{STYLES}</style>

                    <header
                current_user={current_state.current_user.clone()}
                connection_status={current_state.connection_status.clone()}
                theme={current_state.theme}
                on_theme_change={Callback::from(move |theme| {
                    state.set(AppState {
                        theme,
                        ..current_state.clone()
                    });
                })}
            />

            <main class="chat-main">
                        <group_sidebar
                    groups={current_state.groups.clone()}
                    current_group={current_state.current_group.clone()}
                    on_group_select={Callback::from(move |group_name: Option<String>| {
                        state.set(AppState {
                            current_group: group_name,
                            ..current_state.clone()
                        });
                    })}
                    on_join_group={Callback::from(move |group_name: String| {
                        let mut s = service.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let message = FromClient::Join {
                                group_name: std::sync::Arc::new(group_name),
                            };
                            if let Err(e) = s.send(message).await {
                                console::log_1(&format!("Failed to join group: {}", e).into());
                            }
                        });
                    })}
                />

                        <chat_room
                    messages={current_state.messages.clone()}
                    current_group={current_state.current_group.clone()}
                    current_user={current_state.current_user.clone()}
                    on_send_message={Callback::from(move |(group_name, message): (String, String)| {
                        let mut s = service.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let client_message = FromClient::Post {
                                group_name: std::sync::Arc::new(group_name),
                                message: std::sync::Arc::new(message),
                            };
                            if let Err(e) = s.send(client_message).await {
                                console::log_1(&format!("Failed to send message: {}", e).into());
                            }
                        });
                    })}
                    theme={current_state.theme}
                />
            </main>
        </div>
    }
}

/// Handle incoming messages from server
fn handle_server_message(message: FromServer, state: &UseStateHandle<AppState>) {
    let mut current_state = (*state).clone();

    match message {
        FromServer::Message {
            group_name,
            message,
        } => {
            let chat_message = ChatMessage::from_server_message(
                (*group_name).clone(),
                (*message).clone(),
                "Server".to_string(), // Server doesn't send sender info yet
                current_state.current_user.clone(),
            );

            current_state.messages.push(chat_message);

            // Update group's last message
            if let Some(group) = current_state
                .groups
                .iter_mut()
                .find(|g| g.name == *group_name)
            {
                group.update_activity(&(*message));
            }
        }
        FromServer::Error(error) => {
            console::log_1(&format!("Server error: {}", error).into());
        }
    }

    state.set(current_state);
}

/// Handle connection status changes
fn handle_connection_status_change(status: ConnectionStatus, state: &UseStateHandle<AppState>) {
    let mut current_state = (*state).clone();
    current_state.connection_status = status;
    state.set(current_state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_initialization() {
        // Basic test to ensure component can be created
        let _app = App();
    }
}
