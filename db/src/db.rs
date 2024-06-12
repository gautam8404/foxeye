use deadpool_redis::{Config, Connection, Pool as RedisPool, Runtime};
use redis::cmd;
use sqlx::pool::PoolConnection;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Postgres};
use tracing::error;

use crate::DbError;

#[derive(Clone, Debug)]
pub struct Db {
    pub pg: PgPool,
    pub redis: RedisPool,
}

impl Db {
    pub async fn new(max_connections: u32) -> Result<Self, DbError> {
        let database_url = std::env::var("DATABASE_URL").map_err(|e| {
            error!("Db::new: error getting database url, DATABASE_URL env not set");
            DbError::Other(e.to_string())
        })?;

        let redis_url = std::env::var("REDIS_URL").map_err(|e| {
            error!("Db::new: error getting database url, REDIS_URL env not set");
            DbError::Other(e.to_string())
        })?;

        let pg = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(&database_url)
            .await?;

        let conf = Config::from_url(redis_url);
        let redis = conf.create_pool(Some(Runtime::Tokio1)).map_err(|e| {
            error!("DB::new: error creating redis pool {e:?}");
            DbError::Other(format!("Db::new: Error creating Redis pool {e}"))
        })?;

        Ok(Self { pg, redis })
    }

    pub async fn get_pg(&self) -> Result<PoolConnection<Postgres>, DbError> {
        Ok(self.pg.acquire().await?)
    }

    async fn cmd_set(
        conn: &mut Connection,
        key: &str,
        val: Vec<u8>,
        ttl: Option<u32>,
    ) -> Result<(), DbError> {
        let mut command = cmd("SET");

        command.arg(key).arg(val);
        if let Some(t) = ttl {
            command.arg("EX").arg(t);
        }

        command.query_async(conn).await?;

        Ok(())
    }

    async fn cmd_get(conn: &mut Connection, key: &str) -> Result<Option<Vec<u8>>, DbError> {
        Ok(cmd("GET").arg(key).query_async(conn).await?)
    }

    async fn cmd_del(conn: &mut Connection, key: &str) -> Result<(), DbError> {
        cmd("DEL").arg(key).query_async(conn).await?;

        Ok(())
    }

    async fn cmd_exist(conn: &mut Connection, key: &str) -> Result<bool, DbError> {
        Ok(cmd("EXISTS").arg(key).query_async(conn).await?)
    }

    pub async fn set_cache(
        &self,
        key: &str,
        val: Vec<u8>,
        ttl: Option<u32>,
    ) -> Result<(), DbError> {
        let mut conn = self.redis.get().await.map_err(|e| {
            error!("Db.set_cache: failed to get redis connection {e:?}");
            DbError::Other(format!("failed to get redis connection {e:?}"))
        })?;

        Self::cmd_set(&mut conn, key, val, ttl).await?;

        Ok(())
    }

    pub async fn get_cache(&self, key: &str) -> Result<Option<Vec<u8>>, DbError> {
        let mut conn = self.redis.get().await.map_err(|e| {
            error!("Db.set_cache: failed to get redis connection {e:?}");
            DbError::Other(format!("failed to get redis connection {e:?}"))
        })?;

        Self::cmd_get(&mut conn, key).await
    }

    pub async fn del_cache(&self, key: &str) -> Result<(), DbError> {
        let mut conn = self.redis.get().await.map_err(|e| {
            error!("Db.set_cache: failed to get redis connection {e:?}");
            DbError::Other(format!("failed to get redis connection {e:?}"))
        })?;

        Self::cmd_del(&mut conn, key).await
    }

    pub async fn exists(&self, key: &str) -> Result<bool, DbError> {
        let mut conn = self.redis.get().await.map_err(|e| {
            error!("Db.set_cache: failed to get redis connection {e:?}");
            DbError::Other(format!("failed to get redis connection {e:?}"))
        })?;

        Self::cmd_exist(&mut conn, key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_get_del_cache() {
        // Set up environment variables

        // Create a new Db instance
        let db = Db::new(5).await.expect("Failed to create Db instance");

        // Test set_cache
        let key = "test_key";
        let value = b"test_value";
        db.set_cache(key, value.to_vec(), None)
            .await
            .expect("Failed to set cache");

        // Test key_exists

        assert!(db.exists(key).await.expect("Failed to check key exists"));

        // Test get_cache
        let retrieved_value = db.get_cache(key).await.expect("Failed to get cache");
        assert!(retrieved_value.is_some());
        assert_eq!(retrieved_value.unwrap(), value);

        // Test del_cache
        db.del_cache(key).await.expect("Failed to delete cache");
        let result = db.get_cache(key).await.expect("failed to get cache");
        assert!(result.is_none()); // Key should not exist anymore
    }
}
