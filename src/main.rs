mod commands;
mod filters;
mod handlers;
mod models;
mod util;

use dotenv::dotenv;

use serenity::async_trait;
use serenity::client::Client;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::prelude::*;
use std::{env, net::SocketAddr};

use self::models::*;
use crate::models::DbKey;
use commands::*;
use sqlx::postgres::PgPoolOptions;

#[group]
#[commands(ping, register_server, register_group)]
struct General;

struct DiscordHandler;

#[async_trait]
impl EventHandler for DiscordHandler {}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    env_logger::init();

    println!("Starting up left-gang service");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await?;

    // let guild = Guild {
    //     id: "514950774552395826".parse().expect("It"),
    // };
    //
    // sqlx::query!("INSERT INTO guilds VALUES ($1) RETURNING id", guild.id)
    //     .fetch_one(&pool)
    //     .await?;
    //
    // let results = sqlx::query_as!(
    //     Guild,
    //     "
    //     SELECT *
    //     FROM guilds
    // "
    // )
    // .fetch_all(&pool)
    // .await?;
    //
    // println!("{:#?}", results);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let db = models::blank_db();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let server = warp::serve(filters::movers(token.clone(), pool.clone())).run(addr);

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
        data.insert::<DbKey>(pool);
    }

    if let Err(why) = client.start().await {
        eprintln!("An error occurred while running the client: {}", why)
    }

    match server_task.await {
        Ok(_) => println!("Closing Server"),
        Err(why) => eprintln!("{}", why),
    };

    Ok(())
}
