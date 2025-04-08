use async_chat::{FromClient, FromServer, utils};
use async_std::{io::BufReader, net, prelude::*, stream::StreamExt, task};
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::sync::Arc;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Server address (e.g., 127.0.0.1:8080)
    address: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Join a chat group
    Join {
        /// Name of the group to join
        #[arg(short, long)]
        group: String,
    },
    /// Post a message to a group
    Post {
        /// Name of the group to post to
        #[arg(short, long)]
        group: String,
        /// Message to post
        #[arg(short, long)]
        message: String,
    },
    /// Start interactive chat mode
    Chat {
        /// Name of the group to chat in
        #[arg(short, long)]
        group: String,
        /// Username to use
        #[arg(short, long)]
        username: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    task::block_on(async {
        let mut socket = net::TcpStream::connect(&cli.address).await?;
        socket.set_nodelay(true)?;

        match cli.command {
            Commands::Chat { group, username } => {
                // First register/login the user
                let register_command = FromClient::Register {
                    username: Arc::new(username.clone()),
                    password: Arc::new("password".to_string()), // TODO: Add proper password handling
                };
                utils::send_as_json(&mut socket, &register_command).await?;
                socket.flush().await?;

                // Then join the group
                let join_command = FromClient::Join {
                    group_name: Arc::new(group.clone()),
                };
                utils::send_as_json(&mut socket, &join_command).await?;
                socket.flush().await?;

                // Then start interactive mode
                interactive_chat(socket, group, username).await?;
            }
            _ => {
                let to_server = send_commands(socket.clone(), cli.command);
                let from_server = handle_replies(socket);
                from_server.race(to_server).await?;
            }
        }
        Ok(())
    })
}

async fn interactive_chat(
    mut socket: net::TcpStream,
    group: String,
    _username: String,
) -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let socket_clone = socket.clone();

    // Spawn a task to handle incoming messages
    let handle_replies = task::spawn(async move { handle_replies(socket_clone).await });

    // Main loop for sending messages
    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "/quit" {
            break;
        }

        let command = FromClient::Post {
            group_name: Arc::new(group.clone()),
            message: Arc::new(input.to_string()),
        };

        utils::send_as_json(&mut socket, &command).await?;
        socket.flush().await?;
    }

    // Wait for the reply handler to finish
    handle_replies.await?;
    Ok(())
}

async fn send_commands(mut to_server: net::TcpStream, command: Commands) -> anyhow::Result<()> {
    let command = match command {
        Commands::Join { group } => FromClient::Join {
            group_name: Arc::new(group),
        },
        Commands::Post { group, message } => FromClient::Post {
            group_name: Arc::new(group),
            message: Arc::new(message),
        },
        Commands::Chat { .. } => unreachable!(), // Handled separately
    };

    utils::send_as_json(&mut to_server, &command).await?;
    to_server.flush().await?;
    Ok(())
}

async fn handle_replies(from_server: net::TcpStream) -> anyhow::Result<()> {
    let buffered = BufReader::new(from_server);
    let mut reply_stream = utils::receive_as_json(buffered);

    while let Some(reply) = reply_stream.next().await {
        let reply = reply?;
        match reply {
            FromServer::Message {
                group_name,
                message,
                sender,
            } => {
                println!("\n[{}] {}: {}", group_name, sender.username, message);
                print!("> ");
                io::stdout().flush().unwrap();
            }
            FromServer::Error(error) => {
                eprintln!("\nError: {}", error);
                print!("> ");
                io::stdout().flush().unwrap();
            }
            FromServer::AuthSuccess { user } => {
                println!("\nSuccessfully authenticated as {}", user.username);
                print!("> ");
                io::stdout().flush().unwrap();
            }
            FromServer::AuthError(error) => {
                eprintln!("\nAuthentication error: {}", error);
                print!("> ");
                io::stdout().flush().unwrap();
            }
        }
    }
    Ok(())
}
