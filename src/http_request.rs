use crate::http_method::HttpMethod;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub(crate) method: HttpMethod,
    pub(crate) path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    pub(crate) fn empty() -> HttpRequest {
        HttpRequest {
            method: HttpMethod::Get,
            path: String::new(),
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    /// Creates a new `HttpRequest` instance with the specified parameters.
    pub fn new(
        method: HttpMethod,
        path: String,
        headers: HashMap<String, String>,
        body: String,
    ) -> HttpRequest {
        HttpRequest {
            method,
            path,
            headers,
            body,
        }
    }
}
