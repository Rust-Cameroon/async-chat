pub mod connection;
pub mod group;
pub mod group_table;

use connection::serve;

use async_std::net::TcpListener;
use async_std::prelude::*;
use async_std::task;
use std::sync::Arc;

/// The main entry point for the async-chat server.
///
/// Accepts incoming TCP connections and spawns a task to handle each.
/// Expects one argument: the address to bind to (e.g., `127.0.0.1:8080`)
fn main() -> anyhow::Result<()> {
    let address = std::env::args().nth(1).expect(
        "Usage: server
    ADDRESS",
    );
    // A thread-safe table that stores all active chat groups by name.
    let chat_group_table = Arc::new(group_table::GroupTable::new());
    async_std::task::block_on(async {
        let listener = TcpListener::bind(address).await?;
        let mut new_connections = listener.incoming();
        // Accept incoming connections and spawn an asynchronous task to handle each
        while let Some(socket_result) = new_connections.next().await {
            let socket = socket_result?;
            let groups = chat_group_table.clone();
            task::spawn(async {
                log_error(serve(socket, groups).await);
            });
        }
        Ok(())
    })
}

/// Logs errors from client handler tasks.
fn log_error(result: anyhow::Result<()>) {
    if let Err(error) = result {
        eprintln!("Error: {}", error);
    }
}
