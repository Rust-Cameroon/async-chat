#![allow(dead_code, unused_variables, unused_mut)] // Suppresses warnings

use async_chat::{FromServer, utils};
use async_std::{io::BufReader, net, prelude::FutureExt, stream::StreamExt, task};

/// Client binary for connecting to the async chat server.
///
/// Expects one argument: the server address and port to connect to.
/// Example usage: `client 127.0.0.1:8080`
fn main() -> anyhow::Result<()> {
    let address = std::env::args().nth(1).expect("Usage: client ADDRESS:PORT");

    task::block_on(async {
        let socket = net::TcpStream::connect(address).await?;
        socket.set_nodelay(true)?; // Disable Nagle's algorithm for lower latency.

        // Race two futures: sending commands vs. receiving server.
        let to_server = send_commands(socket.clone());
        let from_server = handle_replies(socket);

        from_server.race(to_server).await?;
        Ok(())
    })
}

/// Reads user input (planned via `clap`) and sends commands to the server.
async fn send_commands(_to_server: net::TcpStream) -> anyhow::Result<()> {
    // TODO: Implement use clap to parse command line arguments and print help message
    todo!()
}
/// Handles responses from the server and prints them to stdout as they arrive.
async fn handle_replies(from_server: net::TcpStream) -> anyhow::Result<()> {
    let buffered = BufReader::new(from_server);
    let mut reply_stream = utils::receive_as_json(buffered);

    while let Some(reply) = reply_stream.next().await {
        let reply = reply?;
        match reply {
            FromServer::Message {
                group_name,
                message,
            } => {
                println!("message posted to {}: {}", group_name, message);
            }
            FromServer::Error(error) => {
                eprintln!("Error: {}", error);
            }
        }
    }

    Ok(())
}
