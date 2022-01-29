use async_trait::async_trait;
use chrono::Duration;
use serenity::model::id::{GuildId, UserId};
use url::Url;

use super::types::{Guild, Playback, Sound};

#[async_trait]
pub trait BackendProvider: Sync + Send {
    async fn setup() -> Self;

    // guilds
    async fn ensure_guild(&self, guild_id: GuildId) -> anyhow::Result<Guild>;

    // sounds
    async fn add(&self, guild_id: GuildId, uploader_id: UserId, name: &str, source: Url, length: Duration) -> anyhow::Result<Sound>;
    async fn list(&self, guild_id: GuildId) -> anyhow::Result<Vec<Sound>>;
    async fn rename(&self, guild_id: GuildId, old_name: &str, new_name: &str) -> anyhow::Result<Sound>;
    async fn remove(&self, guild_id: GuildId, name: &str) -> anyhow::Result<Sound>;

    async fn autocomplete(&self, guild_id: GuildId, starts_with: &str) -> anyhow::Result<Vec<String>>;

    // playbacks
    async fn play(&self, guild_id: GuildId, player_id: UserId, name: &str) -> anyhow::Result<Playback>;
    async fn random(&self, guild_id: GuildId, player_id: UserId) -> anyhow::Result<Playback>;
    async fn stop(&self, guild_id: GuildId, stopper_id: UserId, name: &str) -> anyhow::Result<Playback>;

    async fn history(&self, guild_id: GuildId, name: Option<&str>) -> anyhow::Result<Vec<Playback>>;
}