pub mod connection;
pub mod group;
pub mod group_table;
pub mod websocket;

use connection::serve;
use websocket::{handle_websocket_connection, is_websocket_upgrade, start_websocket_server};

use async_std::net::TcpListener;
use async_std::prelude::*;
use async_std::task;
use log::{error, info};
use std::sync::Arc;

/// The main entry point for the async-chat server.
///
/// Accepts incoming TCP and WebSocket connections and spawns tasks to handle each.
/// Supports both existing CLI clients and new web clients.
///
/// Usage: server <TCP_ADDRESS> <WS_ADDRESS>
/// Example: server 127.0.0.1:8080 127.0.0.1:8081
fn main() -> anyhow::Result<()> {
    // Initialize logger
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <TCP_ADDRESS> [WS_ADDRESS]", args[0]);
        eprintln!("Example: {} 127.0.0.1:8080 127.0.0.1:8081", args[0]);
        eprintln!("If WS_ADDRESS is omitted, TCP_ADDRESS+1 will be used for WebSocket");
        std::process::exit(1);
    }

    let tcp_address = args[1].clone();
    let ws_address = if args.len() > 2 {
        args[2].clone()
    } else {
        // Auto-generate WebSocket address (TCP port + 1)
        format!(
            "127.0.0.1:{}",
            tcp_address
                .split(':')
                .last()
                .unwrap_or("8080")
                .parse::<u16>()
                .unwrap_or(8080)
                + 1
        )
    };

    // A thread-safe table that stores all active chat groups by name.
    let chat_group_table = Arc::new(group_table::GroupTable::new());

    // Create a runtime that can handle both async-std and tokio
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let groups_clone = chat_group_table.clone();
        let ws_addr_clone = ws_address.clone();

        // Start WebSocket server in background
        let ws_handle = tokio::spawn(async move {
            if let Err(e) = start_websocket_server(&ws_addr_clone, groups_clone).await {
                error!("WebSocket server error: {}", e);
            }
        });

        // Give WebSocket server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        info!("Starting TCP server on: {}", tcp_address);
        info!("WebSocket server listening on: {}", ws_address);
        info!("You can connect CLI clients to: {}", tcp_address);
        info!("Web clients can connect to: {}", ws_address);

        // Handle TCP connections in async-std runtime
        async_std::task::spawn(async move {
            if let Err(e) = run_tcp_server(tcp_address, chat_group_table).await {
                error!("TCP server error: {}", e);
            }
        });

        // Wait for WebSocket server (this will run forever)
        if let Err(e) = ws_handle.await {
            error!("WebSocket server task error: {}", e);
        }

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

/// Run the TCP server for CLI clients
async fn run_tcp_server(address: String, chat_group_table: Arc<GroupTable>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(&address).await?;
    let mut new_connections = listener.incoming();

    info!("TCP server listening on: {}", address);

    // Accept incoming connections and spawn an asynchronous task to handle each
    while let Some(socket_result) = new_connections.next().await {
        let socket = socket_result?;
        info!("New TCP connection from: {}", socket.peer_addr()?);

        // Check if this might be a WebSocket upgrade request
        if let Ok(is_ws) = is_websocket_upgrade(&socket).await {
            if is_ws {
                info!("Detected WebSocket upgrade request on TCP port, handling as WebSocket");
                let groups = chat_group_table.clone();
                task::spawn(async {
                    if let Err(e) = handle_websocket_connection(socket, groups).await {
                        error!("WebSocket connection error: {}", e);
                    }
                });
                continue;
            }
        }

        // Handle as regular TCP connection
        let groups = chat_group_table.clone();
        task::spawn(async {
            log_error(serve(socket, groups).await);
        });
    }
    Ok(())
}

/// Logs errors from client handler tasks.
fn log_error(result: anyhow::Result<()>) {
    if let Err(error) = result {
        eprintln!("Error: {:?}", error);
    }
}
