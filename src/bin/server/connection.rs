use crate::group_table::GroupTable;
use crate::user::UserManager;
use async_chat::utils::{self};
use async_chat::{FromClient, FromServer, User};
use async_std::io::BufReader;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::Arc;
use async_std::sync::Mutex;

pub struct Outbound(Mutex<TcpStream>);

impl Outbound {
    pub fn new(to_client: TcpStream) -> Outbound {
        Outbound(Mutex::new(to_client))
    }
    pub async fn send(&self, packet: FromServer) -> anyhow::Result<()> {
        let mut guard = self.0.lock().await;
        utils::send_as_json(&mut *guard, &packet).await?;
        guard.flush().await?;
        Ok(())
    }
}

pub struct Connection {
    user: Option<User>,
}

impl Connection {
    pub fn new() -> Self {
        Connection { user: None }
    }

    pub fn set_user(&mut self, user: User) {
        self.user = Some(user);
    }

    pub fn get_user(&self) -> Option<&User> {
        self.user.as_ref()
    }

    pub fn clear_user(&mut self) {
        self.user = None;
    }
}

pub async fn serve(
    socket: TcpStream,
    groups: Arc<GroupTable>,
    user_manager: Arc<UserManager>,
) -> anyhow::Result<()> {
    let outbound = Arc::new(Outbound::new(socket.clone()));
    let mut connection = Connection::new();
    let buffered = BufReader::new(socket);
    let mut from_client = utils::receive_as_json(buffered);

    while let Some(request_result) = from_client.next().await {
        let request = request_result?;
        let result = match request {
            FromClient::Register {
                username,
                password: _,
            } => match user_manager.register(username.clone()).await {
                Ok(user) => {
                    connection.set_user(user.clone());
                    outbound.send(FromServer::AuthSuccess { user }).await
                }
                Err(e) => outbound.send(FromServer::AuthError(e.to_string())).await,
            },
            FromClient::Login {
                username,
                password: _,
            } => match user_manager.login(username.clone()).await {
                Ok(user) => {
                    connection.set_user(user.clone());
                    outbound.send(FromServer::AuthSuccess { user }).await
                }
                Err(e) => outbound.send(FromServer::AuthError(e.to_string())).await,
            },
            FromClient::Logout => {
                if let Some(user) = connection.get_user() {
                    if let Err(e) = user_manager.logout(user.username.clone()).await {
                        return Err(anyhow::anyhow!("Logout error: {}", e));
                    }
                }
                connection.clear_user();
                Ok(())
            }
            FromClient::Join { group_name } => {
                if connection.get_user().is_none() {
                    outbound
                        .send(FromServer::Error("Not authenticated".to_string()))
                        .await
                } else {
                    let group = groups.get_or_create(group_name).await;
                    group.join(outbound.clone());
                    Ok(())
                }
            }
            FromClient::Post {
                group_name,
                message,
            } => {
                if let Some(user) = connection.get_user() {
                    match groups.get(&group_name).await {
                        Some(group) => {
                            group.post(message, user.clone());
                            Ok(())
                        }
                        None => {
                            outbound
                                .send(FromServer::Error(format!(
                                    "Group '{}' does not exist",
                                    group_name
                                )))
                                .await
                        }
                    }
                } else {
                    outbound
                        .send(FromServer::Error("Not authenticated".to_string()))
                        .await
                }
            }
        };

        if let Err(e) = result {
            eprintln!("Error handling request: {}", e);
        }
    }
    Ok(())
}
