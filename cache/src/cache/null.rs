use failure::Fail;
use std::marker::PhantomData;

use super::Cache;

#[derive(Clone, Debug)]
pub struct NullCache<T, E> {
    phantom_t: PhantomData<T>,
    phantom_e: PhantomData<E>,
}

impl<T, E> NullCache<T, E> {
    pub fn new() -> Self {
        NullCache {
            phantom_t: PhantomData,
            phantom_e: PhantomData,
        }
    }
}

impl<T, E: Fail> Cache<T> for NullCache<T, E> {
    type Error = E;

    fn get(&self, _key: &str) -> Result<Option<T>, Self::Error> {
        Ok(None)
    }

    fn set(&self, _key: &str, _value: T) -> Result<(), Self::Error> {
        Ok(())
    }

    fn remove(&self, _key: &str) -> Result<bool, Self::Error> {
        Ok(false)
    }
}
