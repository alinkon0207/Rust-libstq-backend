use futures::Future;
use hyper::header::Headers;
use std::cmp;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::{Error, HttpClient, Response};
use request_util::RequestTimeout as RequestTimeoutHeader;

#[derive(Clone)]
pub struct TimeLimitedHttpClient<S: HttpClient> {
    inner: S,
    initial_time_limit: Duration,
    time_left: Arc<Mutex<Duration>>,
}

impl<S: HttpClient> TimeLimitedHttpClient<S> {
    pub fn new(client: S, time_limit: Duration) -> Self {
        Self {
            inner: client,
            initial_time_limit: time_limit,
            time_left: Arc::new(Mutex::new(time_limit)),
        }
    }
}

impl<S: HttpClient> HttpClient for TimeLimitedHttpClient<S> {
    fn request(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
    ) -> Box<Future<Item = Response, Error = Error> + Send> {
        let time_left_mutex = self.time_left.clone();
        let time_left_before_request = { *time_left_mutex.lock().unwrap() };
        let time_left_before_request_ms = time_left_before_request.as_secs() as u32 * 1000 + time_left_before_request.subsec_millis();

        let mut headers = headers.unwrap_or(Headers::new());
        headers.set(RequestTimeoutHeader(time_left_before_request_ms.to_string()));

        if time_left_before_request == Duration::new(0, 0) {
            return Box::new(futures::future::err(Error::Timeout));
        }

        debug!(
            "Requesting {} {}. Time remaining for client (ms): {}",
            &method, &url, time_left_before_request_ms
        );

        let start_time = Instant::now();
        let request = self
            .inner
            .request(method.clone(), url.clone(), body, Some(headers))
            .map(move |response| {
                // time_left can be updated by a cloned client on another thread (parallel requests)
                // so we calculate the minimum of the current time_left of the client
                // and the time_left that was calculated for this request

                let elapsed_time = Instant::now() - start_time;
                let time_left_after_request = time_left_before_request.checked_sub(elapsed_time).unwrap_or(Duration::new(0, 0));
                let new_time_left = {
                    let mut time_left_current = time_left_mutex.lock().unwrap();
                    let new_time_left = cmp::min(*time_left_current, time_left_after_request);
                    *time_left_current = new_time_left;
                    new_time_left
                };

                let elapsed_time_ms = elapsed_time.as_secs() as u32 * 1000 + elapsed_time.subsec_millis();
                let new_time_left_ms = new_time_left.as_secs() as u32 * 1000 + new_time_left.subsec_millis();
                debug!(
                    "Got response for {} {}. Elapsed time (ms): {}. Time remaining for client (ms): {}",
                    &method, &url, elapsed_time_ms, new_time_left_ms,
                );

                response
            });

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
    use hyper::header::Headers;
    use hyper::Method;
    use tokio_core::reactor::Core;

    use super::*;

    #[test]
    fn time_limited_http_client_returns_error_on_time_exceeded() {
        let mock_client = MockHttpClient::new(Duration::from_millis(10));

        let timed_client = TimeLimitedHttpClient::new(mock_client, Duration::from_millis(9));
        let timed_client_clone = timed_client.clone();

        run_sync(
            timed_client
                .request(Method::Get, "url1".to_string(), None, None)
                .then(|result| {
                    result.expect("First request should have succeeded");
                    timed_client_clone.request(Method::Get, "url2".to_string(), None, None)
                })
                .then(|result| {
                    result.expect_err("Second request should have timed out");
                    future::ok::<(), Error>(())
                }),
        )
    }

    #[test]
    fn time_limited_http_client_correctly_calculates_timeout_on_parallel_requests() {
        let mock_client = MockHttpClient::new(Duration::from_millis(0));
        let request_duration = mock_client.request_duration.clone();

        let timed_client = TimeLimitedHttpClient::new(mock_client, Duration::from_millis(100));

        {
            *request_duration.lock().unwrap() = Duration::from_millis(20);
        };
        let req1 = timed_client.request(Method::Get, "url1".to_string(), None, None);

        {
            *request_duration.lock().unwrap() = Duration::from_millis(5);
        };
        let req2 = timed_client.request(Method::Get, "url2".to_string(), None, None);

        {
            *request_duration.lock().unwrap() = Duration::from_millis(10);
        };
        let req3 = timed_client.request(Method::Get, "url3".to_string(), None, None);

        let all_requests_fut = Future::join3(req1, req2, req3);

        run_sync(all_requests_fut.then(move |result| {
            result.expect("All request should have succeeded");

            let time_left = { *timed_client.time_left.lock().unwrap() };
            println!("Time left (ms): {}.{}", time_left.subsec_millis(), time_left.subsec_micros());

            let expected_time_left = Duration::from_millis(80);
            let tolerance = Duration::from_millis(3);
            assert!(time_left > expected_time_left - tolerance);
            assert!(time_left < expected_time_left);

            futures::future::ok::<_, Error>(())
        }))
    }

    #[test]
    fn time_limited_http_client_sets_request_timeout_header() {
        let mock_client = MockHttpClient::new(Duration::from_millis(1));

        let timed_client = TimeLimitedHttpClient::new(mock_client.clone(), Duration::from_millis(10));

        run_sync(
            timed_client
                .request(Method::Get, "url1".to_string(), None, None)
                .then(move |result| {
                    result.expect("Request should have succeeded");
                    let request = mock_client.next_request().expect("Request has not been sent");
                    let headers = request.headers.into_iter().next().expect("No headers");
                    let request_timeout = headers.get::<RequestTimeoutHeader>().expect("Request-timeout header is missing");
                    assert_eq!("10", request_timeout.0);
                    future::ok::<(), Error>(())
                }),
        )
    }

    #[test]
    fn time_limited_http_client_updates_request_timeout_header() {
        let mock_client = MockHttpClient::new(Duration::from_millis(1));

        let timed_client = TimeLimitedHttpClient::new(mock_client.clone(), Duration::from_millis(10));

        let mut headers = Headers::new();
        headers.set(RequestTimeoutHeader("50".to_string()));

        run_sync(
            timed_client
                .request(Method::Get, "url1".to_string(), None, Some(headers))
                .then(move |result| {
                    result.expect("Request should have succeeded");
                    let request = mock_client.next_request().expect("Request has not been sent");
                    let headers = request.headers.into_iter().next().expect("No headers");
                    let header_view = headers.iter().next().expect("Request-timeout header is missing");
                    let header_value = header_view.value_string();
                    assert_eq!("10", header_value);
                    future::ok::<(), Error>(())
                }),
        )
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
        request_duration: Arc<Mutex<Duration>>,
    }

    #[derive(Debug, Clone)]
    struct Request {
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
    }

    impl MockHttpClient {
        fn new(request_duration: Duration) -> MockHttpClient {
            MockHttpClient {
                requests: Arc::new(Mutex::new(VecDeque::new())),
                request_duration: Arc::new(Mutex::new(request_duration)),
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
            let request_duration = { *self.request_duration.lock().unwrap() };
            Box::new(
                tokio_timer::sleep(request_duration)
                    .map_err(|_| Error::Unknown("Tokio timer error".to_string()))
                    .map(move |_| {
                        requests.lock().unwrap().push_back(Request {
                            method,
                            url,
                            body,
                            headers,
                        });
                        Response(String::new())
                    }),
            )
        }
    }
}
