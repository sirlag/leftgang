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

    let id = *msg.mentions.first().expect("oops").id.as_u64();

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
            id,
            original_channel,
            new_channel,
        })
    }

    msg.reply(
        ctx,
        format!("Added entry to move <@!{}> to <#{}>", id, new_channel),
    )
    .await?;

    Ok(())
}
