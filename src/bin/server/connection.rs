use crate::group_table::GroupTable;
use async_chat::{FromClient, FromServer};
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::Arc;
use async_std::sync::Mutex;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use futures_util::{SinkExt, StreamExt};

/// Represents a thread-safe outbound connection to a client.
/// This struct wraps a `WebSocketStream` in a `Mutex` to provide a safe and exclusive way to send data to the client.
pub struct Outbound(Mutex<WebSocketStream<TcpStream>>);
impl Outbound {
    /// Creates a new `Outbound` connection.
    ///
    /// # Arguments
    ///
    /// * `to_client` - The WebSocket stream to write to.
    pub fn new(to_client: WebSocketStream<TcpStream>) -> Outbound {
        Outbound(Mutex::new(to_client))
    }
    /// Sends a message to the connected client in JSON format.
    ///
    /// # Arguments
    ///
    /// * `packet` - The message to send, wrapped in the `FromServer` enum.
    ///
    /// # Errors
    ///
    /// Returns an error if writing or flushing to the stream fails.
    pub async fn send(&self, packet: FromServer) -> anyhow::Result<()> {
        let mut guard = self.0.lock().await;
        let json = serde_json::to_string(&packet)?;
        guard.send(Message::Text(json)).await?;
        Ok(())
    }
}

/// Serves a single client connection by reading messages and interacting with group state.
///
/// # Arguments
///
/// * `socket` - The WebSocket connection to the client.
/// * `groups` - A shared reference to the server's group table.
///
/// # Errors
///
/// Returns an error if:
/// - Reading from the socket fails
/// - Sending a message fails
/// - A user tries to post to a group that does not exist
pub async fn serve(mut socket: WebSocketStream<TcpStream>, groups: Arc<GroupTable>) -> anyhow::Result<()> {
    // wrapping our connection in outbound so as to have exclusive access to it in the groups and avoid interference
    let outbound = Arc::new(Outbound::new(socket));
    
    // receive data from clients
    while let Some(msg_result) = outbound.0.lock().await.next().await {
        let msg = msg_result?;
        if let Message::Text(text) = msg {
            let request: FromClient = serde_json::from_str(&text)?;
            let result = match request {
                FromClient::Join { group_name } => {
                    let group = groups.get_or_create(group_name);
                    group.join(outbound.clone());
                    Ok(())
                }
                FromClient::Post {
                    group_name,
                    message,
                } => match groups.get(&group_name) {
                    Some(group) => {
                        group.post(message);
                        Ok(())
                    }
                    None => Err(format!("Group '{}' does not exist", group_name)),
                },
            };
            // If an error occurred, send an error message back to the client
            if let Err(message) = result {
                let report = FromServer::Error(message);
                // send error back to client
                outbound.send(report).await?;
            }
        }
    }
    Ok(())
}
