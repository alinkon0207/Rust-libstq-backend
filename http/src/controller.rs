use std;
use std::str;
use std::sync::Arc;

use chrono::prelude::*;
use failure;
use failure::Fail;
use futures::future::{self, Either};
use futures::prelude::*;
use hyper;
use hyper::header::{AccessControlAllowHeaders, AccessControlAllowMethods, AccessControlRequestHeaders, ContentLength, ContentType};
use hyper::server::Service;
use hyper::Method::{Get, Options, Post};
use hyper::{mime, Error, Headers, StatusCode};
use hyper::{Request, Response};
use serde_json;

use log::{self, Level};

use request_util::{get_correlation_token, try_read_body};

use errors::*;
use system::{SystemService, SystemServiceImpl};

pub type ControllerFuture = Box<Future<Item = String, Error = failure::Error>>;

/// The meat of your application. Best used with RouteParser in utils.
pub trait Controller {
    fn call(&self, request: Request) -> ControllerFuture;
}

pub type ServerFuture = Box<Future<Item = Response, Error = hyper::Error>>;

/// Batteries-included Service for Hyper HTTP server. Feed it your Controller and it'll adapt it for Hyper.
pub struct Application<E: Fail + Codeable + PayloadCarrier> {
    pub controller: Arc<dyn Controller>,
    pub system_service: Box<SystemService>,
    pub middleware: Arc<Fn(Response) -> Response>,
    _error_type: std::marker::PhantomData<E>,
}

impl<E> Service for Application<E>
where
    E: Fail + Codeable + PayloadCarrier,
{
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = ServerFuture;

    fn call(&self, req: Request) -> ServerFuture {
        let call_start = Local::now();

        let correlation_token = get_correlation_token(&req);

        Box::new(
            match *req.method() {
                Options => {
                    let req_headers = req.headers().clone();
                    let acah = req_headers.get::<AccessControlRequestHeaders>();

                    let mut resp = Response::new();
                    let mut new_headers = Headers::new();
                    new_headers.set(AccessControlAllowMethods(vec![Get, Post, Options]));
                    if let Some(a) = acah {
                        new_headers.set(AccessControlAllowHeaders(a.to_vec()));
                    };
                    new_headers.set(ContentType(mime::TEXT_HTML));
                    std::mem::replace(resp.headers_mut(), new_headers);

                    Box::new(future::ok(resp)) as ServerFuture
                }
                _ => {
                    let token = correlation_token.clone();

                    Box::new(
                        match req.uri().path() {
                            "/healthcheck" => {
                                Box::new(self.system_service.healthcheck().then(|res| {
                                    let response = match res {
                                        Ok(data) => Self::response_with_json(data.clone()),
                                        Err(err) => Self::response_with_error(&err),
                                    };

                                    future::ok(response)
                                })) as ServerFuture
                            },
                            _ => {
                                let controller = self.controller.clone();
                                let level = log::max_level();

                                let fut = if level == Level::Debug || level == Level::Trace {
                                    let (method, uri, http_version, headers, body) = req.deconstruct();
                                    Either::A(
                                        try_read_body(body)
                                            .map_err(From::from)
                                            .and_then(move |bytes| {
                                                {
                                                    let body_log = match str::from_utf8(&bytes) {
                                                        Ok(data) => data,
                                                        Err(_) => "`can not parse body to string`",
                                                    };
                                                    debug!(
                                                        "Server received Request, method: {}, url: {}, headers: {:#?}, body: {}, correlation token: {}",
                                                        method, uri, headers, body_log, token
                                                    );
                                                }

                                                let mut req = Request::new(method, uri);
                                                req.set_body(bytes);
                                                req.set_version(http_version);
                                                std::mem::replace(req.headers_mut(), headers);

                                                Ok(req)
                                            }).and_then(move |req| controller.call(req)),
                                    )
                                } else {
                                    Either::B(self.controller.call(req))
                                };

                                Box::new(fut.then({
                                let token = correlation_token.clone();

                                move |res| {
                                let (response, body) = match res {
                                    Ok(data) => (Self::response_with_json(data.clone()), data),
                                    Err(err) => (Self::response_with_error(&err), Self::error_to_body(&err)),
                                };

                                let dt = Local::now() - call_start;
                                debug!(
                                    "Server send Response, status: {}, headers: {:#?}, body: {:?}, elapsed time = {}.{:03}, correlation token: {}",
                                    response.status().as_u16(),
                                    response.headers(),
                                    body,
                                    dt.num_seconds(),
                                    dt.num_milliseconds(),
                                    token
                                );

                                future::ok(response)
                            }
                        }))
                        }
                    }) as ServerFuture
                }
            }.map({
                let middleware = self.middleware.clone();
                move |resp| middleware(resp)
            }),
        )
    }
}

impl<E> Application<E>
where
    E: Fail + Codeable + PayloadCarrier,
{
    pub fn new<T>(controller: T) -> Self
    where
        T: Controller + 'static,
    {
        Self {
            controller: Arc::new(controller),
            middleware: Arc::new(|resp| resp),
            system_service: Box::new(SystemServiceImpl::default()),
            _error_type: Default::default(),
        }
    }

    /// Replaces controller in the application
    pub fn with_controller<T>(mut self, controller: T) -> Self
    where
        T: Controller + 'static,
    {
        self.controller = Arc::new(controller);
        self
    }

    /// Installs custom healthcheck handler
    pub fn with_system_service<T>(mut self, system_service: T) -> Self
    where
        T: SystemService + 'static,
    {
        self.system_service = Box::new(system_service);
        self
    }

    /// Installs custom middleware called for each response
    pub fn with_middleware<F>(mut self, f: F) -> Self
    where
        F: Fn(Response) -> Response + 'static,
    {
        self.middleware = Arc::new(f);
        self
    }

    /// Responds with success, logs response body
    fn response_with_json(body: String) -> Response {
        Self::response_with_body(body).with_status(StatusCode::Ok)
    }

    /// Responds with JSON error, logs response body
    fn response_with_error(error: &failure::Error) -> Response {
        trace!("Trace: {}", error.backtrace());
        let error_data = ErrorMessageWrapper::<E>::from(&error).inner;
        error!("Description: \"{}\". Payload: {:?}", error_data.description, error_data.payload);
        let mes = serde_json::to_string(&error_data).unwrap();
        Self::response_with_body(mes).with_status(hyper::StatusCode::try_from(error_data.code).unwrap())
    }

    fn error_to_body(error: &failure::Error) -> String {
        let error_data = ErrorMessageWrapper::<E>::from(&error).inner;

        serde_json::to_string(&error_data).unwrap()
    }

    fn response_with_body(body: String) -> Response {
        Response::new()
            .with_header(ContentLength(body.len() as u64))
            .with_header(ContentType(mime::APPLICATION_JSON))
            .with_body(body)
    }
}
