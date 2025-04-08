#![allow(clippy::new_without_default)] // Suppresses Clippy warning

use crate::group::Group;
use async_std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct GroupTable {
    groups: Mutex<HashMap<Arc<String>, Arc<Group>>>,
}

impl GroupTable {
    pub fn new() -> GroupTable {
        GroupTable {
            groups: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get(&self, name: &Arc<String>) -> Option<Arc<Group>> {
        let groups = self.groups.lock().await;
        groups.get(name).cloned()
    }

    pub async fn get_or_create(&self, name: Arc<String>) -> Arc<Group> {
        let mut groups = self.groups.lock().await;
        groups.get(&name).cloned().unwrap_or_else(|| {
            let group = Arc::new(Group::new(name.clone()));
            groups.insert(name, group.clone());
            group
        })
    }
}

// Implement Default to satisfy Clippy's `new_without_default` lint
impl Default for GroupTable {
    fn default() -> Self {
        Self::new()
    }
}
