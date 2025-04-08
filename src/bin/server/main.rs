use async_std::net::TcpListener;
use async_std::stream::StreamExt;
use async_std::sync::Arc;
use async_std::task;

mod connection;
mod group;
mod group_table;
mod user;

use connection::serve;
use group_table::GroupTable;
use user::UserManager;

fn main() -> anyhow::Result<()> {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    task::block_on(async {
        let listener = TcpListener::bind(&address).await?;
        println!("Server listening on {}", address);

        let groups = Arc::new(GroupTable::new());
        let user_manager = Arc::new(UserManager::new());

        let mut handles = vec![];
        while let Some(stream) = listener.incoming().next().await {
            let stream = stream?;
            let groups = groups.clone();
            let user_manager = user_manager.clone();

            let handle = task::spawn(async move {
                if let Err(e) = serve(stream, groups, user_manager).await {
                    eprintln!("Error serving connection: {}", e);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await;
        }
        Ok(())
    })
}
