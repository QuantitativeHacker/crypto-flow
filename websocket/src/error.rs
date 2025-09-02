use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    WebSocketError(String),
    AuthenticationError(String),
    JsonError(serde_json::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WebSocketError(s) => write!(f, "WebSocketError: {}", s),
            Error::AuthenticationError(s) => write!(f, "AuthenticationError: {}", s),
            Error::JsonError(e) => write!(f, "JsonError: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError(value)
    }
}
