pub mod models {
    use serde::{Deserialize, Serialize};
    use chrono;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TelemetryEvent {
        pub id: Option<i64>,
        pub cadence: String,
        pub channel: String,
        pub country_code: String,
        pub metric_name: String,
        pub metric_value: i32,
        pub platform: String,
        pub version: String,
        pub woi: i16,
        pub wos: Option<i16>,
        pub yoi: i16,
        pub yos: i16,
        pub received_at: Option<chrono::DateTime<chrono::Utc>>,
    }
}

pub async fn insert_event(
    pool: &sqlx::PgPool,
    event: &models::TelemetryEvent,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO telemetry_events (
            cadence, channel, country_code, metric_name, metric_value,
            platform, version, woi, wos, yoi, yos, received_at
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9, $10, $11, $12
        )",
        event.cadence,
        event.channel,
        event.country_code,
        event.metric_name,
        event.metric_value,
        event.platform,
        event.version,
        event.woi,
        event.wos.unwrap_or(0),
        event.yoi,
        event.yos,
        event.received_at.unwrap_or_else(chrono::Utc::now)
    )
        .execute(pool)
        .await?;
    Ok(())
}