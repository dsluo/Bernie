use anyhow::Result;
use async_trait::async_trait;
use sqlx::{Error, FromRow, PgPool, Row};
use sqlx::postgres::PgRow;
use url::Url;

use crate::{
    soundboard::{
        Playback,
        Sound,
        Guild,
        Snowflake
    },
    SoundboardBackend,
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

    async fn get_sound_id(&self, guild_id: Snowflake, name: &str) -> Result<i32> {
        Ok(sqlx::query(
            "select id from sounds \
            where guild_id = ? and name = ? and deleted_at is null")
            .bind(guild_id as i64)
            .bind(name)
            .map(|row: PgRow| row.get(0))
            .fetch_one(&self.pool)
            .await?)
    }
}

#[async_trait]
impl SoundboardBackend for DatabaseBackend {
    async fn ensure_guild(&self, guild_id: Snowflake) -> Result<Guild> {
        Ok(sqlx::query_as::<_, Guild>(
            "insert into guilds \
            values(?) \
            on conflict do nothing \
            returning *")
            .bind(guild_id as i64)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn add(&self, guild_id: Snowflake, uploader_id: Snowflake, name: &str, source: Url) -> Result<Sound> {
        let guild = self.ensure_guild(guild_id).await?;

        let length = chrono::Duration::zero();

        Ok(sqlx::query_as::<_, Sound>(
            "insert into sounds(guild_id, name, source, uploader_id, length) \
            values(?, ?, ?, ?, ?) \
            returning *")
            .bind(guild.id as i64)
            .bind(name)
            .bind(source.as_str())
            .bind(uploader_id as i64)
            .bind(length)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn list(&self, guild_id: Snowflake) -> Result<Vec<Sound>> {
        let guild = self.ensure_guild(guild_id).await?;

        Ok(sqlx::query_as::<_, Sound>(
            "select * from sounds \
            where guild_id = ? and deleted_at is null")
            .bind(guild.id as i64)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn rename(&self, guild_id: Snowflake, old_name: &str, new_name: &str) -> Result<Sound> {
        let guild = self.ensure_guild(guild_id).await?;

        Ok(sqlx::query_as::<_, Sound>(
            "update sounds set name = ? \
            where guild_id = ? and name = ? and deleted_at is null \
            returning *")
            .bind(new_name)
            .bind(guild.id as i64)
            .bind(old_name)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn remove(&self, guild_id: Snowflake, name: &str) -> Result<Sound> {
        let guild = self.ensure_guild(guild_id).await?;

        Ok(sqlx::query_as::<_, Sound>(
            "update sounds set deleted_at = current_timestamp \
            where guild_id = ? and name = ? and deleted_at is null \
            returning *")
            .bind(guild.id as i64)
            .bind(name)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn autocomplete(&self, guild_id: Snowflake, starts_with: &str) -> Result<Vec<String>> {
        let guild = self.ensure_guild(guild_id).await?;

        Ok(sqlx::query(
            "select name from sounds \
            where guild_id = ? and starts_with(name, ?) and deleted_at is null \
            limit 25")
            .bind(guild.id as i64)
            .bind(starts_with)
            .map(|row: PgRow| row.get("name"))
            .fetch_all(&self.pool)
            .await?)
    }

    async fn play(&self, guild_id: Snowflake, player_id: Snowflake, name: &str) -> Result<Playback> {
        let guild = self.ensure_guild(guild_id).await?;

        let sound_id: i32 = self.get_sound_id(guild_id, name).await?;

        Ok(sqlx::query_as::<_, Playback>(
            "insert into playbacks(sound_id, player_id) \
            values(?, ?) \
            returning *")
            .bind(sound_id)
            .bind(player_id as i64)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn random(&self, guild_id: Snowflake, player_id: Snowflake) -> Result<Playback> {
        // let guild = self.ensure_guild(guild_id).await?;
        //
        // let name = "asdf"; // todo
        //
        // let sound_id: i64 = sqlx::query(
        //     "select id from sounds \
        //     where guild_id = ? and name = ? and deleted_at is null")
        //     .bind(guild.id as i64)
        //     .bind(name)
        //     .map(|row: PgRow| row.get(0))
        //     .fetch_one(&self.pool)
        //     .await?;
        //
        // Ok(sqlx::query_as::<_, Playback>(
        //     "insert into playbacks(sound_id, player_id) \
        //     values(?, ?) \
        //     returning *")
        //     .bind(sound_id)
        //     .bind(player_id as i64)
        //     .fetch_one(&self.pool)
        //     .await?);

        todo!()
    }

    async fn stop(&self, guild_id: Snowflake, stopper_id: Snowflake, name: &str) -> Result<Playback> {
        let guild = self.ensure_guild(guild_id).await?;
        let sound = self.get_sound_id(guild.id, name).await?;

        Ok(sqlx::query_as::<_, Playback>(
            "update playbacks set stopper_id = ?, stopped_at = current_timestamp \
            where sound_id = ? and stopped_at is null \
            returning *")
            .bind(stopper_id as i64)
            .bind(sound)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn history(&self, guild_id: Snowflake, name: Option<&str>) -> Result<Vec<Playback>> {
        let guild = self.ensure_guild(guild_id).await?;

        let query = if let Some(name) = name {
            let sound_id = self.get_sound_id(guild_id, name).await?;
            sqlx::query_as::<_, Playback>(
                "select playbacks.*, sounds.guild_id from playbacks \
                inner join sounds on sounds.id = playbacks.sound_id \
                where guild_id = ? and sound_id = ? \
                order by playbacks.created_at desc")
                .bind(guild.id as i64)
                .bind(sound_id)
        } else {
            sqlx::query_as::<_, Playback>(
                "select playbacks.*, sounds.guild_id from playbacks \
                inner join sounds on sounds.id = playbacks.sound_id \
                where guild_id = ? \
                order by playbacks.created_at desc")
                .bind(guild.id as i64)
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
            id: id as Snowflake
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
            guild_id: guild_id as Snowflake,
            name: row.try_get("name")?,
            source: row.try_get("source")?,
            uploader_id: uploader_id as Snowflake,
            length: chrono::Duration::milliseconds(length),
        })
    }
}

impl FromRow<'_, PgRow> for Playback {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, Error> {
        let player_id: i64 = row.try_get("player_id")?;

        let stopper_id: Option<i64> = row.try_get("stopper_id")?;
        let stopper_id: Option<Snowflake> = stopper_id
            .map_or(
                None,
                |id| Some(id as Snowflake),
            );

        Ok(Playback {
            id: row.try_get("id")?,
            started_at: row.try_get("created_at")?,
            stopped_at: row.try_get("stopped_at")?,
            sound_id: row.try_get("sound_id")?,
            player_id: player_id as Snowflake,
            stopper_id,
        })
    }
}