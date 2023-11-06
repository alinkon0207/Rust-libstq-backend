use errors::*;

use failure;
use futures::{future, prelude::*};
use reqwest::async::{Decoder, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};
use serde_json;

pub fn serialize_payload<T>(v: T) -> impl Future<Item = String, Error = failure::Error>
where
    T: Serialize,
{
    future::result(serde_json::to_string(&v).map_err(failure::Error::from))
}

/// Reads body of request and response in Future format
fn read_body(body: Decoder) -> Box<Future<Item = String, Error = Error> + Send> {
    Box::new(
        body.map_err(|e| Error::Network(format!("{:?}", e)))
            .fold(Vec::new(), |mut acc, chunk| {
                acc.extend_from_slice(&*chunk);
                future::ok::<_, Error>(acc)
            })
            .and_then(|bytes| match String::from_utf8(bytes) {
                Ok(data) => future::ok(data),
                Err(err) => future::err(Error::Parse(format!(
                    "Failed to parse data as string: {}",
                    err
                ))),
            }),
    )
}

pub fn http_req<T>(b: RequestBuilder) -> Box<Future<Item = T, Error = Error> + Send>
where
    T: DeserializeOwned + Send + 'static,
{
    Box::new(
        b.send()
            .map_err(|e| {
                if e.is_http() || e.is_redirect() {
                    return Error::Network(format!("{:?}", e));
                }

                if let Some(status) = e.status() {
                    return Error::Api(status, None);
                }

                Error::Unknown(format!("{:?}", e))
            })
            .and_then(|mut rsp| {
                let status = rsp.status();
                match status.as_u16() {
                    200...299 => Box::new(
                        rsp.json::<T>()
                            .map_err(|e| Error::Parse(format!("{:?}", e))),
                    )
                        as Box<Future<Item = T, Error = Error> + Send>,
                    _ => Box::new(read_body(rsp.into_body()).then(move |res| {
                        future::result(match res {
                            Err(e) => Err(Error::Network(format!("{:?}", e))),
                            Ok(s) => Err(Error::Api(
                                status,
                                Some(serde_json::from_str(&s).unwrap_or_else(|_| ErrorMessage {
                                    code: 422,
                                    description: s,
                                    payload: None,
                                })),
                            )),
                        })
                    })),
                }
            }),
    )
}

pub trait RouteBuilder {
    fn route(&self) -> String;

    fn build_route(&self, base: Option<&AsRef<str>>) -> String {
        {
            format!(
                "{}{}",
                match base {
                    Some(url) => format!("{}/", url.as_ref()),
                    None => "".to_string(),
                },
                self.route()
            )
        }
    }
}
