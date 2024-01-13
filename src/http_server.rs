use std::{ops::Add, io::{BufReader, BufRead, Write}};
use regex::Regex;

use std::net::{TcpListener, TcpStream};

use crate::{http_response::HttpResponse, http_request::HttpRequest, http_input_stream::HttpInputStream};


pub struct Route {
    pub regex_path: String,
    pub path: String,
    pub callback: Box<dyn FnMut(HttpRequest) -> HttpResponse + 'static>
}

pub struct HttpServer {
    pub get_routes: Vec<Route>
}


impl HttpServer {
    pub fn new() -> Self {
        Self {
            get_routes: Vec::new()
        }
    }

    pub fn get(&mut self, path: &str, callback: Box<dyn FnMut(HttpRequest) -> HttpResponse + 'static>) {
        let match_url_param_pattern = r":(\w+)";

        let match_url_param_regex = Regex::new(match_url_param_pattern).unwrap();

        // /users/([^/]+)/books/([^/]+)
        let key: std::borrow::Cow<'_, str> = match_url_param_regex.replace_all(path, r"([^/]+)");

        self.get_routes.push(Route{
            path: String::from(path),
            regex_path: String::from(key).add(r#"(?:\?([^#]+))?(?:#(.*))?$"#),
            callback,
        });
    }

    pub fn match_route(&mut self, resource: &HttpInputStream) -> Option<&mut Route> {
        for route in &mut self.get_routes {
            let regex = Regex::new(&route.regex_path).unwrap();

            if regex.is_match(&resource.relative_path) {
                return Some(route)
            }
        }

        return None;
    }

    pub fn bind(&mut self, addr: &str){
        let listener = match TcpListener::bind(addr) {
            Ok(listener) => {
                println!("Servidor rodando: {}", addr);
                listener
            },
            Err(err) => {
                panic!("Nao foi possível subir o servidor: {}", err)
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut reader = BufReader::new(stream.try_clone().expect("Erro ao tentar clonar stream"));

                    let mut request = String::new();

                    reader.read_line(&mut request).expect("Erro ao ler requição");

                    let stream_parsed = HttpInputStream::parse(&request);
                    let route_matched = self.match_route(&stream_parsed);

                    let response = match route_matched {
                        Some(route) => {
                            let request = HttpRequest::parse(&stream_parsed, &route);
                            let response = (route.callback)(request);

                            format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Lenght: {}\r\n\r\n{}",  response.body.len(), response.body)
                        },
                        None => {
                            "HTTP/1.1 400 BAD REQUEST\r\n\r\n400 Bad Request".to_string()
                        }
                    };

                    if let Err(err) = stream.write_all(response.as_bytes()) {
                        eprintln!("Erro ao gravar a resposta: {}", err)
                    }
                },
                Err(e) => eprintln!("Erro ao aceitar a conexão: {}", e),
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{http_input_stream::HttpInputStream, http_request::HttpRequest, http_response::HttpResponse, http_server::HttpServer};

    #[test]
    fn it_inject_request_struct_in_callback_function(){
        let mut server = HttpServer::new();

        server.get("/users", Box::new(|_| {
            let mut response = HttpResponse::new();
            response.set_status(http::StatusCode::OK);
            response.set_body(String::from("Hello world"));

            response
        }));


        let input_stream = "GET /users HTTP/1.1";
        let input_stream_parsed = HttpInputStream::parse(input_stream);
        let route_matched = server.match_route(&input_stream_parsed).unwrap();
        let request = HttpRequest::parse(&input_stream_parsed, &route_matched);
        let response = (route_matched.callback)(request);

        assert_eq!(response.status, http::StatusCode::OK);
        assert_eq!(response.body, "Hello world");
    } 

    #[test]
    fn it_parse_params_request(){
        let mut request = HttpServer::new();

        request.get("/users", Box::new(|_|{ HttpResponse::new() }));
        request.get("/users/:userId", Box::new(|_|{ HttpResponse::new() }));
        request.get("/users/:userId/books", Box::new(|_|{ HttpResponse::new() }));
        request.get("/users/:userId/books/:bookId", Box::new(|_|{ HttpResponse::new() }));

        let input = HttpInputStream::parse("GET /users HTTP/1.1");
        let route_matched = request.match_route(&input);
        assert!(route_matched.is_some());
        assert_eq!(route_matched.unwrap().path, "/users");

        let input = HttpInputStream::parse("GET /users/10 HTTP/1.1");
        let route_matched = request.match_route(&input);
        assert!(route_matched.is_some());
        assert_eq!(route_matched.unwrap().path, "/users/:userId");

        let input = HttpInputStream::parse("GET /users/10/books HTTP/1.1");
        let route_matched = request.match_route(&input);
        assert!(route_matched.is_some());
        assert_eq!(route_matched.unwrap().path, "/users/:userId/books");
    }

    #[test]
    fn it_parse_query_request(){
        let mut request = HttpServer::new();

        request.get("/users", Box::new(|_|{ HttpResponse::new() }));
        request.get("/users/:userId", Box::new(|_|{ HttpResponse::new() }));
        request.get("/users/:userId/books", Box::new(|_|{ HttpResponse::new() }));
        request.get("/users/:userId/books/:bookId", Box::new(|_|{ HttpResponse::new() }));

        let input = HttpInputStream::parse("GET /users?active=true&teste=[2,3,5] HTTP/1.1");
        let route_matched = request.match_route(&input);
        assert!(route_matched.is_some());
        assert_eq!(route_matched.unwrap().path, "/users");


        let input = HttpInputStream::parse("GET /users/10?active=true&teste=[2,3,5] HTTP/1.1");
        let route_matched = request.match_route(&input);
        assert!(route_matched.is_some());
        assert_eq!(route_matched.unwrap().path, "/users/:userId");


        let input = HttpInputStream::parse("GET /users/10/books?active=true&teste=[2,3,5] HTTP/1.1");
        let route_matched = request.match_route(&input);
        assert!(route_matched.is_some());
        assert_eq!(route_matched.unwrap().path, "/users/:userId/books");


        let input = HttpInputStream::parse("GET /users/10/books/123?active=true&teste=[2,3,5] HTTP/1.1");
        let route_matched = request.match_route(&input);
        assert!(route_matched.is_some());
        assert_eq!(route_matched.unwrap().path, "/users/:userId/books/:bookId");
    }

    #[test]
    fn it_parse_fragment_request(){
        let mut request = HttpServer::new();

        request.get("/users", Box::new(|_|{ HttpResponse::new() }));
        request.get("/users/:userId", Box::new(|_|{ HttpResponse::new() }));
        request.get("/users/:userId/books", Box::new(|_|{ HttpResponse::new() }));
        request.get("/users/:userId/books/:bookId", Box::new(|_|{ HttpResponse::new() }));

        let input = HttpInputStream::parse("GET /users#teste HTTP/1.1");
        let route_matched = request.match_route(&input);
        assert!(route_matched.is_some());
        assert_eq!(route_matched.unwrap().path, "/users");


        let input = HttpInputStream::parse("GET /users/10#teste HTTP/1.1");
        let route_matched = request.match_route(&input);
        assert!(route_matched.is_some());
        assert_eq!(route_matched.unwrap().path, "/users/:userId");


        let input = HttpInputStream::parse("GET /users/10/books#teste HTTP/1.1");
        let route_matched = request.match_route(&input);
        assert!(route_matched.is_some());
        assert_eq!(route_matched.unwrap().path, "/users/:userId/books");


        let input = HttpInputStream::parse("GET /users/10/books/123#teste HTTP/1.1");
        let route_matched = request.match_route(&input);
        assert!(route_matched.is_some());
        assert_eq!(route_matched.unwrap().path, "/users/:userId/books/:bookId");
    }
}