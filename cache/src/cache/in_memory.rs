use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::Cache;

#[derive(Clone, Debug)]
pub struct InMemoryCache<T>(Arc<RwLock<HashMap<String, T>>>);

impl<T> InMemoryCache<T> {
    pub fn new() -> InMemoryCache<T> {
        InMemoryCache(Arc::new(RwLock::new(HashMap::default())))
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Unexpected error occurred in in-memory cache")]
pub struct InMemoryCacheError;

impl<T> Cache<T> for InMemoryCache<T>
where
    T: Clone,
{
    type Error = InMemoryCacheError;

    fn get(&self, key: &str) -> Result<Option<T>, Self::Error> {
        let lock = self.0.clone();
        let hash_map = lock.read().map_err(|_| InMemoryCacheError)?;
        Ok(hash_map.get(key).cloned())
    }

    fn set(&self, key: &str, value: T) -> Result<(), Self::Error> {
        let lock = self.0.clone();
        let mut hash_map = lock.write().map_err(|_| InMemoryCacheError)?;
        hash_map.insert(key.to_string(), value);
        Ok(())
    }

    fn remove(&self, key: &str) -> Result<bool, Self::Error> {
        let lock = self.0.clone();
        let mut hash_map = lock.write().map_err(|_| InMemoryCacheError)?;
        Ok(match hash_map.remove(key) {
            None => false,
            Some(_) => true,
        })
    }
}
