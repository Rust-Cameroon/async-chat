use async_chat::User;
use async_std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

pub struct UserManager {
    users: Mutex<HashMap<Arc<String>, User>>,
    active_users: Mutex<HashMap<Arc<String>, Arc<String>>>,
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            users: Mutex::new(HashMap::new()),
            active_users: Mutex::new(HashMap::new()),
        }
    }

    pub async fn register(&self, username: Arc<String>) -> anyhow::Result<User> {
        let mut users = self.users.lock().await;
        if users.contains_key(&username) {
            return Err(anyhow::anyhow!("Username already exists"));
        }

        let user = User {
            username: username.clone(),
            id: Arc::new(Uuid::new_v4().to_string()),
        };

        users.insert(username.clone(), user.clone());
        Ok(user)
    }

    pub async fn login(&self, username: Arc<String>) -> anyhow::Result<User> {
        let users = self.users.lock().await;
        let user = users
            .get(&username)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let mut active_users = self.active_users.lock().await;
        active_users.insert(username, user.id.clone());

        Ok(user)
    }

    pub async fn logout(&self, username: Arc<String>) -> anyhow::Result<()> {
        let mut active_users = self.active_users.lock().await;
        active_users
            .remove(&username)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn is_active(&self, username: &Arc<String>) -> bool {
        let active_users = self.active_users.lock().await;
        active_users.contains_key(username)
    }
}
