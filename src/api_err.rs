use crate::http_status::HttpStatus;
use serde_json::{json, Value};
use std::{fmt, io};

#[derive(Debug)]
pub enum ApiErr {
    InternalError(String),
    InvalidMethod,
    MediaTypeNotSupported,
    StreamError(io::Error),
    Conflict(String),
    InvalidRequest,
}

impl ApiErr {
    pub fn http_status(&self) -> HttpStatus {
        match self {
            ApiErr::StreamError(_) => HttpStatus::InternalServerError,
            ApiErr::InternalError(_) => HttpStatus::InternalServerError,
            ApiErr::MediaTypeNotSupported => HttpStatus::BadRequest,
            ApiErr::InvalidMethod => HttpStatus::BadRequest,
            ApiErr::Conflict(_) => HttpStatus::Conflict,
            ApiErr::InvalidRequest => HttpStatus::BadRequest,
        }
    }

    pub fn to_value(&self) -> Value {
        let message = self.to_string();
        json!({
            "message": message,
        })
    }
}

impl fmt::Display for ApiErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error = match self {
            ApiErr::StreamError(err) => err.to_string(),
            ApiErr::InternalError(err) => err.clone(),
            ApiErr::MediaTypeNotSupported => "Media type not supported.".into(),
            ApiErr::InvalidMethod => "Invalid method.".into(),
            ApiErr::Conflict(err) => format!("{err} already exists!"),
            ApiErr::InvalidRequest => "Invalid request.".into(),
        };
        write!(f, "{error}")
    }
}
