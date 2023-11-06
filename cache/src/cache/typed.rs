use failure::Fail;
use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use std::marker::PhantomData;

use super::Cache;

#[derive(Clone, Debug)]
pub struct TypedCache<C, E, T>
where
    C: Cache<String, Error = E>,
    E: Fail,
    T: DeserializeOwned + Serialize,
{
    backend: C,
    phantom: PhantomData<T>,
}

#[derive(Debug, Fail)]
pub enum TypedCacheError<E>
where
    E: Fail,
{
    #[fail(display = "An error occurred in backend cache")]
    BackendCacheError(E),
    #[fail(display = "An error occurred on JSON serialization/deserialization")]
    JsonError(serde_json::Error),
}

impl<C, E, T> TypedCache<C, E, T>
where
    C: Cache<String, Error = E>,
    E: Fail,
    T: DeserializeOwned + Serialize,
{
    pub fn new(backend: C) -> Self {
        TypedCache {
            backend,
            phantom: PhantomData,
        }
    }
}

impl<C, E, T> Cache<T> for TypedCache<C, E, T>
where
    C: Cache<String, Error = E>,
    E: Fail,
    T: DeserializeOwned + Serialize,
{
    type Error = TypedCacheError<E>;

    fn get(&self, key: &str) -> Result<Option<T>, Self::Error> {
        self.backend
            .get(key)
            .map_err(|e| TypedCacheError::BackendCacheError(e))
            .and_then(|json_opt| match json_opt {
                None => Ok(None),
                Some(json) => serde_json::from_str(&json)
                    .map(Some)
                    .map_err(|e| TypedCacheError::JsonError(e)),
            })
    }

    fn set(&self, key: &str, value: T) -> Result<(), Self::Error> {
        serde_json::to_string(&value)
            .map_err(|e| TypedCacheError::JsonError(e))
            .and_then(|json| {
                self.backend
                    .set(key, json)
                    .map_err(|e| TypedCacheError::BackendCacheError(e))
            })
    }

    fn remove(&self, key: &str) -> Result<bool, Self::Error> {
        self.backend
            .remove(key)
            .map_err(|e| TypedCacheError::BackendCacheError(e))
    }
}

#[cfg(test)]
mod tests {
    use cache::{in_memory::InMemoryCache, typed::TypedCache, Cache};

    #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct TestStruct {
        pub s: String,
        pub i: i32,
    }

    #[test]
    fn test_typed_cache() {
        let backend = InMemoryCache::<String>::new();
        let typed_cache = TypedCache::<_, _, TestStruct>::new(backend);

        let key = "key";
        let original_value = TestStruct {
            s: "string".to_string(),
            i: 10,
        };

        typed_cache
            .set(key, original_value.clone())
            .expect("Failed to set value");
        let value_from_cache = typed_cache
            .get(key)
            .expect("Failed to get value")
            .expect("Value does not exist in cache");
        assert_eq!(original_value, value_from_cache);

        let value_was_removed = typed_cache.remove(key).expect("Failed to remove value");
        assert!(value_was_removed);

        let non_existent_value_was_removed = typed_cache
            .remove(key)
            .expect("Failed to attempt to remove value");
        assert!(!non_existent_value_was_removed);

        let missing_value = typed_cache.get(key).expect("Failed to get value");
        assert_eq!(None, missing_value);
    }
}
