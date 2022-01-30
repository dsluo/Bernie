use poise::Command;

use crate::{Context, Data, Error};

pub const COMMANDS: [fn() -> Command<Data, Error>; 8] =
    [play, random, stop, add, list, rename, remove, history];

async fn autocomplete_sound_name(
    _ctx: Context<'_>,
    _partial: String,
) -> impl Iterator<Item = String> + '_ {
    // let results = if let Some(guild_id) = ctx.guild_id() {
    //     let db = &ctx.data().db;

    //     db.autocomplete(guild_id, &partial)
    //         .await?
    // } else {
    vec![String::from("asdf")].into_iter()
    // };

    // log::info!("{:?}", results);

    // results
}

async fn ensure_guild_check(ctx: Context<'_>) -> Result<bool, Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let db = &ctx.data().db;

        sqlx::query!(
            "insert into guilds \
            values($1) \
            on conflict do nothing",
            guild_id.0 as i64
        )
        .execute(db)
        .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Add a new sound.
#[poise::command(slash_command, prefix_command, check = "ensure_guild_check")]
async fn add(
    ctx: Context<'_>,
    #[description = "Name of the new sound."] name: String,
    #[description = "Where to download the sound from."] source: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;

    let guild_id = ctx.guild_id().unwrap();
    let uploader_id = ctx.author().id;
    let url = url::Url::parse(&source)?;
    let length = chrono::Duration::zero();

    sqlx::query!(
        "insert into sounds(guild_id, name, source, uploader_id, length) \
            values($1, $2, $3, $4, $5)",
        guild_id.0 as i64,
        name,
        url.to_string(),
        uploader_id.0 as i64,
        length.num_milliseconds() as i32
    );

    // todo: actually download or something

    Ok(())
}

/// List all sounds on this server.
#[poise::command(slash_command, prefix_command, check = "ensure_guild_check")]
async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;

    let guild_id = ctx.guild_id().unwrap();

    let sounds: Vec<String> = sqlx::query!(
        "select name from sounds \
        where guild_id = $1 and deleted_at is null \
        order by name",
        guild_id.0 as i64
    )
    .fetch_all(db)
    .await?
    .iter()
    .map(|record| record.name.to_string())
    .collect();

    let msg = if sounds.is_empty() {
        "There's nothing here. Try adding a sound using /add.".to_owned()
    } else {
        sounds.join("\n")
    };
    ctx.say(msg).await?;

    Ok(())
}

/// Rename a sound.
#[poise::command(slash_command, prefix_command)]
async fn rename(
    ctx: Context<'_>,
    #[description = "Sound to rename."]
    #[autocomplete = "autocomplete_sound_name"]
    old_name: String,
    #[description = "New name for the sound."] new_name: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;

    let guild_id = ctx.guild_id().unwrap();

    // let _sound = db.rename(guild_id, &old_name, &new_name).await?;

    sqlx::query!(
        "update sounds set name = $1 \
        where guild_id = $2 and name = $3 and deleted_at is null",
        new_name,
        guild_id.0 as i64,
        old_name
    )
    .execute(db)
    .await?;

    ctx.say("✅").await?;

    Ok(())
}

/// Delete a sound.
#[poise::command(slash_command, prefix_command)]
async fn remove(
    _ctx: Context<'_>,
    #[description = "Sound to remove."]
    #[autocomplete = "autocomplete_sound_name"]
    _name: String,
) -> Result<(), Error> {
    // "update sounds set deleted_at = current_timestamp \
    // where guild_id = $1 and name = $2 and deleted_at is null \
    todo!()
}

/// Play a sound in your current voice channel.
#[poise::command(slash_command, prefix_command)]
async fn play(
    ctx: Context<'_>,
    #[description = "Sound to play."]
    #[autocomplete = "autocomplete_sound_name"]
    name: String,
) -> Result<(), Error> {
    // todo!()
    // "insert into playbacks(sound_id, player_id) \
    // values($1, $2) \
    ctx.say(name).await?;

    Ok(())
}

/// Play a random sound in your current voice channel.
#[poise::command(slash_command, prefix_command)]
async fn random(_ctx: Context<'_>) -> Result<(), Error> {
    todo!()
}

/// Stop the currently playing sound.
#[poise::command(slash_command, prefix_command)]
async fn stop(_ctx: Context<'_>) -> Result<(), Error> {
    // "update playbacks set stopper_id = $1, stopped_at = current_timestamp \
    // where sound_id = $2 and stopped_at is null \
    todo!()
}

/// Show sound play history.
#[poise::command(slash_command, prefix_command)]
async fn history(
    _ctx: Context<'_>,
    #[description = "Optional sound to get the playback history of."]
    #[autocomplete = "autocomplete_sound_name"]
    _name: Option<String>,
) -> Result<(), Error> {
    // "select playbacks.*, sounds.guild_id from playbacks \
    // inner join sounds on sounds.id = playbacks.sound_id \
    // where guild_id = $1 and sound_id = $2 \
    // order by playbacks.created_at desc"

    // "select playbacks.*, sounds.guild_id from playbacks \
    // inner join sounds on sounds.id = playbacks.sound_id \
    // where guild_id = $1 \
    // order by playbacks.created_at desc"
    todo!()
}
