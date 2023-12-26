use crate::http_request::HttpRequest;
use crate::http_status::HttpStatus;
use serde_json::{json, Value};
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Display;
use std::io;
use std::sync::mpsc::Sender;

const HTTP_VERSION: &str = "HTTP/1.1";

type Writer = dyn io::Write;

pub struct Context {
    pub request: HttpRequest,
    pub logger: Option<Sender<String>>,
    writer: Box<Writer>,
    response_headers: HashMap<String, String>,
    pub(crate) path_params: HashMap<String, String>,
}

impl Context {
    pub fn new<W: io::Write + 'static>(writer: W) -> Context {
        Context {
            request: HttpRequest::empty(),
            logger: None,
            writer: Box::new(writer),
            path_params: HashMap::new(),
            response_headers: HashMap::new(),
        }
    }

    pub fn add_response_header<K: Display, V: Display>(&mut self, k: K, v: V) {
        self.response_headers.insert(k.to_string(), v.to_string());
    }

    /// Send a json response to the client
    /// If the body is a Value type it will be sent as is
    /// otherwise it will be wrapped in a json object with the status and body keys like
    /// ```json
    /// {
    ///    "status": "200 OK",
    ///    "body": "Hello World"
    /// }
    pub fn json<T: Display + 'static>(&mut self, status: HttpStatus, body: T) {
        let mut r;
        if TypeId::of::<T>() == TypeId::of::<Value>() {
            r = body.to_string();
        } else {
            r = json!({"status": status.to_string(), "body": body.to_string()}).to_string()
        }

        self.add_response_header("Content-Type", "application/json");
        self.add_response_header("Content-Length", r.len());
        self.send_response(status, &r)
    }

    /// Send a string response to the client
    pub fn string(&mut self, status: HttpStatus, body: &str) {
        self.add_response_header("Content-Type", "text/plain");
        self.add_response_header("Content-Length", body.len());
        self.send_response(status, body)
    }

    fn send_response(&mut self, status: HttpStatus, body: &str) {
        let mut response = format!("{HTTP_VERSION} {status}\r\n");
        response += &self
            .response_headers
            .iter()
            .map(|(key, value)| format!("{}: {}\r\n", key, value))
            .collect::<String>();

        response += "\r\n";

        if let Some(size) = self.response_headers.get("Content-Length") {
            if size != "0" {
                response += body;
            }
        }

        if let Err(e) = self.writer.write(response.as_bytes()) {
            println!("Error writing response: {}", e);
        }
    }

    pub fn param(&self, key: &str) -> Option<String> {
        self.path_params.get(key).cloned()
    }

    pub fn header(&self, key: &str) -> Option<String> {
        self.request.headers.get(key).cloned()
    }

    pub fn body(&self) -> String {
        self.request.body.clone()
    }
}
