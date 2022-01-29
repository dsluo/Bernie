use anyhow::Result;
use async_trait::async_trait;
use serenity::model::id::{GuildId, UserId};
use sqlx::{
    Error,
    FromRow,
    PgPool,
    Row,
    postgres::PgRow
};
use url::Url;

use crate::soundboard::{
    backend::BackendProvider,
    types::{Guild, Playback, Sound},
};

pub struct DatabaseBackend {
    pool: PgPool,
}

impl DatabaseBackend {
    pub async fn new(uri: &str) -> Result<Self> {
        Ok(DatabaseBackend {
            pool: PgPool::connect(uri).await?
        })
    }

    async fn get_sound_id(&self, guild_id: GuildId, name: &str) -> Result<i32> {
        Ok(sqlx::query(
            "select id from sounds \
            where guild_id = $1 and name = $2 and deleted_at is null")
            .bind(guild_id.0 as i64)
            .bind(name)
            .map(|row: PgRow| row.get(0))
            .fetch_one(&self.pool)
            .await?)
    }
}

#[async_trait]
impl BackendProvider for DatabaseBackend {
    async fn setup() -> Self {
        let uri = std::env::var("DATABASE_URL").expect("Expected DATABASE_URL in environment.");
        DatabaseBackend::new(&uri).await.expect("Couldn't connect to backend.")
    }

    async fn ensure_guild(&self, guild_id: GuildId) -> Result<Guild> {
        Ok(sqlx::query_as::<_, Guild>(
            "insert into guilds \
            values($1) \
            on conflict(id) \
                do update set updated_at = current_timestamp \
            returning *")
            .bind(guild_id.0 as i64)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn add(&self, guild_id: GuildId, uploader_id: UserId, name: &str, source: Url) -> Result<Sound> {
        let length = chrono::Duration::zero();

        Ok(sqlx::query_as::<_, Sound>(
            "insert into sounds(guild_id, name, source, uploader_id, length) \
            values($1, $2, $3, $4, $5) \
            returning *")
            .bind(guild_id.0 as i64)
            .bind(name)
            .bind(source.as_str())
            .bind(uploader_id.0 as i64)
            .bind(length)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn list(&self, guild_id: GuildId) -> Result<Vec<Sound>> {
        Ok(sqlx::query_as::<_, Sound>(
            "select * from sounds \
            where guild_id = $1 and deleted_at is null \
            order by name")
            .bind(guild_id.0 as i64)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn rename(&self, guild_id: GuildId, old_name: &str, new_name: &str) -> Result<Sound> {
        Ok(sqlx::query_as::<_, Sound>(
            "update sounds set name = $1 \
            where guild_id = $2 and name = $3 and deleted_at is null \
            returning *")
            .bind(new_name)
            .bind(guild_id.0 as i64)
            .bind(old_name)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn remove(&self, guild_id: GuildId, name: &str) -> Result<Sound> {
        Ok(sqlx::query_as::<_, Sound>(
            "update sounds set deleted_at = current_timestamp \
            where guild_id = $1 and name = $2 and deleted_at is null \
            returning *")
            .bind(guild_id.0 as i64)
            .bind(name)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn autocomplete(&self, guild_id: GuildId, starts_with: &str) -> Result<Vec<String>> {
        Ok(sqlx::query(
            "select name from sounds \
            where guild_id = $1 and starts_with(name, $2) and deleted_at is null \
            limit 25 \
            order by name")
            .bind(guild_id.0 as i64)
            .bind(starts_with)
            .map(|row: PgRow| row.get("name"))
            .fetch_all(&self.pool)
            .await?)
    }

    async fn play(&self, guild_id: GuildId, player_id: UserId, name: &str) -> Result<Playback> {
        let sound_id: i32 = self.get_sound_id(guild_id, name).await?;

        Ok(sqlx::query_as::<_, Playback>(
            "insert into playbacks(sound_id, player_id) \
            values($1, $2) \
            returning *")
            .bind(sound_id)
            .bind(player_id.0 as i64)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn random(&self, guild_id: GuildId, player_id: UserId) -> Result<Playback> {
        todo!()
    }

    async fn stop(&self, guild_id: GuildId, stopper_id: UserId, name: &str) -> Result<Playback> {
        let sound = self.get_sound_id(guild_id, name).await?;

        Ok(sqlx::query_as::<_, Playback>(
            "update playbacks set stopper_id = $1, stopped_at = current_timestamp \
            where sound_id = $2 and stopped_at is null \
            returning *")
            .bind(stopper_id.0 as i64)
            .bind(sound)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn history(&self, guild_id: GuildId, name: Option<&str>) -> Result<Vec<Playback>> {
        let query = if let Some(name) = name {
            let sound_id = self.get_sound_id(guild_id, name).await?;
            sqlx::query_as::<_, Playback>(
                "select playbacks.*, sounds.guild_id from playbacks \
                inner join sounds on sounds.id = playbacks.sound_id \
                where guild_id = $1 and sound_id = $2 \
                order by playbacks.created_at desc")
                .bind(guild_id.0 as i64)
                .bind(sound_id)
        } else {
            sqlx::query_as::<_, Playback>(
                "select playbacks.*, sounds.guild_id from playbacks \
                inner join sounds on sounds.id = playbacks.sound_id \
                where guild_id = $1 \
                order by playbacks.created_at desc")
                .bind(guild_id.0 as i64)
        };

        Ok(query
            .fetch_all(&self.pool)
            .await?)
    }
}

impl FromRow<'_, PgRow> for Guild {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, Error> {
        let id: i64 = row.try_get("id")?;
        Ok(Guild {
            id: GuildId(id as u64)
        })
    }
}

impl FromRow<'_, PgRow> for Sound {
    fn from_row(row: &PgRow) -> std::result::Result<Self, Error> {
        let guild_id: i64 = row.try_get("guild_id")?;
        let uploader_id: i64 = row.try_get("uploader_id")?;
        let length: i64 = row.try_get("length")?;

        Ok(Sound {
            id: row.try_get("id")?,
            guild_id: GuildId(guild_id as u64),
            name: row.try_get("name")?,
            source: row.try_get("source")?,
            uploader_id: UserId(uploader_id as u64),
            length: chrono::Duration::milliseconds(length),
        })
    }
}

impl FromRow<'_, PgRow> for Playback {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, Error> {
        let player_id: i64 = row.try_get("player_id")?;

        let stopper_id: Option<i64> = row.try_get("stopper_id")?;
        let stopper_id: Option<UserId> = stopper_id
            .map_or(
                None,
                |id| Some(UserId(id as u64)),
            );

        Ok(Playback {
            id: row.try_get("id")?,
            started_at: row.try_get("created_at")?,
            stopped_at: row.try_get("stopped_at")?,
            sound_id: row.try_get("sound_id")?,
            player_id: UserId(player_id as u64),
            stopper_id,
        })
    }
}