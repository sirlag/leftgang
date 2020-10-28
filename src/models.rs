use serde::{Deserialize, Serialize};
use serenity::prelude::TypeMapKey;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type Db = Arc<RwLock<HashMap<String, Vec<User>>>>;
pub struct DbKey;
impl TypeMapKey for DbKey {
    type Value = Pool<Postgres>;
}

pub fn blank_db() -> Db {
    Arc::new(RwLock::new(HashMap::new()))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: u64,
    pub original_channel: u64,
    pub new_channel: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Guild {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MoveGroup {
    pub id: Uuid,
    pub guild_id: String,
    pub name: String,
    pub home_channel: String,
    pub group_channel: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Member {
    pub id: Uuid,
    pub user_id: String,
    pub move_group_id: Uuid,
}
