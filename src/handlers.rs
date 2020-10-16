use crate::models::Db;
use serde_json::json;
use serenity::http::Http;

pub async fn move_users_to_original(
    guild_id: String,
    token: String,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    let http = Http::new_with_token(&token);
    {
        let map = db.read().await;

        let guild_vec = match map.get(&guild_id) {
            Some(guild_vec) => guild_vec,
            None => return Err(warp::reject::not_found()),
        };

        for user in guild_vec.iter() {
            let mut map = serde_json::Map::new();
            map.insert("channel_id".to_string(), json!(user.original_channel));

            http.edit_member(
                guild_id.parse().expect("This should be a number"),
                user.id,
                &map,
            )
            .await
            .expect("That shouldn't have happened")
        }
    }
    Ok(warp::reply::json(&"Moved users"))
}

pub async fn move_users_to_group(
    guild_id: String,
    token: String,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    let http = Http::new_with_token(&token);
    {
        let map = db.read().await;

        let guild_vec = match map.get(&guild_id) {
            Some(guild_vec) => guild_vec,
            None => return Err(warp::reject::not_found()),
        };

        println!("{:#?}", guild_vec);

        for user in guild_vec.iter() {
            let mut map = serde_json::Map::new();
            map.insert("channel_id".to_string(), json!(user.new_channel));

            http.edit_member(
                guild_id.parse().expect("This should be a number"),
                user.id,
                &map,
            )
            .await
            .expect("That shouldn't have happened")
        }
    }

    Ok(warp::reply::json(&"Moved two users"))
}
