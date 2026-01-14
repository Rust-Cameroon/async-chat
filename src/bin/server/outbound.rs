use async_chat::{FromServer, utils};
use async_std::net::TcpStream;
use async_std::prelude::*;
use std::sync::Arc;

/// Trait for outbound connections that can send messages to clients
#[async_trait::async_trait]
pub trait OutboundConnection: Send + Sync {
    async fn send(&self, packet: FromServer) -> anyhow::Result<()>;
}

/// TCP outbound connection implementation
pub struct TcpOutbound {
    stream: Arc<async_std::sync::Mutex<TcpStream>>,
}

impl TcpOutbound {
    pub fn new(to_client: TcpStream) -> Self {
        Self {
            stream: Arc::new(async_std::sync::Mutex::new(to_client)),
        }
    }
}

#[async_trait::async_trait]
impl OutboundConnection for TcpOutbound {
    async fn send(&self, packet: FromServer) -> anyhow::Result<()> {
        let mut guard = self.stream.lock().await;
        utils::send_as_json(&mut *guard, &packet).await?;
        guard.flush().await?;
        Ok(())
    }
}

/// Convert from the original Outbound to our new trait-based approach
impl From<crate::connection::Outbound> for Arc<dyn OutboundConnection> {
    fn from(original: crate::connection::Outbound) -> Self {
        // We can't directly convert because the original Outbound doesn't expose the stream
        // For now, we'll need to modify the original approach
        panic!("This conversion is not yet implemented. Please update the connection handling.")
    }
}

// We'll also need to add async_trait to dependencies
// For now, let's create a simpler approach with enum-based outbound
