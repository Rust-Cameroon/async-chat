#![allow(dead_code)] // Suppresses warnings about unused code

use crate::connection::Outbound;
use async_std::task;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Group {
    name: Arc<String>,
    sender: broadcast::Sender<Arc<String>>,
}

impl Group {
    pub fn new(name: Arc<String>) -> Group {
        let (sender, _receiver) = broadcast::channel(1000);
        Group { name, sender }
    }

    pub fn join(&self, outbound: Arc<Outbound>) {
        let receiver = self.sender.subscribe();
        task::spawn(handle_subscriber(self.name.clone(), receiver, outbound));
    }

    pub fn post(&self, message: Arc<String>) {
        let _ = self.sender.send(message); // Ignoring the result to suppress warning
    }
}

async fn handle_subscriber(
    _group_name: Arc<String>,
    _receiver: broadcast::Receiver<Arc<String>>,
    _outbound: Arc<Outbound>,
) {
    todo!()
}
