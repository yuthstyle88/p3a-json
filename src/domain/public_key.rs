use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicKey {
    pub id: i32,
    pub key: String,
    pub speed: String,
}

impl PublicKey {
    pub async fn get_public_key(pool: &PgPool, speed: &str) -> Result<PublicKey, anyhow::Error> {
        let public_key = sqlx::query_as!(
            PublicKey,
            r#"
        SELECT id, key, speed
        FROM public_keys
        WHERE
            speed = $1
        "#,
            speed
        )
        .fetch_one(pool)
        .await?;
        Ok(public_key)
    }
}
