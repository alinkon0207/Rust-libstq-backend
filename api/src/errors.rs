use hyper;
use serde_json::Value;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage {
    pub code: u16,
    pub description: String,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Fail)]
pub enum Error {
    Api(hyper::StatusCode, Option<ErrorMessage>),
    Network(String),
    Parse(String),
    Unknown(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Api(ref status, Some(ref error_message)) => write!(
                f,
                "API client 100: Api error: status: {}, code: {}, description: {}, payload: {:?}",
                status, error_message.code, error_message.description, error_message.payload
            ),
            Error::Api(status, None) => write!(f, "API client 100: Api error: status: {}", status),
            Error::Network(ref err) => write!(f, "API client 200: Network error: {}", err),
            Error::Parse(ref err) => write!(f, "API client 300: Parse error: {}", err),
            Error::Unknown(ref err) => write!(f, "API client 400: Unknown error: {}", err),
        }
    }
}
