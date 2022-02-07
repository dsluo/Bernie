use crate::{Context, Error};
use anyhow::anyhow;
use poise::serenity_prelude::{Mention, UserId};

/// Play a sound in your current voice channel.
#[poise::command(slash_command, prefix_command)]
pub(super) async fn play(
    ctx: Context<'_>,
    #[description = "Sound to play."]
    #[autocomplete = "super::meta::autocomplete_sound_name"]
    name: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let storage_dir = &ctx.data().storage_dir;
    let mut transaction = db.begin().await?;

    let guild_id = ctx.guild_id().unwrap();
    let player_id = ctx.author().id;

    let _playback_id = sqlx::query!(
        "insert into playbacks(sound_id, player_id) \
        select id, $1 from sounds where guild_id = $2 and name = $3 and deleted_at is null
        returning playbacks.id",
        player_id.0 as i64,
        guild_id.0 as i64,
        &name
    )
    .map(|record| record.id)
    .fetch_one(&mut transaction)
    .await?;

    let manager = songbird::get(ctx.discord())
        .await
        .expect("Expected songbird client in data at initialization.")
        .clone();

    // todo: change this to manager.get_or_insert and always attempt to join author's voice channel.
    let call_lock = if let Some(call) = manager.get(guild_id.0) {
        call
    } else {
        let channel = ctx
            .guild()
            .ok_or(anyhow!("Not in a guild."))?
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id)
            .ok_or(anyhow!("Not in a voice channel."))?;

        let (call, result) = manager.join(guild_id, channel).await;

        result?;

        call
    };

    let mut call = call_lock.lock().await;

    let file = storage_dir.join(guild_id.0.to_string()).join(name);
    let source = songbird::ffmpeg(file).await?;

    call.play_source(source);

    transaction.commit().await?;
    ctx.say("✅").await?;
    Ok(())
}

/// Play a random sound in your current voice channel.
#[poise::command(slash_command, prefix_command)]
pub(super) async fn random(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("✅").await?;
    Ok(())
}

/// Stop the currently playing sound.
#[poise::command(slash_command, prefix_command)]
pub(super) async fn stop(ctx: Context<'_>) -> Result<(), Error> {
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
pub(super) async fn history(
    ctx: Context<'_>,
    #[description = "Optional sound to get the playback history of."]
    #[autocomplete = "super::meta::autocomplete_sound_name"]
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
