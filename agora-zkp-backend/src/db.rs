use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub struct DBClient {
    pool: Pool<Postgres>,
}

impl DBClient {
    pub fn new(db_uri: String) -> Result<DBClient> {
        let pool = PgPoolOptions::new()
            .idle_timeout(Some(std::time::Duration::from_secs(30)))
            .connect_lazy(&db_uri)?;
        Ok(DBClient { pool })
    }

    pub async fn set_rpoint_guildid(&self, rpoint: &str, guild_id: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO rpoint_guildid (rpoint, guildid) VALUES ($1, $2)",
            rpoint,
            guild_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn check_rpoint_guildid(&self, rpoint: &str, guild_id: &str) -> Result<bool> {
        let result = sqlx::query!(
            "SELECT * FROM rpoint_guildid WHERE rpoint = $1 AND guildid = $2",
            rpoint,
            guild_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result.rpoint.is_some())
    }
}
