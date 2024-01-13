use std::collections::HashMap;

use regex::Regex;
use url::Url;

use crate::{http_input_stream::HttpInputStream, http_server::Route};

pub struct HttpRequest {
    pub method: http::Method,
    pub params: HashMap<String, String>,
    pub query :HashMap<String, String>,
}


impl HttpRequest {
    pub fn new() -> Self {
        Self {
            method: http::Method::default(),
            params: HashMap::new(),
            query: HashMap::new()
        }
    }

    pub fn parse(input_stream: &HttpInputStream, route: &Route) -> Self {
        let query = extract_from_http_input_stream_query(input_stream.relative_path);
        let params = extract_from_http_input_stream_params(route, input_stream.relative_path);

        return Self {
            method: http::Method::GET,
            params,
            query
        }
    }
}

fn extract_from_http_input_stream_query<'a>(url: &'a str) -> HashMap<String,String>  {
    let base_url = Url::parse("http://localhost:8000").unwrap();

    let url_parser =  Url::options().base_url(Some(&base_url)).parse(url).unwrap();

    return url_parser.query_pairs().into_owned().collect::<HashMap<String,String>>();
}

fn extract_from_http_input_stream_params<'a>(route: &Route, url: &'a str) -> HashMap<String, String>  {
    let mut params: HashMap<String, String> = HashMap::new();

    let regex = Regex::new(&route.regex_path).expect("Erro ao compilar a regex");

    let url = remove_query_and_fragment(url);

    // Verifique se a regex corresponde à entrada
    if let Some(captures) = regex.captures(&url) {
        // Captura os valores correspondentes aos grupos na regex
        let path_parts = route.path.split("/").collect::<Vec<&str>>();

        let mut captures_index = 1;

        for url_part in &path_parts {
            if  url_part.starts_with(":") {

                if let Some(value) = captures.get(captures_index) {
                    let param_key = &url_part.replace(":", "");
                    let param_value = value.as_str();

                    params.insert(String::from(param_key), String::from(param_value));
                }

                captures_index += 1;
            }
        }
    }

    return params;
}

fn remove_query_and_fragment(url_str: &str) -> String {
    if let Some(query_index) = url_str.find('?') {
        let url_without_query = &url_str[..query_index];

        if let Some(fragment_index) = url_without_query.find('#') {
            return url_without_query[..fragment_index].to_string();
        }

        return String::from(url_without_query);
    }

    if let Some(fragment_index) = url_str.find('#') {
        return String::from(&url_str[..fragment_index]);
    }

    String::from(url_str)
}


#[cfg(test)]
pub mod tests {
    use crate::{http_input_stream::HttpInputStream, http_response::HttpResponse, http_server::Route};

    use super::HttpRequest;

    #[test]
    fn it_parse_params_request(){
        let input_stream = HttpInputStream::parse("GET /users/10/books/123 HTTP/1.1"); 

        let route = Route {
            callback: Box::new(|_| { HttpResponse::new() }),
            path: "/users/:userId/books/:bookId".to_string(),
            regex_path: r"/users/(\d+)/books/(\d+)".to_string()
        };

        let request = HttpRequest::parse(&input_stream, &route);

        assert_eq!(request.method, http::Method::GET);

        assert!(!request.params.is_empty());

        let user_id= request.params.get("userId");
        let book_id= request.params.get("bookId");

        assert_eq!(*user_id.unwrap(), "10");
        assert_eq!(*book_id.unwrap(), "123");
    }


     #[test]
    fn it_parse_query_request(){
        let input_stream = HttpInputStream::parse("GET /users?type=hatch&color=red HTTP/1.1"); 

        let route = Route {
            callback: Box::new(|_| { HttpResponse::new() }),
            path: "/users".to_string(),
            regex_path: r"/users".to_string()
        };

        let request = HttpRequest::parse(&input_stream, &route);

        assert_eq!(request.method, http::Method::GET);

        assert!(!request.query.is_empty());
        
        let car_type= request.query.get("type");
        let car_color= request.query.get("color");

        assert_eq!(car_type.unwrap(), "hatch");
        assert_eq!(car_color.unwrap(), "red");
    }


    #[test]
    fn it_parse_query_and_params_request(){
        let input_stream = HttpInputStream::parse("GET /cars/10?type=hatch HTTP/1.1"); 

        let route = Route {
            callback: Box::new(|_| { HttpResponse::new() }),
            path: "/cars/:carId".to_string(),
            regex_path: r"/cars/([^/]+)(?:\?([^#]+))?(?:#(.*))?$".to_string()
        };

        let request = HttpRequest::parse(&input_stream, &route);

        assert_eq!(request.method, http::Method::GET);

        let car_id = request.params.get("carId");

        match car_id {
            Some(value) => assert_eq!(value, "10"),
            None => panic!("Expected Some, got None."),
        }

        let car_type= request.query.get("type");

        match car_type {
            Some(value) => assert_eq!(value, "hatch"),
            None => panic!("Expected Some, got None."),
        }
    }
}