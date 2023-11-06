pub mod time_limited;
pub mod with_headers;

pub use self::time_limited::*;
pub use self::with_headers::*;

use std::fmt;
use std::mem;
use std::time::Duration;

use futures::future;
use futures::future::Either;
use futures::prelude::*;
use futures::sync::{mpsc, oneshot};
use hyper;
use hyper::header::{Authorization, Headers};
use hyper_tls::HttpsConnector;
use juniper::FieldError;
use serde::de::Deserialize;
use serde_json;
use tokio_core;
use tokio_core::reactor::Handle;

use errors::ErrorMessage;
use request_util::read_body;

#[derive(Clone, Debug)]
pub struct Response(String);

pub trait HttpClient: Send + Sync + 'static {
    fn request(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
    ) -> Box<Future<Item = Response, Error = Error> + Send>;

    fn request_json<T>(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
    ) -> Box<Future<Item = T, Error = Error> + Send>
    where
        T: for<'a> Deserialize<'a> + 'static + Send,
        Self: Sized,
    {
        Box::new(self.request(method, url, body, headers).and_then(|response| {
            if response.0.is_empty() {
                serde_json::from_value(serde_json::Value::Null)
            } else {
                serde_json::from_str::<T>(&response.0)
            }
            .map_err(|e| Error::Parse(e.to_string()))
        }))
    }
}

pub type ClientResult = Result<String, Error>;

pub type HyperClient = hyper::Client<HttpsConnector<hyper::client::HttpConnector>>;

pub struct Config {
    pub http_client_retries: usize,
    pub http_client_buffer_size: usize,
    pub timeout_duration_ms: u64,
}

pub struct Client {
    client: HyperClient,
    tx: mpsc::Sender<Payload>,
    rx: mpsc::Receiver<Payload>,
    max_retries: usize,
    timeout_duration_ms: u64,
    handle: Handle,
}

impl Client {
    pub fn new(config: &Config, handle: &Handle) -> Self {
        let max_retries = config.http_client_retries;
        let timeout_duration_ms = config.timeout_duration_ms;
        let (tx, rx) = mpsc::channel::<Payload>(config.http_client_buffer_size);
        let client = hyper::Client::configure()
            .connector(HttpsConnector::new(4, &handle).unwrap())
            .build(handle);

        Client {
            client,
            tx,
            rx,
            max_retries,
            timeout_duration_ms,
            handle: handle.clone(),
        }
    }

    pub fn stream(self) -> Box<Stream<Item = (), Error = ()>> {
        let Self {
            client,
            rx,
            handle,
            timeout_duration_ms,
            ..
        } = self;

        Box::new(rx.and_then(move |payload| Self::send_request(&handle, &client, payload, timeout_duration_ms).then(|_| Ok(()))))
    }

    pub fn handle(&self) -> ClientHandle {
        ClientHandle {
            tx: self.tx.clone(),
            max_retries: self.max_retries,
        }
    }

