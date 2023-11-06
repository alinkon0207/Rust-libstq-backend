use r2d2_redis::{
    r2d2::{ManageConnection, Pool},
    redis::{cmd, Connection as RedisConnection, RedisError},
};
use std::time::Duration;

use cache::Cache;

#[derive(Clone, Debug)]
pub struct RedisCache<M>
where
    M: ManageConnection<Connection = RedisConnection>,
{
    namespace: String,
    pool: Pool<M>,
    ttl: Option<Duration>,
}

#[derive(Debug, Fail)]
pub enum RedisCacheError {
    #[fail(display = "No available Redis connections left")]
    NoAvailableConnections,
    #[fail(display = "{}", _0)]
    RedisError(RedisError),
}

impl From<RedisError> for RedisCacheError {
    fn from(e: RedisError) -> Self {
        RedisCacheError::RedisError(e)
    }
}

impl<M> RedisCache<M>
where
    M: ManageConnection<Connection = RedisConnection>,
{
    pub fn new(pool: Pool<M>, namespace: String) -> Self {
        RedisCache {
            namespace: String::from(namespace),
            pool,
            ttl: None,
        }
    }

    pub fn with_ttl(self, ttl: Duration) -> Self {
        RedisCache {
            ttl: Some(ttl),
            ..self
        }
    }

    fn make_redis_key(&self, key: &str) -> String {
        format!("{}:{}", self.namespace, key)
    }

    fn using_connection<T, F>(&self, f: F) -> Result<T, RedisCacheError>
    where
        F: Fn(&RedisConnection) -> T,
    {
        self.pool
            .try_get()
            .map(|conn| f(&conn))
            .ok_or(RedisCacheError::NoAvailableConnections)
    }
}

impl<M> Cache<String> for RedisCache<M>
where
    M: ManageConnection<Connection = RedisConnection>,
{
    type Error = RedisCacheError;

    fn get(&self, key: &str) -> Result<Option<String>, Self::Error> {
        self.using_connection(|conn| cmd("GET").arg(self.make_redis_key(key)).query(conn))
            .and_then(|res| res.map_err(From::from))
    }

    fn set(&self, key: &str, value: String) -> Result<(), Self::Error> {
        self.using_connection(|conn| match self.ttl {
            None => cmd("SET")
                .arg(self.make_redis_key(key))
                .arg(&value)
                .query(conn),
            Some(ttl) => cmd("SETEX")
                .arg(self.make_redis_key(key))
                .arg(ttl.as_secs())
                .arg(&value)
                .query(conn),
        })
        .and_then(|res| res.map_err(From::from))
    }

    fn remove(&self, key: &str) -> Result<bool, Self::Error> {
        self.using_connection(|conn| {
            cmd("DEL")
                .arg(self.make_redis_key(key))
                .query(conn)
                .map(|keys_removed: u32| if keys_removed > 0 { true } else { false })
        })
        .and_then(|res| res.map_err(From::from))
    }
}
