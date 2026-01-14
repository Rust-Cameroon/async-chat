#![allow(dead_code, unused_variables, unused_mut)] // Suppresses warnings

use async_chat::{FromClient, FromServer, utils};
use async_std::{
    io::{BufReadExt, BufReader, stdin},
    net,
    prelude::FutureExt,
    stream::StreamExt,
    task,
};
use std::sync::Arc;

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

/// Reads user input and sends commands to the server.
///
/// Commands:
/// - `/join <group_name>` - Join a chat group
/// - `/post <group_name> <message>` - Post a message to a group
/// - `/help` - Show help message
/// - `/quit` - Exit the client
async fn send_commands(to_server: net::TcpStream) -> anyhow::Result<()> {
    let mut to_server = to_server;
    println!("Welcome to Async Chat!");
    println!("Commands:");
    println!("  /join <group_name>     - Join a chat group");
    println!("  /post <group_name> <message> - Post a message to a group");
    println!("  /help                  - Show this help message");
    println!("  /quit                  - Exit the client");
    println!();

    let stdin = BufReader::new(stdin());
    let mut lines = stdin.lines();

    while let Some(line_result) = lines.next().await {
        let line = line_result?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if line == "/quit" {
            println!("Goodbye!");
            break;
        }

        if line == "/help" {
            println!("Commands:");
            println!("  /join <group_name>     - Join a chat group");
            println!("  /post <group_name> <message> - Post a message to a group");
            println!("  /help                  - Show this help message");
            println!("  /quit                  - Exit the client");
            continue;
        }

        let command = parse_command(line);
        match command {
            Ok(from_client) => {
                if let Err(e) = utils::send_as_json(&mut to_server, &from_client).await {
                    eprintln!("Failed to send command: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                eprintln!("Type /help for available commands.");
            }
        }
    }

    Ok(())
}

/// Parses a command line input into a FromClient message.
///
/// # Arguments
/// * `input` - The user input string to parse
///
/// # Returns
/// A Result containing either a FromClient message or an error string
fn parse_command(input: &str) -> Result<FromClient, String> {
    let parts: Vec<&str> = input.splitn(3, ' ').collect();

    match parts.as_slice() {
        ["/join", group_name] => {
            if group_name.is_empty() {
                return Err("Group name cannot be empty".to_string());
            }
            Ok(FromClient::Join {
                group_name: Arc::new(group_name.to_string()),
            })
        }
        ["/post", group_name, message] => {
            if group_name.is_empty() {
                return Err("Group name cannot be empty".to_string());
            }
            if message.is_empty() {
                return Err("Message cannot be empty".to_string());
            }
            Ok(FromClient::Post {
                group_name: Arc::new(group_name.to_string()),
                author: Arc::new("CLI-User".to_string()),
                message: Arc::new(message.to_string()),
            })
        }
        ["/join"] => Err("Usage: /join <group_name>".to_string()),
        ["/post"] => Err("Usage: /post <group_name> <message>".to_string()),
        ["/post", _] => Err("Usage: /post <group_name> <message>".to_string()),
        _ => Err(format!(
            "Unknown command: '{}'. Type /help for available commands.",
            input
        )),
    }
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
                author,
                message,
            } => {
                println!("{}: message posted to {}: {}", author, group_name, message);
            }
            FromServer::Error(error) => {
                eprintln!("Error: {}", error);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_join_command() {
        let result = parse_command("/join general");
        assert!(result.is_ok());
        match result.unwrap() {
            FromClient::Join { group_name } => {
                assert_eq!(*group_name, "general".to_string());
            }
            _ => panic!("Expected Join command"),
        }
    }

    #[test]
    fn test_parse_post_command() {
        let result = parse_command("/post general Hello world!");
        assert!(result.is_ok());
        match result.unwrap() {
            FromClient::Post {
                group_name,
                author,
                message,
            } => {
                assert_eq!(*group_name, "general".to_string());
                assert_eq!(*author, "CLI-User".to_string());
                assert_eq!(*message, "Hello world!".to_string());
            }
            _ => panic!("Expected Post command"),
        }
    }

    #[test]
    fn test_parse_invalid_command() {
        let result = parse_command("/invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown command"));
    }

    #[test]
    fn test_parse_join_without_group() {
        let result = parse_command("/join");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Usage: /join <group_name>");
    }

    #[test]
    fn test_parse_post_without_message() {
        let result = parse_command("/post general");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Usage: /post <group_name> <message>");
    }

    #[test]
    fn test_parse_post_without_group() {
        let result = parse_command("/post");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Usage: /post <group_name> <message>");
    }
}
