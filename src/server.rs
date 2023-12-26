use crate::api_err::ApiErr;
use crate::http_method::HttpMethod;
use crate::http_status::HttpStatus;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::mpsc::Sender;
use std::{
    io,
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use crate::utils::thread_pool::ThreadPool;

use super::{context::Context, http_request::HttpRequest, router::Router};

const MAX_THREADS: usize = 40;

pub struct Server {
    pub router: Arc<Router>,
    pub pool: ThreadPool,
    pub logger: Option<Sender<String>>,
}

impl Server {
    pub fn new(router: Router, logger: Option<Sender<String>>) -> Server {
        let threads = (router.routes.len() * 5).min(MAX_THREADS);
        Server {
            router: Arc::new(router),
            pool: ThreadPool::new(threads),
            logger,
        }
    }

    /// Starts the server on the specified address.
    pub(crate) fn start(&self, addr: &str) -> io::Result<()> {
        let listener = TcpListener::bind(addr)?;
        println!("Server listening on port {}", addr);
        for stream in listener.incoming() {
            let mut stream = stream?;
            let router = Arc::clone(&self.router);
            let logger = self.logger.clone();

            // Submit the connection handling task to the thread pool
            self.pool.execute(move || {
                match Server::handle_connection(&mut stream) {
                    Ok(request) => {
                        let mut ctx = Context::new(stream);
                        // Handle the request in the router layer
                        ctx.request = request;
                        ctx.logger = logger;
                        router.handle_request(&mut ctx);
                    }
                    Err(e) => {
                        let mut ctx = Context::new(stream);
                        if let Some(logger) = logger {
                            _ = logger.send(e.to_string());
                        }
                        ctx.string(HttpStatus::BadRequest, &e.to_string());
                    }
                }
            });
        }

        Ok(())
    }

    fn read_head<S: Read>(mut stream: &mut S) -> Result<String, ApiErr> {
        let mut buffer = Vec::new();
        let mut buf = [0; 1];

        loop {
            stream.read_exact(&mut buf).map_err(ApiErr::StreamError)?;
            buffer.push(buf[0]);
            if buffer.ends_with(b"\r\n\r\n") {
                // Read until double newline is encountered
                break;
            }
        }

        let head = String::from_utf8_lossy(&buffer);
        Ok(head.trim().to_string())
    }

    fn handle_connection<S: Read>(mut stream: &mut S) -> Result<HttpRequest, ApiErr> {
        let head = Server::read_head(&mut stream)?;
        let mut head_lines = head.split("\r\n").collect::<Vec<&str>>();
        let start_line = head_lines
            .remove(0)
            .split_whitespace()
            .collect::<Vec<&str>>();
        let verb = start_line.get(0).ok_or(ApiErr::InvalidRequest)?;
        let path = start_line.get(1).ok_or(ApiErr::InvalidRequest)?;
        let mut headers: HashMap<String, String> = HashMap::new();
        for line in &head_lines {
            let (key, value) = match line.split_once(":") {
                Some((key, value)) => (key, value),
                None => continue,
            };
            headers.insert(key.to_string(), value.trim().to_string());
        }

        let mut body = String::new();
        if let Some(content_length) = headers.get("Content-Length") {
            let content_length = content_length
                .parse::<usize>()
                .map_err(|_| ApiErr::InvalidRequest)?;
            let mut buff = vec![0; content_length];
            stream.read_exact(&mut buff).map_err(ApiErr::StreamError)?;
            body = String::from_utf8_lossy(&buff).to_string();
        }

        Ok(HttpRequest::new(
            HttpMethod::from_string(verb)?,
            path.to_string(),
            headers,
            body,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::mock_stream::MockTcpStream;

    #[test]
    fn handle_message_without_body() {
        let bytes = b"GET / HTTP/1.1\r\nHost: localhost:8080\r\n\r\n";
        let mut stream = MockTcpStream {
            read_data: bytes.to_vec(),
            position: 0,
            write_data: vec![],
        };

        let request = Server::handle_connection(&mut stream).unwrap();
        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.path, "/");
        assert_eq!(request.headers.len(), 1);
        assert_eq!(
            request.headers.get("Host"),
            Some(&"localhost:8080".to_string())
        );
        assert_eq!(request.body, "");
    }

    #[test]
    fn handle_message_with_body() {
        let bytes = b"POST / HTTP/1.1\r\nHost: localhost:8080\r\nContent-Length: 5\r\nContent-Type: text/plain\r\n\r\nHello";
        let mut stream = MockTcpStream {
            read_data: bytes.to_vec(),
            position: 0,
            write_data: vec![],
        };

        let request = Server::handle_connection(&mut stream).unwrap();
        assert_eq!(request.method, HttpMethod::Post);
        assert_eq!(request.path, "/");
        assert_eq!(request.headers.len(), 3);
        assert_eq!(
            request.headers.get("Host"),
            Some(&"localhost:8080".to_string())
        );
        assert_eq!(
            request.headers.get("Content-Length"),
            Some(&"5".to_string())
        );
        assert_eq!(
            request.headers.get("Content-Type"),
            Some(&"text/plain".to_string())
        );
        assert_eq!(request.body, "Hello");
    }

    #[test]
    fn handle_message_read_up_to_content_length() {
        let bytes = b"POST / HTTP/1.1\r\nHost: localhost:8080\r\nContent-Length: 3\r\nContent-Type: text/plain\r\n\r\nHello";
        let mut stream = MockTcpStream {
            read_data: bytes.to_vec(),
            position: 0,
            write_data: vec![],
        };

        let request = Server::handle_connection(&mut stream).unwrap();
        assert_eq!(request.method, HttpMethod::Post);
        assert_eq!(request.path, "/");
        assert_eq!(request.headers.len(), 3);
        assert_eq!(
            request.headers.get("Host"),
            Some(&"localhost:8080".to_string())
        );
        assert_eq!(
            request.headers.get("Content-Length"),
            Some(&"3".to_string())
        );
        assert_eq!(
            request.headers.get("Content-Type"),
            Some(&"text/plain".to_string())
        );
        assert_eq!(request.body, "Hel");
    }

    #[test]
    fn handle_message_read_up_to_content_length_shorter_than_body() {
        let bytes = b"POST / HTTP/1.1\r\nHost: localhost:8080\r\nContent-Length: 3\r\nContent-Type: text/plain\r\n\r\nHello";
        let mut stream = MockTcpStream {
            read_data: bytes.to_vec(),
            position: 0,
            write_data: vec![],
        };

        let request = Server::handle_connection(&mut stream).unwrap();
        assert_eq!(request.method, HttpMethod::Post);
        assert_eq!(request.path, "/");
        assert_eq!(request.headers.len(), 3);
        assert_eq!(
            request.headers.get("Host"),
            Some(&"localhost:8080".to_string())
        );
        assert_eq!(
            request.headers.get("Content-Length"),
            Some(&"3".to_string())
        );
        assert_eq!(
            request.headers.get("Content-Type"),
            Some(&"text/plain".to_string())
        );
        assert_eq!(request.body, "Hel");
    }
}
