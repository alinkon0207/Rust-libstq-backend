use failure::{Context, Error, Fail};
use hyper::StatusCode;
use serde_json::Value;
use std;

pub trait Codeable {
    fn code(&self) -> StatusCode;
}

pub trait PayloadCarrier {
    fn payload(&self) -> Option<Value>;
}

pub struct ErrorMessageWrapper<E: Fail + Codeable> {
    pub inner: ErrorMessage,
    _type: std::marker::PhantomData<E>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorMessage {
    pub code: u16,
    pub description: String,
    pub payload: Option<Value>,
}

impl<E> ErrorMessageWrapper<E>
where
    E: Fail + Codeable + PayloadCarrier,
{
    pub fn from(e: &Error) -> Self {
        let description = e.iter_chain().fold(String::new(), |mut acc, real_err| {
            if !acc.is_empty() {
                acc += " | ";
            }
            acc += &real_err.to_string();
            acc
        });

        let mut code = 500;
        let mut payload = None;

        for cause in e.iter_chain() {
            let real_err = if let Some(ctx) = cause.downcast_ref::<Context<E>>() {
                Some(ctx.get_context())
            } else {
                cause.downcast_ref::<E>()
            };

            if let Some(e) = real_err {
                code = e.code().as_u16();
                payload = e.payload();
                break;
            }
        }

        Self {
            inner: ErrorMessage {
                code,
                description,
                payload,
            },
            _type: Default::default(),
        }
    }
}
