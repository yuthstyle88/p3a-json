use std::sync::Arc;
use crate::models::DBPool;
use crate::payload::MyPayload;

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
pub async fn insert_events(
    pool: Arc<DBPool>,
    events: Vec<MyPayload>,
) -> Result<(), sqlx::Error> {
    if events.is_empty() {
        return Ok(());
    }

    // เตรียมข้อมูลแต่ละ column เป็น Vec
    let cadence: Vec<&str> = events.iter().map(|e| e.cadence.as_str()).collect();
    let channel: Vec<&str> = events.iter().map(|e| e.channel.as_str()).collect();
    let country_code: Vec<&str> = events.iter().map(|e| e.country_code.as_str()).collect();
    let metric_name: Vec<&str> = events.iter().map(|e| e.metric_name.as_str()).collect();
    let metric_value: Vec<f64> = events.iter().map(|e| e.metric_value as f64).collect();
    let platform: Vec<&str> = events.iter().map(|e| e.platform.as_str()).collect();
    let version: Vec<&str> = events.iter().map(|e| e.version.as_str()).collect();
    let woi: Vec<i32> = events.iter().map(|e| e.woi as i32).collect();
    let wos: Vec<i32> = events.iter().map(|e| e.wos.unwrap_or(0) as i32).collect();
    let yoi: Vec<i32> = events.iter().map(|e| e.yoi as i32).collect();
    let yos: Vec<i32> = events.iter().map(|e| e.yos as i32).collect();

    sqlx::query(
        r#"
        INSERT INTO telemetry_events (
            cadence, channel, country_code, metric_name, metric_value,
            platform, version, woi, wos, yoi, yos
        )
        SELECT *
        FROM UNNEST(
            $1::text[],      -- cadence
            $2::text[],      -- channel
            $3::text[],      -- country_code
            $4::text[],      -- metric_name
            $5::float8[],    -- metric_value
            $6::text[],      -- platform
            $7::text[],      -- version
            $8::int4[],      -- woi
            $9::int4[],      -- wos
            $10::int4[],     -- yoi
            $11::int4[]     -- yos
        )
        "#
    )
        .bind(cadence)
        .bind(channel)
        .bind(country_code)
        .bind(metric_name)
        .bind(metric_value)
        .bind(platform)
        .bind(version)
        .bind(woi)
        .bind(wos)
        .bind(yoi)
        .bind(yos)
        .execute(&pool.inner_pool)
        .await?;

    Ok(())
}