pub mod in_memory;
pub mod null;
pub mod redis;
pub mod typed;

use failure::Fail;

pub use self::in_memory::{InMemoryCache, InMemoryCacheError};
pub use self::null::NullCache;
pub use self::typed::{TypedCache, TypedCacheError};

pub trait Cache<T> {
    type Error: Fail;

    fn get(&self, key: &str) -> Result<Option<T>, Self::Error>;

    fn set(&self, key: &str, value: T) -> Result<(), Self::Error>;

    fn remove(&self, key: &str) -> Result<bool, Self::Error>;
}

impl<C, T> Cache<T> for Box<C>
where
    C: ?Sized + Cache<T>,
{
    type Error = C::Error;

    fn get(&self, key: &str) -> Result<Option<T>, Self::Error> {
        (**self).get(key)
    }

    fn set(&self, key: &str, value: T) -> Result<(), Self::Error> {
        (**self).set(key, value)
    }

    fn remove(&self, key: &str) -> Result<bool, Self::Error> {
        (**self).remove(key)
    }
}

pub trait CacheSingle<T> {
    type Error: Fail;

    fn get(&self) -> Result<Option<T>, Self::Error>;

    fn set(&self, value: T) -> Result<(), Self::Error>;

    fn remove(&self) -> Result<bool, Self::Error>;
}

impl<C, E, T> CacheSingle<T> for C
where
    C: Cache<T, Error = E>,
    E: Fail,
{
    type Error = E;

    fn get(&self) -> Result<Option<T>, Self::Error> {
        self.get("")
    }

    fn set(&self, value: T) -> Result<(), Self::Error> {
        self.set("", value)
    }

    fn remove(&self) -> Result<bool, Self::Error> {
        self.remove("")
    }
}
