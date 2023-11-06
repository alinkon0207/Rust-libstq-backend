//! This crate provides common utilities for DB interaction.
extern crate bb8;
extern crate bb8_postgres;
extern crate diesel;
extern crate either;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate futures_state_stream;
extern crate stq_acl;
extern crate tokio_postgres;

pub mod connection;
pub mod diesel_repo;
pub mod pool;
pub mod repo;
pub mod sequence;
pub mod statement;
