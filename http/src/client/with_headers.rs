use futures::Future;
use hyper::header::Headers;
use std::sync::{Arc, Mutex};

use super::{Error, HttpClient, Response};

#[derive(Clone)]
pub struct HttpClientWithDefaultHeaders<S: HttpClient> {
    inner: S,
    headers: Arc<Mutex<Headers>>,
}

impl<S: HttpClient> HttpClientWithDefaultHeaders<S> {
    pub fn new(client: S, headers: Headers) -> Self {
        Self {
            inner: client,
            headers: Arc::new(Mutex::new(headers)),
        }
    }
}

impl<S: HttpClient> HttpClient for HttpClientWithDefaultHeaders<S> {
    fn request(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
    ) -> Box<Future<Item = Response, Error = Error> + Send> {
        let self_headers = self.headers.clone();
        let mut existing_headers = (*self_headers.lock().unwrap()).clone();

        if let Some(headers) = headers {
            existing_headers.extend(headers.iter());
        };

        let request = self.inner.request(method, url, body, Some(existing_headers));
        Box::new(request)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    use futures::future;
    use futures::prelude::*;
    use hyper;
    use hyper::header::{Authorization, Headers};
    use hyper::Method;
    use tokio_core::reactor::Core;

    use super::*;

    #[test]
    fn new_headers_override_existing_headers() {
        //given
        let mock_client = MockHttpClient::new();
        let client_with_old_default_headers = HttpClientWithDefaultHeaders::new(mock_client.clone(), headers("old_auth"));
        let client_with_new_headers = HttpClientWithDefaultHeaders::new(client_with_old_default_headers, headers("new_auth"));

        run_sync(
            //when
            client_with_new_headers
                .request(Method::Get, "url".to_string(), None, None)
                .map(move |_response| {
                    //then
                    assert_eq!(
                        mock_client.next_request().unwrap().headers.unwrap().get(),
                        Some(&Authorization("new_auth".to_string()))
                    );
                }),
        )
    }

    fn headers(auth_header: &str) -> Headers {
        let mut headers = Headers::new();
        headers.set(Authorization(auth_header.to_string()));
        headers
    }

    fn run_sync<E, F>(fut: F) -> F::Item
    where
        E: std::fmt::Debug,
        F: Future<Error = E>,
    {
        let mut core = Core::new().unwrap();
        core.run(fut).unwrap()
    }

    #[derive(Clone)]
    struct MockHttpClient {
        requests: Arc<Mutex<VecDeque<Request>>>,
    }

    #[derive(Debug, Clone)]
    struct Request {
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
    }

    impl MockHttpClient {
        fn new() -> MockHttpClient {
            MockHttpClient {
                requests: Arc::new(Mutex::new(VecDeque::new())),
            }
        }

        fn next_request(&self) -> Option<Request> {
            self.requests.lock().unwrap().pop_front()
        }
    }

    impl HttpClient for MockHttpClient {
        fn request(
            &self,
            method: hyper::Method,
            url: String,
            body: Option<String>,
            headers: Option<Headers>,
        ) -> Box<Future<Item = Response, Error = Error> + Send> {
            let requests = self.requests.clone();
            requests.lock().unwrap().push_back(Request {
                method,
                url,
                body,
                headers,
            });

            Box::new(future::ok(Response(String::new())))
        }
    }
}
