#![allow(clippy::new_without_default)] // Suppresses Clippy warning

use crate::group::Group;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A thread-safe table that stores all active chat groups by name.
///
/// Internally wraps a `HashMap<Arc<String>, Arc<Group>>` in a `Mutex` for safe concurrent access.
pub struct GroupTable(Mutex<HashMap<Arc<String>, Arc<Group>>>);

impl GroupTable {
    /// Creates a new, empty `GroupTable`.
    pub fn new() -> GroupTable {
        GroupTable(Mutex::new(HashMap::new()))
    }

    /// Retrieves a group by name, if it exists.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the group to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing the group, or `None` if it doesn't exist.
    pub fn get(&self, name: &String) -> Option<Arc<Group>> {
        self.0.lock().unwrap().get(name).cloned()
    }

    pub fn get_or_create(&self, name: Arc<String>) -> Arc<Group> {
        self.0
            .lock()
            .unwrap()
            .entry(name.clone())
            .or_insert_with(|| Arc::new(Group::new(name)))
            .clone()
    }
}

// Implement Default to satisfy Clippy's `new_without_default` lint
impl Default for GroupTable {
    fn default() -> Self {
        Self::new()
    }
}
