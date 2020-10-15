use crate::models::Db;
use serde_json::json;
use serenity::http::Http;
use std::convert::Infallible;

pub async fn move_users_to_original(
    guild_id: String,
    token: String,
    db: Db,
) -> Result<impl warp::Reply, Infallible> {
    let http = Http::new_with_token(&token);
    {
        let map = db.read().await;

        let guild_vec = map.get(&guild_id).expect("Just for now");

        for user in guild_vec.iter() {
            let mut map = serde_json::Map::new();
            map.insert("channel_id".to_string(), json!(user.original_channel));

            http.edit_member(guild_id.parse().expect("We need this"), user.id, &map)
                .await
                .expect("That shouldn't have happened")
        }
    }
    Ok(warp::reply::json(&"Moved two users"))
}

pub async fn move_users_to_group(
    guild_id: String,
    token: String,
    db: Db,
) -> Result<impl warp::Reply, Infallible> {
    let http = Http::new_with_token(&token);
    {
        let map = db.read().await;

        let guild_vec = map.get(&guild_id).expect("Just for now");

        println!("{:#?}", guild_vec);

        for user in guild_vec.iter() {
            let mut map = serde_json::Map::new();
            map.insert("channel_id".to_string(), json!(user.new_channel));

            http.edit_member(
                guild_id.parse::<u64>().expect("Crashy crashy"),
                user.id,
                &map,
            )
            .await
            .expect("That shouldn't have happened")
        }
    }

    Ok(warp::reply::json(&"Moved two users"))
}