    fn send_request(handle: &Handle, client: &HyperClient, payload: Payload, timeout: u64) -> Box<Future<Item = (), Error = ()>> {
        let Payload {
            url,
            method,
            body: maybe_body,
            headers: maybe_headers,
            callback,
        } = payload;

        let uri = match url.parse() {
            Ok(val) => val,
            Err(err) => {
                error!("Url `{}` passed to http client cannot be parsed: `{}`", url, err);
                return Box::new(
                    callback
                        .send(Err(Error::Parse(format!("Cannot parse url `{}`", url))))
                        .into_future()
                        .map(|_| ())
                        .map_err(|err| {
                            error!("Failed to send a response to the oneshot callback channel: {:?}", err);
                            ()
                        }),
                );
            }
        };
        let mut req = hyper::Request::new(method.clone(), uri);

        if let Some(headers) = maybe_headers {
            mem::replace(req.headers_mut(), headers);
        }

        for body in maybe_body.iter() {
            req.set_body(body.clone());
        }

        let timeout_duration = Duration::from_millis(timeout);

        let timeout = match tokio_core::reactor::Timeout::new(timeout_duration, handle) {
            Ok(t) => t,
            Err(_) => {
                error!("Could not get timeout for handle.");
                return Box::new(future::err(()));
            }
        };

        let req_task = client.request(req);

        let work = req_task.select2(timeout).then(move |res| match res {
            Ok(Either::A((got, _timeout))) => Ok(got),
            Ok(Either::B((_timeout_error, _get))) => {
                error!(
                    "Client timed out while connecting to {}, using method: {} after: {:?}.",
                    url, method, timeout_duration
                );
                Err(Error::Timeout)
            }
            Err(Either::A((get_error, _timeout))) => Err(Error::Network(get_error)),
            Err(Either::B((timeout_error, _get))) => {
                error!(
                    "Timeout future error occurred while connecting to {}, using method: {} after: {:?}.",
                    url, method, timeout_duration
                );
                Err(Error::Network(From::from(timeout_error)))
            }
        });

        let work_with_timeout = work
            .and_then(move |res| {
                let status = res.status();
                let body_future: Box<Future<Item = String, Error = Error>> = Box::new(read_body(res.body()).map_err(Error::Network));
                match status.as_u16() {
                    200...299 => body_future,

                    _ => Box::new(body_future.and_then(move |body| {
                        let message = serde_json::from_str::<ErrorMessage>(&body).ok();
                        let error = Error::Api(
                            status,
                            message.or_else(|| {
                                Some(ErrorMessage {
                                    code: 422,
                                    description: body,
                                    payload: None,
                                })
                            }),
                        );
                        future::err(error)
                    })),
                }
            })
            .then(|result| callback.send(result))
            .map(|_| ())
            .map_err(|err| {
                error!("Failed to send a response to the oneshot callback channel: {:?}", err);
                ()
            });

        Box::new(work_with_timeout)
    }
}

#[derive(Clone)]
pub struct ClientHandle {
    tx: mpsc::Sender<Payload>,
    max_retries: usize,
}

impl ClientHandle {
    pub fn request_with_auth_header<T>(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        auth_data: Option<String>,
    ) -> Box<Future<Item = T, Error = Error> + Send>
    where
        T: for<'a> Deserialize<'a> + 'static + Send,
    {
        let headers = auth_data.and_then(|s| {
            let mut headers = Headers::new();
            headers.set(Authorization(s));
            Some(headers)
        });
        self.request(method, url, body, headers)
    }

    pub fn request<T>(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
    ) -> Box<Future<Item = T, Error = Error> + Send>
    where
        T: for<'a> Deserialize<'a> + 'static + Send,
    {
        Box::new(self.simple_request(method, url, body, headers).and_then(|response| {
            if response.is_empty() {
                serde_json::from_value(serde_json::Value::Null)
            } else {
                serde_json::from_str::<T>(&response)
            }
            .map_err(|err| Error::Parse(format!("Parsing response {:?} failed with error {}", response, err)))
        }))
    }

    pub fn simple_request(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
    ) -> Box<Future<Item = String, Error = Error> + Send> {
        Box::new(self.send_request_with_retries(method, url, body, headers, None, self.max_retries))
    }

    fn send_request_with_retries(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
        last_err: Option<Error>,
        retries: usize,
    ) -> Box<Future<Item = String, Error = Error> + Send> {
        if retries == 0 {
            let error = last_err.unwrap_or_else(|| Error::Unknown("Unexpected missing error in send_request_with_retries".to_string()));
            Box::new(future::err(error))
        } else {
            let self_clone = self.clone();
            let method_clone = method.clone();
            let body_clone = body.clone();
            let url_clone = url.clone();
            let headers_clone = headers.clone();
            Box::new(self.send_request(method, url, body, headers).or_else(move |err| match err {
                Error::Network(err) => {
                    warn!(
                        "Failed to fetch `{}` with error `{}`, retrying... Retries left {}",
                        url_clone, err, retries
                    );
                    self_clone.send_request_with_retries(
                        method_clone,
                        url_clone,
                        body_clone,
                        headers_clone,
                        Some(Error::Network(err)),
                        retries - 1,
                    )
                }
                _ => Box::new(future::err(err)),
            }))
        }
    }

