use std::{env, net::SocketAddr};

#[tokio::main]
async fn main() {
    println!("Starting up left-gang service");

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let db = models::blank_db();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let server = warp::serve(filters::movers(token, db)).run(addr);

    let server_task = tokio::spawn(server);
    println!("Listening on http://{}", addr);

    match server_task.await {
        Ok(_) => println!("Closing Server"),
        Err(why) => eprintln!("{}", why),
    };
}

mod filters {
    use super::models::Db;
    use crate::handlers;
    use warp::Filter;

    pub fn movers(
        token: String,
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        move_users_to_new(token.clone(), db.clone())
            .or(move_users_to_original(token, db))
            .or(hello())
    }

    pub fn move_users_to_original(
        token: String,
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("move" / "original")
            .and(warp::get())
            .and(warp::any().map(move || token.clone()))
            .and(with_db(db))
            .and_then(handlers::move_users_to_original)
    }

    pub fn move_users_to_new(
        token: String,
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("move")
            .and(warp::get())
            .and(warp::any().map(move || token.clone()))
            .and(with_db(db))
            .and_then(handlers::move_users_to_group)
    }

    pub fn hello() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("hello" / String).map(|name| format!("Hello, {}!", name))
    }

    fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
}

mod handlers {
    use crate::models::Db;
    use serde_json::json;
    use serenity::http::Http;
    use std::convert::Infallible;

    pub async fn move_users_to_original(
        token: String,
        db: Db,
    ) -> Result<impl warp::Reply, Infallible> {
        let vec = db.lock().await;

        let http = Http::new_with_token(&token);

        for user in vec.iter() {
            let mut map = serde_json::Map::new();
            map.insert("channel_id".to_string(), json!(user.original_channel));

            http.edit_member(340006336659980289, user.id, &map)
                .await
                .expect("That shouldn't have happened")
        }

        Ok(warp::reply::json(&"Moved two users"))
    }

    pub async fn move_users_to_group(
        token: String,
        db: Db,
    ) -> Result<impl warp::Reply, Infallible> {
        let vec = db.lock().await;

        let http = Http::new_with_token(&token);

        for user in vec.iter() {
            let mut map = serde_json::Map::new();
            map.insert("channel_id".to_string(), json!(user.new_channel));

            http.edit_member(340006336659980289, user.id, &map)
                .await
                .expect("That shouldn't have happened")
        }

        Ok(warp::reply::json(&"Moved two users"))
    }
}

mod models {
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    pub type Db = Arc<Mutex<Vec<User>>>;

    pub fn blank_db() -> Db {
        let mut vec = Vec::new();
        vec.push(User {
            id: 73441680702840832,
            original_channel: 340006336659980290,
            new_channel: 633839420545433669,
        });
        vec.push(User {
            id: 122751192307728387,
            original_channel: 340006336659980290,
            new_channel: 633839420545433669,
        });
        Arc::new(Mutex::new(vec))
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct User {
        pub id: u64,
        pub original_channel: u64,
        pub new_channel: u64,
    }
}
