use serenity::async_trait;
use serenity::client::Client;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::prelude::*;
use std::{env, net::SocketAddr};

use crate::models::DbKey;
use commands::*;

#[group]
#[commands(ping, log_data)]
struct General;

struct DiscordHandler;

#[async_trait]
impl EventHandler for DiscordHandler {}

#[tokio::main]
async fn main() {
    println!("Starting up left-gang service");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let db = models::blank_db_2();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let server = warp::serve(filters::movers(token.clone(), db.clone())).run(addr);

    let server_task = tokio::spawn(server);
    println!("Listening on http://{}", addr);

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    let mut client = Client::new(token)
        .event_handler(DiscordHandler)
        .framework(framework)
        .await
        .expect("Error Creating Discord Client");

    {
        let mut data = client.data.write().await;
        data.insert::<DbKey>(db);
    }

    if let Err(why) = client.start().await {
        eprintln!("An error occured while running the client: {}", why)
    }

    match server_task.await {
        Ok(_) => println!("Closing Server"),
        Err(why) => eprintln!("{}", why),
    };
}

mod filters {
    use crate::handlers;
    use crate::models::Db;
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
        warp::path!("api" / "move" / "original")
            .and(warp::get())
            .and(warp::any().map(move || token.clone()))
            .and(with_db(db))
            .and_then(handlers::move_users_to_original)
    }

    pub fn move_users_to_new(
        token: String,
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "move")
            .and(warp::get())
            .and(warp::any().map(move || token.clone()))
            .and(with_db(db))
            .and_then(handlers::move_users_to_group)
    }

    pub fn hello() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "hello" / String).map(|name| format!("Hello, {}!", name))
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

    pub async fn move_users_to_group(
        token: String,
        db: Db,
    ) -> Result<impl warp::Reply, Infallible> {
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
}

mod models {
    use serde::{Deserialize, Serialize};
    use serenity::prelude::TypeMapKey;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    pub type Db = Arc<RwLock<Vec<User>>>;
    pub struct DbKey;
    impl TypeMapKey for DbKey {
        type Value = Db;
    }

    pub fn blank_db_2() -> Db {
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
        Arc::new(RwLock::new(vec))
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct User {
        pub id: u64,
        pub original_channel: u64,
        pub new_channel: u64,
    }
}

mod commands {
    use crate::models::DbKey;
    use serenity::client::Context;
    use serenity::framework::standard::macros::command;
    use serenity::framework::standard::CommandResult;
    use serenity::model::channel::Message;

    #[command]
    async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
        msg.reply(ctx, "Pong!").await?;

        Ok(())
    }

    #[command]
    async fn log_data(ctx: &Context, msg: &Message) -> CommandResult {
        let thingy: Vec<u64> = {
            let data_read = ctx.data.read().await;
            let db_lock = data_read
                .get::<DbKey>()
                .expect("Expected a database reference in TypeMap")
                .clone();
            let db = db_lock.read().await;
            db.iter().map(|it| it.new_channel).collect()
        };

        let new_channel = format!("The new channel will be <#{}>", thingy[0]);

        msg.reply(ctx, new_channel).await?;
        Ok(())
    }
}
