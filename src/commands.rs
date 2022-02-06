use std::process::Stdio;

use crate::{Context, Data, Error};
use poise::{
    serenity_prelude::{Mention, UserId},
    Command,
};

use anyhow::anyhow;
use tokio::io::AsyncWriteExt;

pub const COMMANDS: [fn() -> Command<Data, Error>; 8] =
    [play, random, stop, add, list, rename, remove, history];

async fn autocomplete_sound_name(ctx: Context<'_>, partial: String) -> Vec<String> {
    let db = &ctx.data().db;

    let guild_id = ctx.guild_id().unwrap();

    sqlx::query!(
        "select name from sounds \
        where guild_id = $1 and starts_with(name, $2) and deleted_at is null \
        order by name \
        limit 25",
        guild_id.0 as i64,
        partial
    )
    .map(|record| record.name.to_string())
    .fetch_all(db)
    .await
    .unwrap_or_default()
}

async fn ensure_guild_check(ctx: Context<'_>) -> Result<bool, Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let guild_id = guild_id.0 as i64;
        let db = &ctx.data().db;
        let storage_dir = &ctx.data().storage_dir;

        sqlx::query!(
            "insert into guilds \
            values($1) \
            on conflict do nothing",
            guild_id
        )
        .execute(db)
        .await?;

        let guild_dir = storage_dir.join(guild_id.to_string());

        if !guild_dir.is_dir() {
            tokio::fs::create_dir(&guild_dir)
                .await
                .expect(&format!("Couldn't create guild directory: {:?}", guild_dir));
        }

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
    let mut transaction = db.begin().await?;

    let guild_id = ctx.guild_id().unwrap();
    let guild_id = guild_id.0 as i64;
    let uploader_id = ctx.author().id;
    let uploader_id = uploader_id.0 as i64;

    // try this insert first to make sure the sound doesn't already exist.
    let sound_id = sqlx::query!(
        "insert into sounds(guild_id, name, source, uploader_id, length) \
            values($1, $2, $3, $4, $5)
            returning id",
        guild_id,
        name,
        source,
        uploader_id,
        0 // we calculate this later
    )
    .map(|record| record.id)
    .fetch_one(&mut transaction)
    .await?;

    // let discord know we're not dead.
    let _ = ctx.defer_or_broadcast().await;

    let ytdl_args = [
        "--quiet",
        // "--print-json",
        "-f",
        "webm[abr>0]/bestaudio/best",
        "-R",
        "infinite",
        "--no-playlist",
        "--ignore-config",
        "--no-warnings",
        &source,
        "-o",
        "-",
    ];

    // todo: make this so that it writes directly to file rather than to memory, then to file.
    // download file and write it to `<storage dir>/<guild id>/<sound name>`
    let ytdl_output = tokio::process::Command::new("yt-dlp")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?
        .wait_with_output()
        .await?;

    // let metadata: Value = serde_json::from_slice(&ytdl_output.stderr)?;
    let download = &ytdl_output.stdout;

    let guild_dir = &ctx.data().storage_dir.join(guild_id.to_string());
    let sound_path = guild_dir.join(&name);

    log::debug!("{:#?}", sound_path);
    if sound_path.is_file() {
        panic!("Sound path already exists: {:?}", sound_path);
    }
    let mut file = tokio::fs::File::create(&sound_path).await?;

    file.write_all(download).await?;

    // get the sound's length.
    // using ffprobe here because `metadata["duration"]` is unreliable
    static FFPROBE_ARGS: [&str; 6] = [
        "-v",
        "quiet",
        "-show_entries",
        "format=duration",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
    ];

    let ffprobe_output = tokio::process::Command::new("ffprobe")
        .arg(&sound_path)
        .args(&FFPROBE_ARGS)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    let length = std::str::from_utf8(&ffprobe_output.stdout)?
        .trim()
        .parse::<f64>()?
        * 1000.0;

    sqlx::query!(
        "update sounds set length = $1 \
        where id = $2",
        length as i32,
        sound_id
    )
    .execute(&mut transaction)
    .await?;

    // we're done here.
    transaction.commit().await?;
    ctx.say("✅").await?;
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
    .map(|record| record.name.to_string())
    .fetch_all(db)
    .await?;

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
    ctx: Context<'_>,
    #[description = "Sound to remove."]
    #[autocomplete = "autocomplete_sound_name"]
    name: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;

    let guild_id = ctx.guild_id().unwrap();

    sqlx::query!(
        "update sounds set deleted_at = current_timestamp \
        where guild_id = $1 and name = $2 and deleted_at is null",
        guild_id.0 as i64,
        name
    )
    .execute(db)
    .await?;

    ctx.say("✅").await?;
    Ok(())
}

