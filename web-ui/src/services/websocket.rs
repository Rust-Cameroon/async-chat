use crate::types::*;
use async_chat::{FromClient, FromServer};
use gloo_net::websocket::{Event, Message, WebSocket, WebSocketError};
use std::rc::Rc;
use yew::prelude::*;
use yew::{Callback, NodeRef};

/// WebSocket service for real-time communication with server
#[derive(Clone)]
pub struct WebSocketService {
    websocket: Option<WebSocket>,
    on_message_callback: Callback<FromServer>,
    on_status_change: Callback<ConnectionStatus>,
}

impl WebSocketService {
    /// Create a new WebSocket service
    pub fn new(
        on_message_callback: Callback<FromServer>,
        on_status_change: Callback<ConnectionStatus>,
    ) -> Self {
        Self {
            websocket: None,
            on_message_callback,
            on_status_change,
        }
    }

    /// Connect to WebSocket server
    pub fn connect(&mut self, url: &str) -> Result<(), WebSocketError> {
        self.on_status_change.emit(ConnectionStatus::Connecting);

        let ws = WebSocket::new(url)?;
        let ws_clone = ws.clone();
        let message_callback = self.on_message_callback.clone();
        let status_callback = self.on_status_change.clone();

        // Set up message handler
        ws.set_binary_message_handler(move |_data| {
            log::debug!("Received binary message (not supported)");
        });

        ws.set_text_message_handler(move |text| {
            log::debug!("Received text message: {}", text);

            // Parse the message
            match serde_json::from_str::<FromServer>(&text) {
                Ok(server_message) => {
                    message_callback.emit(server_message);
                }
                Err(e) => {
                    log::error!("Failed to parse server message: {}", e);
                    let error_msg = FromServer::Error(format!("Invalid message format: {}", e));
                    message_callback.emit(error_msg);
                }
            }
        });

        // Set up event handlers for connection lifecycle
        let status_change_clone = status_callback.clone();
        ws.set_onopen(move || {
            log::info!("WebSocket connected");
            status_change_clone.emit(ConnectionStatus::Connected);
        });

        let status_change_clone = status_callback.clone();
        ws.set_onclose(move || {
            log::info!("WebSocket disconnected");
            status_change_clone.emit(ConnectionStatus::Disconnected);
        });

        let status_change_clone = status_callback.clone();
        ws.set_onerror(move || {
            log::error!("WebSocket error occurred");
            status_change_clone.emit(ConnectionStatus::Error("Connection error".to_string()));
        });

        self.websocket = Some(ws);
        Ok(())
    }

    /// Send a message to the server
    pub fn send(&self, message: FromClient) -> Result<(), WebSocketError> {
        if let Some(ws) = &self.websocket {
            let json = serde_json::to_string(&message)
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
            ws.send(Message::Text(json))
        } else {
            Err(WebSocketError::ConnectionError("Not connected".to_string()))
        }
    }

    /// Disconnect from server
    pub fn disconnect(&mut self) {
        if let Some(ws) = self.websocket.take() {
            ws.close();
            self.on_status_change.emit(ConnectionStatus::Disconnected);
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.websocket.is_some()
    }
}

/// HTTP Service for fallback when WebSocket is not available
pub struct HttpService {
    on_message_callback: Callback<FromServer>,
    on_status_change: Callback<ConnectionStatus>,
}

impl HttpService {
    pub fn new(
        on_message_callback: Callback<FromServer>,
        on_status_change: Callback<ConnectionStatus>,
    ) -> Self {
        Self {
            on_message_callback,
            on_status_change,
        }
    }

    /// Send a message via HTTP (simulated)
    pub async fn send(&self, message: FromClient) -> Result<(), Box<dyn std::error::Error>> {
        // For now, this is a placeholder that simulates responses
        // In a real implementation, you'd make HTTP requests to a REST API

        match message {
            FromClient::Join { group_name } => {
                // Simulate successful join
                let response = FromServer::Message {
                    group_name,
                    message: std::sync::Arc::new("You joined the group".to_string()),
                };
                self.on_message_callback.emit(response);
            }
            FromClient::Post {
                group_name,
                message,
            } => {
                // Simulate message echo
                let response = FromServer::Message {
                    group_name,
                    message: message.clone(),
                };
                self.on_message_callback.emit(response);
            }
        }

        self.on_status_change.emit(ConnectionStatus::Connected);
        Ok(())
    }
}

/// Unified connection service that can use WebSocket or HTTP
#[derive(Clone)]
pub struct ConnectionService {
    websocket_service: Rc<std::cell::RefCell<WebSocketService>>,
    http_service: HttpService,
    use_websocket: bool,
}

impl ConnectionService {
    pub fn new(
        on_message_callback: Callback<FromServer>,
        on_status_change: Callback<ConnectionStatus>,
    ) -> Self {
        Self {
            websocket_service: Rc::new(std::cell::RefCell::new(WebSocketService::new(
                on_message_callback.clone(),
                on_status_change.clone(),
            ))),
            http_service: HttpService::new(on_message_callback, on_status_change),
            use_websocket: true,
        }
    }

    /// Connect to server using WebSocket or HTTP fallback
    pub async fn connect(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.use_websocket = true;

        // Try WebSocket first
        if let Ok(_) = self.websocket_service.borrow_mut().connect(url) {
            log::info!("Connected via WebSocket");
            return Ok(());
        }

        // Fall back to HTTP
        log::warn!("WebSocket failed, falling back to HTTP simulation");
        self.use_websocket = false;
        self.http_service
            .send(FromClient::Join {
                group_name: std::sync::Arc::new("general".to_string()),
            })
            .await?;
        Ok(())
    }

    /// Send a message to the server
    pub async fn send(&self, message: FromClient) -> Result<(), Box<dyn std::error::Error>> {
        if self.use_websocket {
            self.websocket_service.borrow().send(message)?;
        } else {
            self.http_service.send(message).await?;
        }
        Ok(())
    }

    /// Disconnect from server
    pub fn disconnect(&mut self) {
        if self.use_websocket {
            self.websocket_service.borrow_mut().disconnect();
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        if self.use_websocket {
            self.websocket_service.borrow().is_connected()
        } else {
            true // HTTP simulation is always "connected"
        }
    }

    /// Set to HTTP mode (for development/testing)
    pub fn set_http_mode(&mut self) {
        self.use_websocket = false;
    }
}

/// Hook for using the connection service in components
#[hook]
pub fn use_connection_service(
    on_message: Callback<FromServer>,
    on_status_change: Callback<ConnectionStatus>,
) -> UseStateHandle<ConnectionService> {
    let service = use_state(|| ConnectionService::new(on_message, on_status_change));
    service
}
