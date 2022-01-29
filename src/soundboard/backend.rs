use async_trait::async_trait;
use url::Url;

use super::types::{Guild, Playback, Snowflake, Sound};

#[async_trait]
pub trait BackendProvider: Sync + Send {
    async fn setup() -> Self;

    // guilds
    async fn ensure_guild(&self, guild_id: Snowflake) -> anyhow::Result<Guild>;

    // sounds
    async fn add(&self, guild_id: Snowflake, uploader_id: Snowflake, name: &str, source: Url) -> anyhow::Result<Sound>;
    async fn list(&self, guild_id: Snowflake) -> anyhow::Result<Vec<Sound>>;
    async fn rename(&self, guild_id: Snowflake, old_name: &str, new_name: &str) -> anyhow::Result<Sound>;
    async fn remove(&self, guild_id: Snowflake, name: &str) -> anyhow::Result<Sound>;

    async fn autocomplete(&self, guild_id: Snowflake, starts_with: &str) -> anyhow::Result<Vec<String>>;

    // playbacks
    async fn play(&self, guild_id: Snowflake, player_id: Snowflake, name: &str) -> anyhow::Result<Playback>;
    async fn random(&self, guild_id: Snowflake, player_id: Snowflake) -> anyhow::Result<Playback>;
    async fn stop(&self, guild_id: Snowflake, stopper_id: Snowflake, name: &str) -> anyhow::Result<Playback>;

    async fn history(&self, guild_id: Snowflake, name: Option<&str>) -> anyhow::Result<Vec<Playback>>;
}