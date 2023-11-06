extern crate r2d2_redis;
extern crate stq_cache;

use r2d2_redis::{r2d2::Pool, RedisConnectionManager};
use std::time::Duration;
use stq_cache::cache::{redis::RedisCache, Cache};

#[test]
fn test_redis_cache() {
    let redis_url = std::env::vars()
        .find(|(k, _v)| k == "REDIS_URL")
        .map(|(_k, v)| v)
        .unwrap_or("redis://127.0.0.1/".to_string());

    let manager = RedisConnectionManager::new(redis_url.as_ref())
        .expect("Failed to create connection manager");

    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create connection pool");

    let ttl = Duration::from_secs(3);
    let cache = RedisCache::new(pool.clone(), "base_key".to_string()).with_ttl(ttl);

    cache
        .set("key", "value".to_string())
        .expect("Failed to set value");

    let cached_value = cache
        .get("key")
        .expect("Failed to get value")
        .expect("Redis did not return a value");
    assert_eq!("value", cached_value);

    let existing_key_was_deleted = cache.remove("key").expect("Failed to delete value");
    assert!(existing_key_was_deleted);

    let non_existing_key_was_deleted = cache
        .remove("non_existing_key")
        .expect("Failed to attempt to delete value");
    assert!(!non_existing_key_was_deleted);

    cache
        .set("key_2", "value_2".to_string())
        .expect("Failed to set value");
    cache
        .get("key_2")
        .expect("Failed to get value")
        .expect("Redis did not return a value");

    std::thread::sleep(ttl + Duration::from_secs(1));

    let expired_value_2 = cache.get("key_2").expect("Failed to get value");
    assert_eq!(None, expired_value_2);
}
