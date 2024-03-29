use http::{StatusCode, HeaderMap, header::{CONTENT_LENGTH, ACCEPT_CHARSET, IntoHeaderName}, HeaderValue};

pub struct HttpResponse {
    pub status: StatusCode,
    pub body: String,
    pub headers: HeaderMap
}

impl HttpResponse {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();

        headers.insert(ACCEPT_CHARSET, HeaderValue::from_str("utf-8").unwrap());

        Self {
            body: String::new(),
            status: StatusCode::default(),
            headers
        }
    }

    pub fn set_status(&mut self, status: StatusCode) {
        self.status = status;
    }

    pub fn set_body(&mut self, content: String) {
        if content.is_empty() {
            return
        }

        self.headers.insert(CONTENT_LENGTH, HeaderValue::from(content.len()));

        self.body = content;
    }

    pub fn set_headers<K>(&mut self, key: K, value: &str)
    where
        K: IntoHeaderName,
    {
        self.headers.insert(key, HeaderValue::from_str(value).unwrap());
    }

}


#[cfg(test)]
pub mod tests {
    use http::{StatusCode, header::{CONTENT_LENGTH, CONTENT_TYPE}, HeaderValue};
    use serde::{Serialize, Deserialize};

    use crate::http_response::HttpResponse;

    #[test]
    fn it_can_create(){
        let result = HttpResponse::new();

        assert!(result.body.is_empty());
        assert_eq!(result.status, StatusCode::default());
    }

    #[test]
    fn it_can_insert_status(){
        let mut result = HttpResponse::new();

        result.set_status(StatusCode::CREATED);

        assert_eq!(result.status, StatusCode::CREATED);
    }

    #[test]
    fn it_can_insert_body(){
        let mut result = HttpResponse::new();

        result.set_body(String::from("Couteúdo da http response"));

        assert!(!result.body.is_empty());
        assert_eq!(result.body, "Couteúdo da http response");
    }

    
    #[test]
    fn it_can_insert_json_body(){
        let mut result = HttpResponse::new();

        #[derive(Serialize, Deserialize)]
        struct Response<'a> {
            user_name: &'a str,
        }

        let response = Response {user_name: "Natã Alves"};

        let response = serde_json::to_string(&response).unwrap();
        let expected = response.clone();

        result.set_body(response);

        assert!(!result.body.is_empty());
        assert_eq!(result.body, expected);
    }


    #[test]
    fn it_can_insert_headers(){
        let mut result = HttpResponse::new();

        result.set_headers(CONTENT_TYPE, "text/html");

        assert_eq!(result.headers.len(), 2)
    }

    #[test]
    fn it_can_insert_headers_with_calc_content_lenght(){
        let mut result = HttpResponse::new();

        result.set_body(String::from("content"));

        assert!(result.headers.contains_key(CONTENT_LENGTH));

        if result.headers.contains_key(CONTENT_LENGTH) {
            assert_eq!(result.headers.get(CONTENT_LENGTH).unwrap(), HeaderValue::from(7));
        }
    }
}