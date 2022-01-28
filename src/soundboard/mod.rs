use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use url::Url;

type Snowflake = u64;

pub struct Guild {
    pub id: Snowflake,
}

pub struct Sound {
    pub id: i32,
    pub guild_id: Snowflake,
    pub name: String,
    pub source: String,
    pub uploader_id: Snowflake,
    pub length: chrono::Duration,
}

pub struct Playback {
    pub id: i32,
    pub started_at: chrono::DateTime<Utc>,
    pub stopped_at: Option<chrono::DateTime<Utc>>,
    pub sound_id: i32,
    pub player_id: Snowflake,
    pub stopper_id: Option<Snowflake>
}

#[async_trait]
pub trait SoundboardBackend: Sync + Send {
    // guilds
    async fn ensure_guild(&self, guild_id: Snowflake) -> Result<Guild>;

    // sounds
    async fn add(&self, guild_id: Snowflake, uploader_id: Snowflake, name: &str, source: Url) -> Result<Sound>;
    async fn list(&self, guild_id: Snowflake) -> Result<Vec<Sound>>;
    async fn rename(&self, guild_id: Snowflake, old_name: &str, new_name: &str) -> Result<Sound>;
    async fn remove(&self, guild_id: Snowflake, name: &str) -> Result<Sound>;

    async fn autocomplete(&self, guild_id: Snowflake, starts_with: &str) -> Result<Vec<String>>;

    // playbacks
    async fn play(&self, guild_id: Snowflake, player_id: Snowflake, name: &str) -> Result<Playback>;
    async fn random(&self, guild_id: Snowflake, player_id: Snowflake) -> Result<Playback>;
    async fn stop(&self, guild_id: Snowflake, stopper_id: Snowflake, name: &str) -> Result<Playback>;

    async fn history(&self, guild_id: Snowflake, name: Option<&str>) -> Result<Vec<Playback>>;
}

#[cfg(feature = "database")]
pub mod database;