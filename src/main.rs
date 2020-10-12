mod commands;
mod filters;
mod handlers;
mod models;
mod util;

use serenity::async_trait;
use serenity::client::Client;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::prelude::*;
use std::{env, net::SocketAddr};

use crate::models::DbKey;
use commands::*;

#[group]
#[commands(ping, add)]
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
