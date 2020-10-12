use serde::{Deserialize, Serialize};
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type Db = Arc<RwLock<Vec<User>>>;
pub struct DbKey;
impl TypeMapKey for DbKey {
    type Value = Db;
}

pub fn blank_db() -> Db {
    Arc::new(RwLock::new(Vec::new()))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: u64,
    pub original_channel: u64,
    pub new_channel: u64,
}