    fn send_request(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<hyper::Headers>,
    ) -> Box<Future<Item = String, Error = Error> + Send> {
        debug!(
            "Starting outbound http request: {} {} with body {} and headers {}",
            method,
            url,
            body.clone().unwrap_or_default(),
            headers.clone().unwrap_or_default()
        );
        let url_clone = url.clone();
        let method_clone = method.clone();

        let (tx, rx) = oneshot::channel::<ClientResult>();
        let payload = Payload {
            url,
            method,
            body,
            headers,
            callback: tx,
        };

        let future = self
            .tx
            .clone()
            .send(payload)
            .map_err(|err| Error::Unknown(format!("Unexpected error sending http client request params to channel: {}", err)))
            .and_then(|_| {
                rx.map_err(|err| Error::Unknown(format!("Unexpected error receiving http client response from channel: {}", err)))
            })
            .and_then(|result| result)
            .map_err(move |err| {
                error!("{} {} : {}", method_clone, url_clone, err);
                err
            });

        Box::new(future)
    }
}

impl HttpClient for ClientHandle {
    fn request(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
    ) -> Box<Future<Item = Response, Error = Error> + Send> {
        Box::new(self.simple_request(method, url, body, headers).map(Response))
    }
}

impl<T: HttpClient> HttpClient for Box<T> {
    fn request(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<Headers>,
    ) -> Box<Future<Item = Response, Error = Error> + Send> {
        (**self).request(method, url, body, headers)
    }
}

struct Payload {
    pub url: String,
    pub method: hyper::Method,
    pub body: Option<String>,
    pub headers: Option<hyper::Headers>,
    pub callback: oneshot::Sender<ClientResult>,
}

#[derive(Debug, Fail)]
pub enum Error {
    Api(hyper::StatusCode, Option<ErrorMessage>),
    Network(hyper::Error),
    Timeout,
    Parse(String),
    Unknown(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Api(ref status, Some(ref error_message)) => write!(
                f,
                "Http client 100: Api error: status: {}, code: {}, description: {}, payload: {:?}",
                status, error_message.code, error_message.description, error_message.payload
            ),
            Error::Api(status, None) => write!(f, "Http client 100: Api error: status: {}", status),
            Error::Timeout => write!(f, "Http client 200: Network timeoout"),
            Error::Network(ref err) => write!(f, "Http client 200: Network error: {}", err),
            Error::Parse(ref err) => write!(f, "Http client 300: Parse error: {}", err),
            Error::Unknown(ref err) => write!(f, "Http client 400: Unknown error: {}", err),
        }
    }
}

impl Error {
    pub fn into_graphql(self) -> FieldError {
        match self {
            Error::Api(
                status,
                Some(ErrorMessage {
                    code,
                    description,
                    payload,
                }),
            ) => {
                let payload = serde_json::to_string(&payload).unwrap();
                let message = payload.clone();
                let code = code.to_string();
                let status = status.to_string();
                FieldError::new(
                    "Error response from microservice",
                    graphql_value!({ "code": 100, "details": {"status": status, "code": code, "description": description, "message": message, "payload": payload }}),
                )
            }
            Error::Api(status, None) => {
                let status = status.to_string();
                FieldError::new(
                    "Error response from microservice",
                    graphql_value!({ "code": 100, "details": { "status": status }}),
                )
            }
            Error::Network(_) => FieldError::new(
                "Network error for microservice",
                graphql_value!({ "code": 200, "details": { "See server logs for details." }}),
            ),
            Error::Timeout => FieldError::new(
                "Network timeout error for microservice",
                graphql_value!({ "code": 200, "details": { "Client timeout expired." }}),
            ),
            Error::Parse(message) => FieldError::new("Unexpected parsing error", graphql_value!({ "code": 300, "details": { message }})),
            _ => FieldError::new(
                "Unknown error for microservice",
                graphql_value!({ "code": 400, "details": { "See server logs for details." }}),
            ),
        }
    }
}
