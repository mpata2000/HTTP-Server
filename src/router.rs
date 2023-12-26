use std::collections::HashMap;

use super::{
    context::Context, http_method::HttpMethod, http_request::HttpRequest, http_status::HttpStatus,
};

#[derive(Debug, Clone)]
pub struct Route {
    pub method: HttpMethod,
    pub path: Vec<String>,
    pub handler: Handler,
}

type Handler = fn(ctx: &mut Context);
impl Route {
    pub fn new(method: HttpMethod, path: &str, handler: Handler) -> Route {
        let path = path.trim_end_matches("/").trim_start_matches("/");
        let path = path.split("/").map(|p| p.to_string()).collect();
        Route {
            method,
            path,
            handler,
        }
    }

    /// Compare the route at the index with the path
    /// if the route at the index is equal to the path return true
    /// if the route at the index is a param return true
    /// otherwise return false
    /// # Example
    /// ```
    /// use HTTP_Server::context::Context;
    /// use HTTP_Server::router::Route;
    /// use HTTP_Server::http_method::HttpMethod;
    ///
    /// fn handler(ctx: &mut Context) {}
    ///
    /// let route = Route::new(HttpMethod::Get, "/test/{param}", handler);
    /// assert!(route.compare_path_at("test", 0));
    /// assert!(route.compare_path_at("any", 1)); // the route has a param at the index 1
    /// assert!(!route.compare_path_at("not", 0));
    /// assert!(!route.compare_path_at("test", 2)); // the route has only two parts
    /// ```
    pub fn compare_path_at(&self, route: &str, index: usize) -> bool {
        if self.path.len() <= index {
            return false;
        }

        if self.path[index].starts_with("{") && self.path[index].ends_with("}") {
            return true;
        }

        self.path[index] == route
    }

    /// Returns the number of matches between the route and the path
    /// # Example
    /// ```
    /// use HTTP_Server::context::Context;
    /// use HTTP_Server::http_method::HttpMethod;
    /// use HTTP_Server::router::Route;
    ///
    /// fn handler(ctx: &mut Context) {}
    ///
    /// let route = Route::new(HttpMethod::Get, "/test/new", handler);
    /// assert_eq!(route.matches(&["test", "new"]), 2);
    /// assert_eq!(route.matches(&["test", "new", "other"]), 2);
    /// assert_eq!(route.matches(&["test", "other"]), 1);
    /// ```
    pub fn matches(&self, path: &[&str]) -> usize {
        let mut matches = 0;
        for (i, p) in self.path.iter().enumerate() {
            if let Some(s) = path.get(i) {
                if s == p {
                    matches += 1;
                }
            }
        }
        matches
    }

    /// Set the path params in the context
    pub fn set_path_params(&self, path: &[&str], ctx: &mut Context) {
        let mut params = HashMap::new();
        for (i, p) in path.iter().enumerate() {
            if self.path[i].starts_with("{") && self.path[i].ends_with("}") {
                params.insert(
                    self.path[i]
                        .trim_start_matches("{")
                        .trim_end_matches("}")
                        .to_string(),
                    p.to_string(),
                );
            }
        }
        ctx.path_params = params;
    }
}

pub struct Router {
    pub routes: Vec<Route>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Router {
        Router { routes: Vec::new() }
    }

    /// Add a new get route to the router
    /// # Example
    /// ```
    /// use HTTP_Server::context::Context;
    /// use HTTP_Server::router::Router;
    ///
    /// fn handler(ctx: &mut Context) {}
    ///
    /// let mut router = Router::new();
    /// router.get("/test", handler);
    /// ```
    pub fn get(&mut self, path: &str, handler: Handler) -> &mut Self {
        self.routes.push(Route::new(HttpMethod::Get, path, handler));
        self
    }

    /// Add a new post route to the router
    /// # Example
    /// ```
    /// use HTTP_Server::router::Router;
    /// use HTTP_Server::context::Context;
    ///
    /// fn handler(ctx: &mut Context) {}
    ///
    /// let mut router = Router::new();
    /// router.post("/test", handler);
    /// ```
    pub fn post(&mut self, path: &str, handler: Handler) -> &mut Self {
        self.routes
            .push(Route::new(HttpMethod::Post, path, handler));
        self
    }

    pub fn put(&mut self, path: &str, handler: Handler) -> &mut Self {
        self.routes.push(Route::new(HttpMethod::Put, path, handler));
        self
    }

    pub fn delete(&mut self, path: &str, handler: Handler) -> &mut Self {
        self.routes
            .push(Route::new(HttpMethod::Delete, path, handler));
        self
    }

    pub fn patch(&mut self, path: &str, handler: Handler) -> &mut Self {
        self.routes
            .push(Route::new(HttpMethod::Patch, path, handler));
        self
    }

    /// Get the route that matches the method and path
    fn get_route(&self, method: HttpMethod, path: &[&str]) -> Option<Route> {
        let mut r = self.routes.clone();
        r.retain(|r| r.method == method && r.path.len() == path.len());
        for (i, p) in path.iter().enumerate() {
            r.retain(|r| r.compare_path_at(p, i));
            if r.is_empty() {
                return None;
            }
        }
        // get the route with the most matches
        r.iter().max_by(|a, b| a.matches(path).cmp(&b.matches(path))).cloned()
    }

