use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use poise::serenity_prelude::{GuildId, Mention, UserId};
use songbird::tracks::TrackHandle;
use tokio::sync::Mutex;

use crate::{Context, Error};

type PlaybackId = i32;

type AtomicHashMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
pub struct TrackManager {
    handles: AtomicHashMap<GuildId, Vec<(PlaybackId, TrackHandle)>>,
}

impl TrackManager {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_playback(
        &self,
        guild_id: GuildId,
        playback_id: PlaybackId,
        track_handle: TrackHandle,
    ) {
        let mut lock = self.handles.lock().await;

        let handlers = lock.entry(guild_id).or_default();
        handlers.push((playback_id, track_handle));
    }

    pub async fn stop_playback(&self, guild_id: &GuildId) -> Result<Vec<PlaybackId>, Error> {
        let mut lock = self.handles.lock().await;
        let handlers = lock.remove(guild_id).unwrap_or_default();

        let mut stopped = vec![];
        for (playback_id, handler) in handlers.iter() {
            handler.stop()?;
            stopped.push(playback_id.to_owned());
        }

        Ok(stopped)
    }
}

struct PlaybackFinisher;

#[async_trait]
impl songbird::EventHandler for PlaybackFinisher {
    async fn act(&self, ctx: &songbird::EventContext<'_>) -> Option<songbird::Event> {
        log::debug!("{:#?}", ctx);
        // todo: in what cases does this slice contain >1 `(&TrackState, &TrackHandle)`?
        if let songbird::EventContext::Track(tracks) = ctx {
            log::debug!("{:#?}", tracks);
        }

        None
    }
}

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

    let playback_id = sqlx::query!(
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

    let (track, track_handle) = songbird::create_player(source);

    use songbird::{Event, TrackEvent};
    track_handle.add_event(Event::Track(TrackEvent::End), PlaybackFinisher)?;

    ctx.data()
        .track_manager
        .register_playback(guild_id, playback_id, track_handle)
        .await;

    call.play(track);

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
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().unwrap();
    let playback_manager = &ctx.data().track_manager;

    let stopped = playback_manager.stop_playback(&guild_id).await?;

    let stopper_id = ctx.author().id;

    sqlx::query!(
        "update playbacks set stopper_id = $1, stopped_at = current_timestamp \
        from (select unnest($2::int[]) as id) as stopped \
        where playbacks.id = stopped.id",
        stopper_id.0 as i64,
        &stopped
    )
    .execute(db)
    .await?;

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
