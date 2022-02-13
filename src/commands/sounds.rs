use crate::{Context, Error};
use anyhow::anyhow;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;

/// Add a new sound.
#[poise::command(
    slash_command,
    prefix_command,
    check = "super::meta::ensure_guild_check"
)]
pub(super) async fn add(
    ctx: Context<'_>,
    #[description = "Name of the new sound."] name: String,
    #[description = "Where to download the sound from."] source: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let mut transaction = db.begin().await?;

    let guild_id = ctx.guild_id().unwrap();
    let uploader_id = ctx.author().id;

    // try this insert first to make sure the sound doesn't already exist.
    let sound_id = sqlx::query!(
        "insert into sounds(guild_id, name, source, uploader_id, length) \
            values($1, $2, $3, $4, $5)
            returning id",
        guild_id.0 as i64,
        name,
        source,
        uploader_id.0 as i64,
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

    let guild_dir = &ctx.data().storage_dir.join(guild_id.0.to_string());
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
#[poise::command(
    slash_command,
    prefix_command,
    check = "super::meta::ensure_guild_check"
)]
pub(super) async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;

    let guild_id = ctx.guild_id().unwrap();

    let sounds: Vec<String> = sqlx::query!(
        "select name from sounds \
        where guild_id = $1 and deleted_at is null \
        order by name",
        guild_id.0 as i64
    )
    .map(|record| record.name)
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
pub(super) async fn rename(
    ctx: Context<'_>,
    #[description = "Sound to rename."]
    #[autocomplete = "super::meta::autocomplete_sound_name"]
    old_name: String,
    #[description = "New name for the sound."] new_name: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let mut transaction = db.begin().await?;
    let storage_dir = &ctx.data().storage_dir;

    let guild_id = ctx.guild_id().unwrap();

    let result = sqlx::query!(
        "update sounds set name = $1 \
        where guild_id = $2 and name = $3 and deleted_at is null",
        new_name,
        guild_id.0 as i64,
        old_name
    )
    .execute(&mut transaction)
    .await?;

    if result.rows_affected() != 1 {
        // todo: better errors
        return Err(anyhow!(
            "non-1 rows updated for remove. sound probably doesn't exist."
        ));
    }

    let old_path = storage_dir.join(guild_id.0.to_string()).join(old_name);
    let new_path = storage_dir.join(guild_id.0.to_string()).join(new_name);

    tokio::fs::rename(old_path, new_path).await?;

    transaction.commit().await?;
    ctx.say("✅").await?;
    Ok(())
}

/// Delete a sound.
#[poise::command(slash_command, prefix_command)]
pub(super) async fn remove(
    ctx: Context<'_>,
    #[description = "Sound to remove."]
    #[autocomplete = "super::meta::autocomplete_sound_name"]
    name: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let mut transaction = db.begin().await?;
    let storage_dir = &ctx.data().storage_dir;

    let guild_id = ctx.guild_id().unwrap();

    let result = sqlx::query!(
        "update sounds set deleted_at = current_timestamp \
        where guild_id = $1 and name = $2 and deleted_at is null",
        guild_id.0 as i64,
        name
    )
    .execute(&mut transaction)
    .await?;

    if result.rows_affected() != 1 {
        // todo: better errors
        return Err(anyhow!(
            "non-1 rows updated for remove. sound probably doesn't exist."
        ));
    }

    let file = storage_dir.join(guild_id.0.to_string()).join(name);
    tokio::fs::remove_file(file).await?;

    transaction.commit().await?;
    ctx.say("✅").await?;
    Ok(())
}
