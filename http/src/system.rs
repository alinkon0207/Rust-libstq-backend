use failure;
use futures::future;
use futures::prelude::*;

/// System service, responsible for common endpoints like healthcheck
pub trait SystemService {
    /// Healthcheck endpoint, always returns OK status
    fn healthcheck(&self) -> Box<Future<Item = String, Error = failure::Error>>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemServiceImpl;

impl SystemService for SystemServiceImpl {
    /// Healthcheck endpoint, always returns OK status
    fn healthcheck(&self) -> Box<Future<Item = String, Error = failure::Error>> {
        Box::new(future::ok("\"Ok\"".to_string()))
    }
}
