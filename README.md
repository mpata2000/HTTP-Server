# HTTP Server

A simple http server

## Usage

```rust
use HTTP_Server::router::Router;
use HTTP_Server::server::Server;

pub fn ping(ctx: &mut Context) {
    ctx.string(HttpStatus::Ok, "pong")
}

fn main(){
  let mut router = Router::new();
  router.get("/ping", ping);

  let server = Server::new(router, None);
  server.start("127.0.0.1:8080").expect("Error starting server");
}
```
