use crate::models::{DbKey, Member, MoveGroup};
use crate::util::parse_channel_id;
use log::info;
use serenity::client::Context;
use serenity::collector::{ReactionAction, ReactionCollectorBuilder};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::futures::stream::StreamExt;
use serenity::model::channel::Message;

use serenity::model::channel::ReactionType::Unicode;
use uuid::Uuid;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn register_server(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Registering server");
    let guild_id = msg.guild_id.expect("Expected Guild id").to_string();
    {
        let type_map = ctx.data.read().await;
        let db_lock = type_map
            .get::<DbKey>()
            .expect("Expected a database reference in TypeMap")
            .clone();

        match sqlx::query!("INSERT INTO guilds VALUES($1) RETURNING id", guild_id)
            .fetch_one(&db_lock)
            .await
        {
            Ok(..) => {
                println!("Registered server {}", guild_id);
                msg.reply(ctx, "Registered to server").await?;
            }
            Err(error) => {
                println!("Unable to register server {}, {:?}", guild_id, error);
                msg.reply(ctx, "Unable to register to server").await?;
            }
        }
    }
    Ok(())
}

#[command]
async fn register_group(ctx: &Context, msg: &Message) -> CommandResult {
    // Ok(())

    let args: Vec<&str> = msg.content.split_whitespace().collect();
    if args.len() < 4 {
        info!("Called with less than three parameters");
        msg.reply(ctx, "Requires three parameters").await?;
        return Ok(());
    }

    let move_group = MoveGroup {
        id: Uuid::new_v4(),
        guild_id: msg
            .guild_id
            .expect("A message must be sent in a guild")
            .to_string(),
        name: args[1].to_string(),
        home_channel: args[2].to_string(),
        group_channel: args[3].to_string(),
    };

    insert_move_group(ctx, move_group.clone()).await;

    let react_msg = msg
        .reply(ctx, "Added group {} to move users between {} (home) and {}. \n React to this message with ✔ to be added to this group")
        .await
        .unwrap();

    react_msg.react(&ctx, Unicode("✔".parse().unwrap())).await?;

    let collector = ReactionCollectorBuilder::new(&ctx)
        .message_id(react_msg.id)
        .removed(true)
        .await;

    let mg = &(move_group.clone());
    collector
        .for_each(|reaction| async move {
            info!("{:#?}", reaction);
            match reaction.as_ref() {
                ReactionAction::Added(react) => {
                    let _ = if let "✔" = react.emoji.as_data().as_str() {
                        let id = react.user_id.expect("Expected a user id").to_string();
                        info!("Added {} to {}", id, mg.id);
                        insert_member(ctx, id, mg.id).await;
                    };
                }
                ReactionAction::Removed(react) => {
                    let _ = if let "✔" = react.emoji.as_data().as_str() {
                        let id = react.user_id.expect("Expected a user id").to_string();
                        info!("Removing {} from {}", id, mg.id);
                        delete_member(ctx, id, mg.id).await
                    };
                }
            }
        })
        .await;

    Ok(())
}

async fn delete_member(ctx: &Context, user_id: String, mg_id: Uuid) {
    let type_map = ctx.data.read().await;
    let db_lock = type_map
        .get::<DbKey>()
        .expect("Expected a database reference in TypeMap")
        .clone();

    info!("Deleting entry");
    let _ = sqlx::query!(
        "DELETE FROM members WHERE user_id = $1 and move_group_id = $2 RETURNING id",
        user_id,
        mg_id
    )
    .fetch_one(&db_lock)
    .await;
}

async fn insert_move_group(ctx: &Context, move_group: MoveGroup) {
    {
        let type_map = ctx.data.read().await;
        let db_lock = type_map
            .get::<DbKey>()
            .expect("Expected a database reference in TypeMap")
            .clone();

        let _ = sqlx::query!(
            "INSERT INTO move_groups VALUES($1, $2, $3, $4, $5) RETURNING id",
            move_group.id,
            move_group.guild_id,
            move_group.name,
            move_group.home_channel,
            move_group.group_channel
        )
        .fetch_one(&db_lock)
        .await;
    }
}

async fn insert_member(ctx: &Context, id: String, group_id: Uuid) -> Result<Member, sqlx::Error> {
    let type_map = ctx.data.read().await;
    let db_lock = type_map
        .get::<DbKey>()
        .expect("Expected a database reference in TypeMap")
        .clone();

    sqlx::query_as!(
        Member,
        "INSERT INTO members VALUES($1, $2, $3) RETURNING id, user_id, move_group_id",
        Uuid::new_v4(),
        id,
        group_id,
    )
    .fetch_one(&db_lock)
    .await
}

// #[command]
// async fn move(ctx: &Context, msg: &Message) -> CommandResult {
//
// }
