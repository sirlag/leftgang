use crate::models::Db;
use serde_json::json;
use serenity::http::Http;
use std::convert::Infallible;

pub async fn move_users_to_original(token: String, db: Db) -> Result<impl warp::Reply, Infallible> {
    let http = Http::new_with_token(&token);
    {
        let vec = db.read().await;

        for user in vec.iter() {
            let mut map = serde_json::Map::new();
            map.insert("channel_id".to_string(), json!(user.original_channel));

            http.edit_member(340006336659980289, user.id, &map)
                .await
                .expect("That shouldn't have happened")
        }
    }
    Ok(warp::reply::json(&"Moved two users"))
}

pub async fn move_users_to_group(token: String, db: Db) -> Result<impl warp::Reply, Infallible> {
    let http = Http::new_with_token(&token);
    {
        let vec = db.read().await;

        for user in vec.iter() {
            let mut map = serde_json::Map::new();
            map.insert("channel_id".to_string(), json!(user.new_channel));

            http.edit_member(340006336659980289, user.id, &map)
                .await
                .expect("That shouldn't have happened")
        }
    }

    Ok(warp::reply::json(&"Moved two users"))
}
