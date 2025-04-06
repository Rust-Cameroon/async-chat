use crate::group_table::GroupTable;
use async_chat::utils::{self};
use async_chat::{FromClient, FromServer};
use async_std::io::BufReader;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::Arc;
use async_std::sync::Mutex;

/// Represents a thread-safe outbound connection to a client.
/// This struct wraps a `TcpStream` in a `Mutex` to provide a safe and exclusive way to send data to the client.

pub struct Outbound(Mutex<TcpStream>);
impl Outbound {
    /// Creates a new `Outbound` connection.
    ///
    /// # Arguments
    ///
    /// * `to_client` - The TCP stream to write to.
    pub fn new(to_client: TcpStream) -> Outbound {
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
        utils::send_as_json(&mut *guard, &packet).await?;
        guard.flush().await?;
        Ok(())
    }
}

/// Serves a single client connection by reading messages and interacting with group state.
///
/// # Arguments
///
/// * `socket` - The TCP connection to the client.
/// * `groups` - A shared reference to the server's group table.
///
/// # Errors
///
/// Returns an error if reading from the socket or sending a message fails.
pub async fn serve(socket: TcpStream, groups: Arc<GroupTable>) -> anyhow::Result<()> {
    // wrapping our connection in outbound so as to have exclusive access to it in the groups and avoid interference
    let outbound = Arc::new(Outbound::new(socket.clone()));
    let buffered = BufReader::new(socket);
    // receive data from clients
    let mut from_client = utils::receive_as_json(buffered);
    while let Some(request_result) = from_client.next().await {
        let request = request_result?;
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
    Ok(())
}
