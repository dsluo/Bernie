use poise::Command;
use crate::{Data, Error, Context};


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


/// Add a new sound.
#[poise::command(slash_command, prefix_command)]
async fn add(ctx: Context<'_>) -> Result<(), Error> {
    todo!()
}

/// List all sounds on this server.
#[poise::command(slash_command, prefix_command)]
async fn list(ctx: Context<'_>) -> Result<(), Error> {
    todo!()
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