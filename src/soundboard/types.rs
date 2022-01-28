use chrono::Utc;

pub type Snowflake = u64;

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
