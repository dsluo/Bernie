use std::fmt::format;
use poise::Command;

use crate::{BackendProvider, Context, Data, Error};

pub const COMMANDS: [fn() -> Command<Data, Error>; 8] = [
    play,
    random,
    stop,
    add,
    list,
    rename,
    remove,
    history
];

async fn ensure_guild_check(ctx: Context<'_>) -> Result<bool, Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let backend = &ctx.data().backend;
        let _guild = backend.ensure_guild(guild_id).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}


/// Add a new sound.
#[poise::command(slash_command, prefix_command, check = "ensure_guild_check")]
async fn add(
    ctx: Context<'_>,
    #[description = "Name of the new sound."]
    name: String,
    #[description = "Where to download the sound from."]
    source: String,
) -> Result<(), Error> {
    let backend = &ctx.data().backend;

    let guild_id = ctx.guild_id().unwrap();
    let url = url::Url::parse(&source)?;
    let length = chrono::Duration::zero();

    let sound = backend.add(guild_id, ctx.author().id, &name, url, length).await?;

    // todo: actually download or something

    ctx.say(format!("{:?}", sound)).await?;

    Ok(())
}

/// List all sounds on this server.
#[poise::command(slash_command, prefix_command, check = "ensure_guild_check")]
async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let backend = &ctx.data().backend;

    let guild_id = ctx.guild_id().unwrap();

    let sounds = backend.list(guild_id)
        .await?
        .iter()
        .map(|sound| String::from(sound.name.as_str()))
        .collect::<Vec<String>>();

    let msg = if sounds.is_empty() {
        "There's nothing here. Try adding a sound using /add.".to_owned()
    }  else {
        sounds.join("\n")
    };
    ctx.say(msg).await?;

    Ok(())
}

/// Rename a sound.
#[poise::command(slash_command, prefix_command)]
async fn rename(ctx: Context<'_>) -> Result<(), Error> {
    todo!()
}

/// Delete a sound.
#[poise::command(slash_command, prefix_command)]
async fn remove(ctx: Context<'_>) -> Result<(), Error> {
    todo!()
}

/// Play a sound in your current voice channel.
#[poise::command(slash_command, prefix_command)]
async fn play(ctx: Context<'_>) -> Result<(), Error> {
    todo!()
}

/// Play a random sound in your current voice channel.
#[poise::command(slash_command, prefix_command)]
async fn random(ctx: Context<'_>) -> Result<(), Error> {
    todo!()
}

/// Stop the currently playing sound.
#[poise::command(slash_command, prefix_command)]
async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    todo!()
}

/// Show sound play history.
#[poise::command(slash_command, prefix_command)]
async fn history(ctx: Context<'_>) -> Result<(), Error> {
    todo!()
}