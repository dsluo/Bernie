use chrono::Utc;
use serenity::model::id::{GuildId, UserId};

pub struct Guild {
    pub id: GuildId,
}

pub struct Sound {
    pub id: i32,
    pub guild_id: GuildId,
    pub name: String,
    pub source: String,
    pub uploader_id: UserId,
    pub length: chrono::Duration,
}

pub struct Playback {
    pub id: i32,
    pub started_at: chrono::DateTime<Utc>,
    pub stopped_at: Option<chrono::DateTime<Utc>>,
    pub sound_id: i32,
    pub player_id: UserId,
    pub stopper_id: Option<UserId>,
}
