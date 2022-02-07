use crate::{Context, Error};

pub(super) async fn autocomplete_sound_name(ctx: Context<'_>, partial: String) -> Vec<String> {
    let db = &ctx.data().db;

    let guild_id = ctx.guild_id().unwrap();

    sqlx::query!(
        "select name from sounds \
        where guild_id = $1 and starts_with(name, $2) and deleted_at is null \
        order by name \
        limit 25",
        guild_id.0 as i64,
        partial
    )
    .map(|record| record.name.to_string())
    .fetch_all(db)
    .await
    .unwrap_or_default()
}

pub(super) async fn ensure_guild_check(ctx: Context<'_>) -> Result<bool, Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let guild_id = guild_id.0 as i64;
        let db = &ctx.data().db;
        let storage_dir = &ctx.data().storage_dir;

        sqlx::query!(
            "insert into guilds \
            values($1) \
            on conflict do nothing",
            guild_id
        )
        .execute(db)
        .await?;

        let guild_dir = storage_dir.join(guild_id.to_string());

        if !guild_dir.is_dir() {
            tokio::fs::create_dir(&guild_dir)
                .await
                .expect(&format!("Couldn't create guild directory: {:?}", guild_dir));
        }

        Ok(true)
    } else {
        Ok(false)
    }
}
