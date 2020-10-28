use crate::models::{Db, Member, MoveGroup};
use serde_json::json;
use serenity::http::Http;
use sqlx::{query_as, Pool, Postgres};

pub async fn move_users_to_original(
    guild_id: String,
    token: String,
    pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let http = Http::new_with_token(&token);
    {
        // let map = db.read().await;
        //
        // let guild_vec = match map.get(&guild_id) {
        //     Some(guild_vec) => guild_vec,
        //     None => return Err(warp::reject::not_found()),
        // };
        //
        // for user in guild_vec.iter() {
        //     let mut map = serde_json::Map::new();
        //     map.insert("channel_id".to_string(), json!(user.original_channel));
        //
        //     http.edit_member(
        //         guild_id.parse().expect("This should be a number"),
        //         user.id,
        //         &map,
        //     )
        //     .await
        //     .expect("That shouldn't have happened")
        // }
    }
    Ok(warp::reply::json(&"Moved users"))
}

pub async fn move_users_to_group(
    guild_id: String,
    token: String,
    pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let http = Http::new_with_token(&token);
    {
        // let map = db.read().await;
        //
        //
        //
        // let guild_vec = match map.get(&guild_id) {
        //     Some(guild_vec) => guild_vec,
        //     None => return Err(warp::reject::not_found()),
        // };
        //
        // println!("{:#?}", guild_vec);

        let guild_vec: Vec<MoveGroup> = query_as!(
            MoveGroup,
            "SELECT * FROM move_groups WHERE guild_id = $1",
            guild_id
        )
        .fetch_all(&pool)
        .await
        .expect("Hopefully there is a move group");

        for group in guild_vec.iter() {
            let member_vec: Vec<Member> = query_as!(
                Member,
                "SELECT * FROM members WHERE move_group_id = $1",
                group.id
            )
            .fetch_all(&pool)
            .await
            .expect("Hopefully there are members");

            for member in member_vec.iter() {
                let mut map = serde_json::Map::new();
                map.insert("channel_id".to_string(), json!(group.group_channel));

                http.edit_member(
                    guild_id.parse().expect("This should be a number"),
                    (&member.user_id).parse().unwrap(),
                    &map,
                )
                .await
                .expect("That shouldn't have happened")
            }
        }
    }

    Ok(warp::reply::json(&"Moved two users"))
}
