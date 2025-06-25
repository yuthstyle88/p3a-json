use derive_more::{Display, Error, From};

#[derive(From, Error, Debug, Display)]
pub enum PgStoreError {
    #[display("sqlx error: {}", _0)]
    SqlxErr(sqlx::Error),
    #[display("pool error")]
    PoolTimeout,
    #[display("failed to apply migrations")]
    Migration,
}