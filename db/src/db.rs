use std::sync::Arc;
use deadpool_redis::{Config, Connection, Pool as RedisPool, Runtime};
use log::error;
use redis::cmd;
use sqlx::{PgConnection, PgPool, Postgres};
use sqlx::pool::PoolConnection;
use sqlx::postgres::PgPoolOptions;
use crate::DbError;

#[derive(Clone)]
pub struct Db {
    pub pg: PgPool,
    pub redis: RedisPool
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
            .connect(&database_url).await?;

        let conf = Config::from_url(redis_url);
        let redis = conf.create_pool(Some(Runtime::Tokio1)).map_err(|e| {
            error!("DB::new: error creating redis pool {e:?}");
            DbError::Other(format!("Db::new: Error creating Redis pool {e}"))
        })?;

        Ok(Self {
            pg,
            redis
        })
    }

    pub async fn get_pg(&self) -> Result<PoolConnection<Postgres> , DbError> {
        Ok(self.pg.acquire().await?)
    }

    async fn cmd_set(conn: &mut Connection, key: &str, val: &[u8], ttl: Option<u16>) -> Result<(), DbError> {
        let mut command = cmd("SET");

        command.arg(key).arg(val);
        if let Some(t) = ttl {
            command.arg("EX").arg(t);
        }

        command.query_async(conn).await?;

        Ok(())
    }

    async fn cmd_get(conn: &mut Connection, key: &str) -> Result<Vec<u8>, DbError> {
        Ok(cmd("GET").arg(key).query_async(conn).await?)
    }

    pub async fn set_cache(&self, key: &str, val: &[u8], ttl: Option<u16>) -> Result<(), DbError> {
        let mut conn = self.redis.get().await.map_err(|e| {
            error!("Db.set_cache: failed to get redis connection {e:?}");
            DbError::Other(format!("failed to get redis connection {e:?}"))
        })?;
        
        Self::cmd_set(&mut conn, key, val, ttl).await?;
        
        Ok(())
    }

    pub async fn get_cache(&self, key: &str) -> Result<Vec<u8>, DbError> {
        let mut conn = self.redis.get().await.map_err(|e| {
            error!("Db.set_cache: failed to get redis connection {e:?}");
            DbError::Other(format!("failed to get redis connection {e:?}"))
        })?;
        
        
        Self::cmd_get(&mut conn, key).await
    } 
}