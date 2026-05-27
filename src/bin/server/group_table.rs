#![allow(clippy::new_without_default)] // Suppresses Clippy warning

use crate::group::Group;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A thread-safe table that stores all active chat groups by name.
///
/// Internally wraps a `HashMap<Arc<String>, Arc<Group>>` in a `Mutex` for safe concurrent access.
pub struct GroupTable(Mutex<HashMap<Arc<String>, Arc<Group>>>);

/// Result of creating a group.
#[derive(Debug, PartialEq)]
pub enum CreateResult {
    /// Group was successfully created.
    Success,
    /// Group already exists.
    AlreadyExists,
}

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

    /// Creates a new group if it doesn't already exist.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the group to create.
    /// * `password` - Optional password for the group.
    ///
    /// # Returns
    ///
    /// `CreateResult::Success` if the group was created,
    /// `CreateResult::AlreadyExists` if the group already exists.
    pub fn create(&self, name: Arc<String>, password: Option<Arc<String>>) -> CreateResult {
        let mut groups = self.0.lock().unwrap();
        if groups.contains_key(&*name) {
            CreateResult::AlreadyExists
        } else {
            let group = Arc::new(Group::new(name.clone(), password));
            groups.insert(name, group);
            CreateResult::Success
        }
    }

    /// Lists all group names in the table.
    ///
    /// # Returns
    ///
    /// A vector of all group names.
    pub fn list_groups(&self) -> Vec<Arc<String>> {
        let groups = self.0.lock().unwrap();
        groups.keys().cloned().collect()
    }

    /// Retrieves a group by name, or creates one without a password if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the group to retrieve or create.
    ///
    /// # Returns
    ///
    /// An `Arc<Group>` for the existing or newly created group.
    ///
    /// # Note
    ///
    /// This method is preserved for backward compatibility. New code should use `create()`
    /// for explicit group creation.
    pub fn get_or_create(&self, name: Arc<String>) -> Arc<Group> {
        self.0
            .lock()
            .unwrap()
            .entry(name.clone())
            .or_insert_with(|| Arc::new(Group::new(name.clone(), None)))
            .clone()
    }
}

// Implement Default to satisfy Clippy's `new_without_default` lint
impl Default for GroupTable {
    fn default() -> Self {
        Self::new()
    }
}
