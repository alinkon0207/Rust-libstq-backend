#[macro_use]
extern crate failure;
extern crate r2d2_redis;
extern crate serde;
extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;

pub mod cache;
