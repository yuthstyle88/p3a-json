mod error;
pub use error::*;

use rand::seq::IndexedRandom;
use sqlx::pool::PoolConnection;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgConnection, Pool, Postgres};
use std::env;
use std::ops::DerefMut;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

use crate::channel::get_data_channel_value_from_env;
use crate::profiler::Profiler;
const DATABASE_URL_ENV_KEY: &str = "DATABASE_URL";
const TEST_DATABASE_URL_ENV_KEY: &str = "TEST_DATABASE_URL";
const DATABASE_NAMES_ENV_KEY: &str = "p3a";
const DEFAULT_DATABASE_NAMES: &str = "p3a=p3a_db";
const MAX_CONN_ENV_KEY: &str = "DATABASE_MAX_CONN";
const MAX_CONN_DEFAULT: &str = "10";
const MAX_WRITE_CONN_ENV_KEY: &str = "DATABASE_MAX_WRITE_CONN";
const MAX_WRITE_CONN_DEFAULT: &str = "8";
const DB_POOL_TIMEOUT_SECS: u64 = 3600;
const DB_POOL_POLL_MS: u64 = 100;

pub type DBConnection = sqlx::pool::PoolConnection<sqlx::Postgres>;

pub enum DBConnectionType<'a> {
    #[allow(dead_code)]
    Test,
    Normal {
        channel_name: &'a str,
    },
}

pub struct DBPool {
    pub(crate) inner_pool: Pool<Postgres>,
}

fn get_channel_db_url(conn_type: &DBConnectionType<'_>) -> String {
    let env_key = match conn_type {
        DBConnectionType::Test => TEST_DATABASE_URL_ENV_KEY,
        DBConnectionType::Normal { .. } => DATABASE_URL_ENV_KEY,
    };
    let db_url =
        env::var(env_key).unwrap_or_else(|_| panic!("{} env var must be defined", env_key));
    match conn_type {
        DBConnectionType::Test => db_url,
        DBConnectionType::Normal { channel_name } => {
            let database_name = get_data_channel_value_from_env(
                DATABASE_NAMES_ENV_KEY,
                DEFAULT_DATABASE_NAMES,
                channel_name,
            );
            format!("{}/{}", db_url, database_name)
        }
    }
}

impl DBPool {
    pub async fn new<'a>(conn_type: DBConnectionType<'a>) -> Self {
        let pool_max_size =
            u32::from_str(&env::var(MAX_CONN_ENV_KEY).unwrap_or(MAX_CONN_DEFAULT.to_string()))
                .unwrap_or_else(|_| panic!("{} must be a positive integer", MAX_CONN_ENV_KEY));

        let db_url = get_channel_db_url(&conn_type);
        let pool = PgPoolOptions::new()
            .max_connections(pool_max_size)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&db_url)
            .await
            .expect("Could not connect to database");

        Self { inner_pool: pool }
    }

    pub async fn get(&self) -> Result<PoolConnection<Postgres>, PgStoreError> {
        let timeout_duration = Duration::from_secs(DB_POOL_TIMEOUT_SECS);
        let poll_duration = Duration::from_millis(DB_POOL_POLL_MS);
        let start_instant = Instant::now();
        while start_instant.elapsed() < timeout_duration {
            if let Ok(conn) = self.inner_pool.acquire().await {
                return Ok(conn);
            }
            sleep(poll_duration).await;
        }
        Err(PgStoreError::PoolTimeout)
    }
}

pub struct DBStorageConnections {
    conns: Vec<Arc<Mutex<DBConnection>>>,
}
impl DBStorageConnections {
    pub async fn new(db_pool: &Arc<DBPool>, using_test_db: bool) -> Result<Self, PgStoreError> {
        let conn_count = if using_test_db {
            1
        } else {
            usize::from_str(
                &env::var(MAX_WRITE_CONN_ENV_KEY).unwrap_or(MAX_WRITE_CONN_DEFAULT.to_string()),
            )
            .unwrap_or_else(|_| panic!("{} must be a positive integer", MAX_WRITE_CONN_ENV_KEY))
        };
        let mut conns = Vec::with_capacity(conn_count);
        for _ in 0..conn_count {
            let conn = Arc::new(Mutex::new(db_pool.get().await?));
            begin_db_transaction(conn.clone())?;
            conns.push(conn);
        }
        Ok(Self { conns })
    }

    pub fn get(&self) -> Arc<Mutex<PoolConnection<Postgres>>> {
        use rand::{seq::SliceRandom, thread_rng};
        self.conns.choose(&mut thread_rng()).unwrap().clone()
    }

    pub fn commit(&self) -> Result<(), PgStoreError> {
        for conn in &self.conns {
            commit_db_transaction(conn.clone())?;
        }
        Ok(())
    }
}

pub fn begin_db_transaction(
    conn: Arc<Mutex<PoolConnection<Postgres>>>,
) -> Result<(), PgStoreError> {
    let guard = conn.lock().unwrap();
    Ok(())
}

pub fn commit_db_transaction(
    conn: Arc<Mutex<PoolConnection<Postgres>>>,
) -> Result<(), PgStoreError> {
    let guard = conn.lock().unwrap();
    Ok(())
}
