use serde::{Serialize, Deserialize};

mod http_server;
mod http_response;
mod http_request;
mod http_input_stream;

#[derive(Serialize, Deserialize)]
struct User<'a> {
    id: &'a str,
    name: &'a str
}

#[cfg(test)]
pub mod tests {
    use crate::User;
    use crate::http_server;
    use crate::http_response;

    #[test]
    fn test(){
        let mut server = http_server::HttpServer::new();

        server.get("/users", Box::new(|_| {
            let mut response = http_response::HttpResponse::new();

            let user = User{
                id: "123",
                name: "Rust user"
            };

            response.set_body(serde_json::to_string(&user).unwrap());
            response
        }));

        server.bind("127.0.0.1:8080")

    }
}