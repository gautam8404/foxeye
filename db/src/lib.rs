pub mod db;

use thiserror::Error;

pub use db::Db;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Sqlx Error: {0:?}")]
    Postgres(#[from] sqlx::Error),
    #[error("Redis Error: {0:?}")]
    Redis(#[from] redis::RedisError),
    #[error("Some error occurred: {0}")]
    Other(String),
}