/// Play a sound in your current voice channel.
#[poise::command(slash_command, prefix_command)]
async fn play(
    ctx: Context<'_>,
    #[description = "Sound to play."]
    #[autocomplete = "autocomplete_sound_name"]
    name: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let mut transaction = db.begin().await?;

    let guild_id = ctx.guild_id().unwrap();
    let player_id = ctx.author().id;

    let sound = sqlx::query!(
        "select id, source from sounds \
        where guild_id = $1 and name = $2 and deleted_at is null",
        guild_id.0 as i64,
        name,
    )
    .fetch_one(&mut transaction)
    .await?;

    sqlx::query!(
        "insert into playbacks(sound_id, player_id) \
        values ($1, $2)",
        sound.id,
        player_id.0 as i64,
    )
    .execute(&mut transaction)
    .await?;

    let manager = songbird::get(ctx.discord())
        .await
        .expect("Expected songbird client in data at initialization.")
        .clone();

    let handler_lock = if let Some(handler) = manager.get(guild_id.0) {
        handler
    } else {
        let channel = ctx
            .guild()
            .ok_or(anyhow!("Not in a guild."))?
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id)
            .ok_or(anyhow!("Not in a voice channel."))?;

        let (handler, result) = manager.join(guild_id, channel).await;

        result?;

        handler
    };

    let mut handler = handler_lock.lock().await;

    let source = songbird::ytdl(sound.source).await?;

    handler.play_source(source);

    transaction.commit().await?;
    ctx.say("✅").await?;
    Ok(())
}

/// Play a random sound in your current voice channel.
#[poise::command(slash_command, prefix_command)]
async fn random(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("✅").await?;
    Ok(())
}

/// Stop the currently playing sound.
#[poise::command(slash_command, prefix_command)]
async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    // let db = &ctx.data().db;

    // let guild_id = ctx.guild_id().unwrap();
    // let stopper_id = ctx.author().id;

    // sqlx::query!(
    //     "with sound_id as ( \
    //         select id from sounds  \
    //         where guild_id = $1 and name = $2 and deleted_at is null \
    //     ) \
    //     update playbacks set stopper_id = $1, stopped_at = current_timestamp \
    //     where sound_id = $2 and stopped_at is null",
    //     guild_id.0 as i64,

    // )
    // .execute(db)
    // .await?;

    ctx.say("✅").await?;
    Ok(())
}

/// Show sound play history.
#[poise::command(slash_command, prefix_command)]
async fn history(
    ctx: Context<'_>,
    #[description = "Optional sound to get the playback history of."]
    #[autocomplete = "autocomplete_sound_name"]
    name: Option<String>,
) -> Result<(), Error> {
    let db = &ctx.data().db;

    let guild_id = ctx.guild_id().unwrap();

    // language=PostgreSQL
    let history: Vec<String> = sqlx::query!(
        "select playbacks.*, sounds.name from playbacks \
        inner join sounds on sounds.id = playbacks.sound_id \
        where sounds.guild_id = $1 and ($2::text is null or sounds.name = $2) \
        order by playbacks.created_at desc",
        guild_id.0 as i64,
        name
    )
    .map(|record| {
        let name = record.name.to_string();
        let player: UserId = (record.player_id as u64).into();
        let stopper: Option<UserId> = record.stopper_id.map(|id| (id as u64).into());

        let created = record.created_at;

        if let Some(stopper) = stopper {
            let stopped = record.stopped_at.unwrap();
            let duration = stopped - created;
            let seconds = duration.num_milliseconds() as f64 / 1000_f64;

            format!(
                "{} by {}; stopped by {} after {} seconds",
                name,
                Mention::from(player),
                Mention::from(stopper),
                seconds
            )
        } else {
            format!("{} by {}", name, Mention::from(player))
        }
    })
    .fetch_all(db)
    .await?;

    let msg = if history.is_empty() {
        "There's nothing here. Play a sound with /play or add a new sound with /add.".to_owned()
    } else {
        history.join("\n")
    };

    ctx.say(msg).await?;
    Ok(())
}
