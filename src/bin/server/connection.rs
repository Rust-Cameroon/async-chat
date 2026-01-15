use crate::group_table::GroupTable;
use async_chat::{FromClient, FromServer};
use async_std::net::TcpStream;
use async_std::sync::Arc;
use async_std::sync::Mutex;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};

/// Represents a thread-safe outbound connection to a client.
/// This struct wraps the write-half (Sink) of a `WebSocketStream` in a `Mutex`.
pub struct Outbound(Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>);

impl Outbound {
    /// Creates a new `Outbound` connection from a WebSocket sink.
    pub fn new(sink: SplitSink<WebSocketStream<TcpStream>, Message>) -> Outbound {
        Outbound(Mutex::new(sink))
    }

    /// Sends a message to the connected client in JSON format.
    pub async fn send(&self, packet: FromServer) -> anyhow::Result<()> {
        let mut guard = self.0.lock().await;
        let json = serde_json::to_string(&packet)?;
        guard.send(Message::Text(json)).await?;
        Ok(())
    }
}

/// Serves a single client connection by reading messages and interacting with group state.
pub async fn serve(socket: WebSocketStream<TcpStream>, groups: Arc<GroupTable>) -> anyhow::Result<()> {
    let (sink, mut stream) = socket.split();
    let outbound = Arc::new(Outbound::new(sink));
    
    // receive data from clients
    while let Some(msg_result) = stream.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                // If the message is empty or just whitespace, skip it
                if text.trim().is_empty() {
                    continue;
                }

                match serde_json::from_str::<FromClient>(&text) {
                    Ok(request) => {
                        let result = match request {
                            FromClient::Join { group_name } => {
                                let group = groups.get_or_create(group_name);
                                group.join(outbound.clone());
                                Ok(())
                            }
                            FromClient::Post {
                                group_name,
                                author,
                                message,
                            } => {
                                eprintln!("Server: Received Post to group '{}' from '{}': {}", group_name, author, message);
                                match groups.get(&*group_name) {
                                    Some(group) => {
                                        group.post(author, message);
                                        Ok(())
                                    }
                                    None => {
                                        eprintln!("Server Error: Group '{}' not found", group_name);
                                        Err(format!("Group '{}' does not exist", group_name))
                                    }
                                }
                            }
                            FromClient::RequestGroups => {
                                let list = groups.list_groups();
                                let _ = outbound.send(FromServer::GroupsList(list)).await;
                                Ok(())
                            }
                            FromClient::PostFile { group_name, author, filename, data } => {
                                match groups.get(&*group_name) {
                                    Some(group) => {
                                        group.post_file(author, filename, data);
                                        Ok(())
                                    }
                                    None => Err(format!("Group '{}' does not exist", group_name)),
                                }
                            }
                            FromClient::PostVoice { group_name, author, duration, data } => {
                                match groups.get(&*group_name) {
                                    Some(group) => {
                                        group.post_voice(author, duration, data);
                                        Ok(())
                                    }
                                    None => Err(format!("Group '{}' does not exist", group_name)),
                                }
                            }
                            FromClient::PostReaction { group_name, author, message_id, emoji } => {
                                match groups.get(&*group_name) {
                                    Some(group) => {
                                        group.post_reaction(author, message_id, emoji);
                                        Ok(())
                                    }
                                    None => Err(format!("Group '{}' does not exist", group_name)),
                                }
                            }
                        };
                        // If an error occurred (logical error), send an error message back to the client
                        if let Err(message) = result {
                            let report = FromServer::Error(message);
                            outbound.send(report).await?;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: expected value or malformed JSON from client: {}. Raw input: {:?}", e, text);
                        // We skip this message but keep the connection open
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => continue, // Ignore other message types like Binary, Ping, Pong
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
