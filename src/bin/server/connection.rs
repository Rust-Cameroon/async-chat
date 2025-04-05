use crate::group_table::GroupTable;
use async_chat::utils::{self};
use async_chat::{FromClient, FromServer};
use async_std::io::BufReader;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::{Arc, Mutex};
use anyhow::{bail, Result};

/// Wraps a TCP connection to a client, allowing safe async writes.
pub struct Outbound(Mutex<TcpStream>); 

impl Outbound {
    pub fn new(to_client: TcpStream) -> Self {
        Self(Mutex::new(to_client))
    }

    pub async fn send(&self, packet: FromServer) -> Result<()> {
        let mut guard = self.0.lock().await;
        utils::send_as_json(&mut *guard, &packet).await?;
        guard.flush().await?;
        Ok(())
    }
}

/// Handles a new client connection, listens for messages, and interacts with group logic.
pub async fn serve(socket: TcpStream, groups: Arc<GroupTable>) -> Result<()> {
    let outbound = Arc::new(Outbound::new(socket.clone()));
    let buffered = BufReader::new(socket);
    let mut from_client = utils::receive_as_json(buffered);

    while let Some(request_result) = from_client.next().await {
        let request = request_result?;
        let result: Result<()> = match request {
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
                None => bail!("Group '{}' does not exist", group_name),
            },
        };

        if let Err(message) = result {
            let report = FromServer::Error(message.to_string());
            outbound.send(report).await?;
        }
    }

    println!("Client disconnected.");
    Ok(())
}
