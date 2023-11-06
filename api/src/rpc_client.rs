use util::*;

use hyper::{
    header::{HeaderName, HeaderValue},
    HeaderMap,
};
use reqwest::async::{Client as HttpClient, ClientBuilder as HttpClientBuilder};
use std::sync::Arc;
use stq_types::UserId;

#[derive(Clone, Debug)]
pub struct RestApiClient {
    pub(crate) http_client: Arc<HttpClient>,
    pub(crate) base_url: String,
}

impl RestApiClient {
    pub fn new<S>(base_url: &S, caller_id: Option<UserId>) -> Self
    where
        S: ToString,
    {
        Self {
            base_url: base_url.to_string(),
            http_client: Arc::new(
                HttpClientBuilder::new()
                    .default_headers(RestApiClient::get_auth_headers(caller_id))
                    .build()
                    .unwrap(),
            ),
        }
    }

    pub fn new_with_default_headers<S>(
        base_url: &S,
        caller_id: Option<UserId>,
        headers: Option<HeaderMap>,
    ) -> Self
    where
        S: ToString,
    {
        let mut default_headers = match headers {
            Some(v) => v,
            None => HeaderMap::new(),
        };

        default_headers.extend(RestApiClient::get_auth_headers(caller_id));

        Self {
            base_url: base_url.to_string(),
            http_client: Arc::new(
                HttpClientBuilder::new()
                    .default_headers(default_headers)
                    .build()
                    .unwrap(),
            ),
        }
    }

    fn get_auth_headers(caller_id: Option<UserId>) -> HeaderMap {
        match caller_id {
            Some(v) => vec![(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(&v.to_string()).unwrap(),
            )],
            None => vec![],
        }
        .into_iter()
        .collect::<HeaderMap>()
    }

    pub fn build_route(&self, route_builder: &RouteBuilder) -> String {
        route_builder.build_route(Some(&self.base_url))
    }
}
