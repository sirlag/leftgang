use serenity::async_trait;
use serenity::client::Client;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::prelude::*;
use std::{env, net::SocketAddr};

use crate::models::DbKey;
use commands::*;

#[group]
#[commands(ping, log_data, add)]
struct General;

struct DiscordHandler;

#[async_trait]
impl EventHandler for DiscordHandler {}

#[tokio::main]
async fn main() {
    println!("Starting up left-gang service");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let db = models::blank_db();
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
        eprintln!("An error occurred while running the client: {}", why)
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

    pub fn blank_db() -> Db {
        Arc::new(RwLock::new(Vec::new()))
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct User {
        pub id: u64,
        pub original_channel: u64,
        pub new_channel: u64,
    }
}

mod commands {
    use crate::models::{DbKey, User};
    use crate::util::parse_channel_id;
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

    #[command]
    async fn add(ctx: &Context, msg: &Message) -> CommandResult {
        println!("{:#?}", msg);

        let args: Vec<&str> = msg.content.split_whitespace().collect();
        if args.len() < 4 {
            msg.reply(ctx, "Requires three parameters").await?;
            return Ok(());
        }

        let original_channel =
            parse_channel_id(args[2]).expect("Unable to parse original channel string");

        let new_channel = parse_channel_id(args[3]).expect("Unable to parse new channel string");

        {
            let data_write = ctx.data.write().await;
            let db_lock = data_write
                .get::<DbKey>()
                .expect("Expected a database reference in TypeMap")
                .clone();
            let mut db = db_lock.write().await;
            db.push(User {
                id: *msg.mentions.first().expect("oops").id.as_u64(),
                original_channel,
                new_channel,
            })
        }

        Ok(())
    }
}

mod util {
    use std::num::ParseIntError;

    pub fn parse_channel_id(str: &str) -> Result<u64, ParseIntError> {
        str.parse::<u64>()
            .or_else(|_| str.replace("<#", "").replace(">", "").parse())
    }
}
