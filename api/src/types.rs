use errors::Error;

use futures::prelude::*;
use reqwest::async::Body;
use serde::Serialize;
use serde_json;
use std::convert::Into;
use tokio_core::reactor::Core;

pub trait ApiFutureExt<T, E> {
    fn sync(self) -> Result<T, E>;
}

impl<T> ApiFutureExt<T, Error> for ApiFuture<T> {
    fn sync(self) -> Result<T, Error> {
        let mut core = Core::new().unwrap();
        core.run(self)
    }
}

pub type ApiFuture<T> = Box<Future<Item = T, Error = Error> + Send>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ValueContainer<T> {
    pub value: T,
}

impl<T> From<T> for ValueContainer<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

pub struct JsonPayload<T>(pub T);

impl<T> Into<Body> for JsonPayload<T>
where
    T: Serialize,
{
    fn into(self) -> Body {
        serde_json::to_string(&self.0).unwrap().into()
    }
}