    /// Route the request to the appropriate handler
    pub fn handle_request(&self, ctx: &mut Context) {
        let path = ctx.request.clone().path;
        let path: Vec<&str> = path
            .trim_end_matches("/")
            .trim_start_matches("/")
            .split("/")
            .collect();
        let route = self.get_route(ctx.request.method, &path);

        if let Some(route) = route {
            route.set_path_params(&path, ctx);
            (route.handler)(ctx);
        } else {
            ctx.string(HttpStatus::NotFound, "Not Found");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::http_method::HttpMethod;
    use crate::http_request::HttpRequest;

    fn dummy_handler(_ctx: &mut Context) {}

    #[test]
    fn test_router_get_route() {
        let mut router = Router::new();
        router.get("/test", dummy_handler);
        let route = router.get_route(HttpMethod::Get, &["test"]);
        assert!(route.is_some());
        assert_eq!(route.unwrap().path, vec!["test".to_string()]);
    }

    #[test]
    fn test_router_get_empty_route() {
        let mut router = Router::new();
        let path = "/".to_string();
        router.get(&path, dummy_handler);
        let path: Vec<&str> = path
            .trim_end_matches("/")
            .trim_start_matches("/")
            .split("/")
            .collect();
        let route = router.get_route(HttpMethod::Get, &path);
        assert!(route.is_some());
        assert_eq!(route.unwrap().path, vec!["".to_string()]);
    }

    #[test]
    fn test_router_get_route_not_found() {
        let mut router = Router::new();
        router.get("/test", dummy_handler);
        let route = router.get_route(HttpMethod::Get, &["test", "1"]);
        assert!(route.is_none());
    }

    #[test]
    fn test_router_get_with_params() {
        let mut router = Router::new();
        router.get("/test/{param}", dummy_handler);
        let route = router.get_route(HttpMethod::Get, &["test", "1"]);
        assert!(route.is_some());
        assert_eq!(
            route.unwrap().path,
            vec!["test".to_string(), "{param}".to_string()]
        );
    }

    #[test]
    fn test_router_get_with_params_many_routes() {
        let mut router = Router::new();
        router.get("/test/{param}", dummy_handler);
        router.get("/test/test", dummy_handler);
        let route = router.get_route(HttpMethod::Get, &["test", "test"]);
        assert!(route.is_some());
        assert_eq!(
            route.unwrap().path,
            vec!["test".to_string(), "test".to_string()]
        );
    }

    #[test]
    fn test_router_get_with_params_not_found() {
        let mut router = Router::new();
        router.get("/test/{param}", dummy_handler);
        let route = router.get_route(HttpMethod::Get, &["test", "1", "2"]);
        assert!(route.is_none());
    }

    #[test]
    fn test_router_get_mathces_start_but_not_end() {
        let mut router = Router::new();
        router.get("/test/1/2/3/4", dummy_handler);
        let route = router.get_route(HttpMethod::Get, &["test", "1", "2", "3", "4", "5"]);
        assert!(route.is_none());
    }

    #[test]
    fn test_router_get_with_multiple_routes() {
        let mut router = Router::new();
        router
            .get("/test/1/2/", dummy_handler)
            .post("/test/1/2/", dummy_handler)
            .get("/test/{param}", dummy_handler)
            .post("/test/{param}/test", dummy_handler);

        let get1 = router.get_route(HttpMethod::Get, &["test", "1", "2"]);
        let get2 = router.get_route(HttpMethod::Get, &["test", "1"]);
        let post1 = router.get_route(HttpMethod::Post, &["test", "1", "2"]);
        let post2 = router.get_route(HttpMethod::Post, &["test", "1", "test"]);
        let not_found = router.get_route(HttpMethod::Get, &["new"]);
        assert!(get1.is_some());
        assert!(get2.is_some());
        assert!(post1.is_some());
        assert!(post2.is_some());
        assert!(not_found.is_none());
    }

    #[test]
    fn test_route_compare_path_at() {
        let route = Route::new(HttpMethod::Get, "/test/{param}", dummy_handler);
        assert!(route.compare_path_at("test", 0));
        assert!(route.compare_path_at("any", 1)); // the route has a param at the index 1
        assert!(!route.compare_path_at("not", 0));
        assert!(!route.compare_path_at("test", 2)); // the route has only two parts
    }

    #[test]
    fn test_route_get_path_params() {
        let route = Route::new(HttpMethod::Get, "/test/{param}", dummy_handler);
        let path = vec!["test", "1"];
        let mut ctx = Context::new(Vec::new());
        ctx.request =
            HttpRequest::new(HttpMethod::Get, "/test/1".into(), HashMap::new(), "".into());
        route.set_path_params(&path, &mut ctx);
        assert_eq!(ctx.param("param"), Some("1".to_string()));
    }
}
