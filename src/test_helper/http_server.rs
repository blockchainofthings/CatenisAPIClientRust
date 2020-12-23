use std::{
    collections::hash_map::HashMap,
    thread,
    sync::{
        Arc,
        atomic::{
            AtomicU16, Ordering,
        }
    },
    error,
    str::FromStr,
    io::Read,
};
use serde::{
    Deserialize, Serialize,
};
use tiny_http::{
    Server, Request, Response, StatusCode, Header
};
use flate2::{
    Compression,
    read::{
        ZlibEncoder, GzEncoder,
    },
};

static HTTP_SERVER_PORT: AtomicU16 = AtomicU16::new(4001);

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct HttpHeader {
    pub field: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct HttpRequest {
    pub secure: bool,
    pub version: String,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, HttpHeader>,
    pub body: String,
}

impl HttpRequest {
    fn from_request(req: &mut Request) -> Self {
        let mut headers: HashMap<String, HttpHeader> = HashMap::new();

        for header in req.headers() {
            let field = header.field.to_string();
            let case_insensitive_field = field.to_ascii_lowercase();
            let mut value = header.value.to_string();

            if headers.contains_key(&case_insensitive_field) {
                value = value + "," + headers.get(&case_insensitive_field).unwrap().value.as_str();
            }

            headers.insert(case_insensitive_field, HttpHeader {
                field,
                value,
            });
        }

        let mut body= String::new();
        req.as_reader().read_to_string(&mut body).unwrap();

        HttpRequest {
            secure: req.secure(),
            version: req.http_version().to_string(),
            method: req.method().to_string(),
            path: req.url().to_string(),
            headers,
            body,
        }
    }

    pub fn from_json(json: &str) -> Result<Self, Box<dyn error::Error + Send + Sync>> {
        let http_req = serde_json::from_str(json)?;

        Ok(http_req)
    }

    pub fn scheme(&self) -> String {
        String::from(if self.secure { "https" } else { "http" })
    }

    pub fn url(&self) -> Option<String> {
        if let Some(header) = self.headers.get("host") {
            Some(self.scheme() + "://" + &header.value + &self.path)
        } else {
            // Request has no Host header so URL cannot be assembled
            None
        }
    }
}

#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
pub struct PartialHttpRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, Option<HttpHeader>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

impl PartialEq<PartialHttpRequest> for HttpRequest {
    fn eq(&self, other: &PartialHttpRequest) -> bool {
        if let Some(other_method) = &other.method {
            if self.method != *other_method {
                return false;
            }
        }

        if let Some(other_path) = &other.path {
            if self.path != *other_path {
                return false;
            }
        }

        if let Some(other_headers) = &other.headers {
            for (key, header) in other_headers {
                if !self.headers.contains_key(key) {
                    return false;
                }

                if let Some(header_info) = header {
                    if *self.headers.get(key).unwrap() != *header_info {
                        return false;
                    }
                }
            }
        }

        if let Some(other_body) = &other.body {
            if self.body != *other_body {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ContentEncoding {
    Identify,
    Deflate,
    Gzip,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HttpBody {
    pub data: Vec<u8>,
    pub content_type: Option<String>,
    pub content_encoding: Option<ContentEncoding>,
}

impl From<&str> for HttpBody {
    fn from(data: &str) -> Self {
        HttpBody {
            data: Vec::from(data.as_bytes()),
            content_type: Some(String::from("text/plain; charset=utf-8")),
            content_encoding: None,
        }
    }
}

impl HttpBody {
    pub fn from_json(json: &str) -> std::result::Result<Self, &str> {
        if serde_json::from_str::<serde_json::Value>(json).is_err() {
            return Err("Invalid JSON");
        }

        Ok(HttpBody {
            data: Vec::from(json.as_bytes()),
            content_type: Some(String::from("application/json; charset=utf-8")),
            content_encoding: None,
        })
    }

    pub fn with_content_encoding(self, content_encoding: ContentEncoding) -> Self {
        HttpBody {
            data: self.data,
            content_type: self.content_type,
            content_encoding: Some(content_encoding),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HttpServerMode {
    Echo,
    Error(u16, Option<HttpBody>),
    Success(HttpBody),
}

#[derive(Debug, Clone)]
pub struct HttpServer {
    mode: HttpServerMode,
    host: String,
    port: u16,
    expected_req: Option<PartialHttpRequest>,
}

impl HttpServer {
    fn next_server_port() -> u16 {
        HTTP_SERVER_PORT.fetch_add(1, Ordering::SeqCst)
    }

    pub fn new(mode: HttpServerMode, host: &str) -> Self {
        let port = HttpServer::next_server_port();

        // Make sure that host uses the (globally) acquired port
        let host_parts: Vec<&str> = host.split(':').collect();

        let host = format!("{}:{}", host_parts[0], port);

        HttpServer {
            mode,
            host,
            port,
            expected_req: None,
        }
    }

    pub fn with_expected_request(mut self, req: PartialHttpRequest) -> Self {
        self.expected_req = Some(req);

        self
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn start(&self) {
        let http_server = self.clone();

        let server = Arc::new(Server::http(&self.host).unwrap());
        let server2 = server.clone();

        thread::spawn(move || {
            match &http_server.mode {
                HttpServerMode::Echo => {
                    // Wait for request
                    let mut req = server2.recv().unwrap();

                    let http_req = HttpRequest::from_request(&mut req);
                    let json = serde_json::to_string(&http_req).unwrap();

                    // Send response with serialized (JSON) received request
                    req.respond(Response::from_string(json)).unwrap();
                },
                HttpServerMode::Error(status_code, err_body) => {
                    if let Some(body) = err_body {
                        // Wait for request
                        let req = server2.recv().unwrap();

                        // Prepare response to be send back

                        // Encode body if required
                        let mut content_encoding_header = None;
                        let enc_body_content = match &body.content_encoding {
                            Some(ContentEncoding::Deflate) => {
                                // Compress body using deflate algorithm
                                let mut enc = ZlibEncoder::new(body.data.as_slice(), Compression::default());
                                let mut enc_body = Vec::new();
                                enc.read_to_end(&mut enc_body).unwrap();

                                // Prepare Content-Encoding header
                                content_encoding_header = Some(Header::from_str("Content-Encoding:deflate").unwrap());

                                Some(enc_body)
                            },
                            Some(ContentEncoding::Gzip) => {
                                // Compress body using gzip algorithm
                                let mut enc = GzEncoder::new(body.data.as_slice(), Compression::default());
                                let mut enc_body = Vec::new();
                                enc.read_to_end(&mut enc_body).unwrap();

                                // Prepare Content-Encoding header
                                content_encoding_header = Some(Header::from_str("Content-Encoding:gzip").unwrap());

                                Some(enc_body)
                            },
                            _ => None,
                        };

                        let mut res = Response::from_data(
                            if let Some(enc_body) = &enc_body_content {
                                enc_body.as_slice()
                            } else {
                                body.data.as_slice()
                            }
                        );
                        res = res.with_status_code(StatusCode(*status_code));

                        if let Some(content_type) = &body.content_type {
                            res = res.with_header(Header::from_str(format!("Content-Type:{}", content_type).as_str()).unwrap());
                        }

                        if let Some(header) = content_encoding_header {
                            res = res.with_header(header);
                        }

                        // Send error response
                        req.respond(res).unwrap();
                    } else {
                        // Wait for request
                        let req = server2.recv().unwrap();

                        // Send error response
                        req.respond(Response::empty(StatusCode(*status_code))).unwrap();
                    }
                },
                HttpServerMode::Success(body) => {
                    // Wait for request
                    let mut req = server2.recv().unwrap();

                    // Prepare response to be send back
                    let mut error_res = None;

                    if let Some(expected_req) = &http_server.expected_req {
                        // Validate request
                        let http_req = HttpRequest::from_request(&mut req);

                        if http_req != *expected_req {
                            // Received request does not match expected request.
                            //  Prepare error response
                            error_res = Some(
                                Response::from_data(format!(
                                    "Invalid request.\n Expected: {}\n Received: {}",
                                    serde_json::to_string(expected_req).unwrap(),
                                    serde_json::to_string(&http_req).unwrap(),
                                )).with_status_code(StatusCode::from(400))
                            );
                        }
                    }

                    if let Some(res) = error_res {
                        // Send error response
                        req.respond(res).unwrap();
                    } else {
                        // Encode body if required
                        let mut content_encoding_header = None;
                        let enc_body_content = match &body.content_encoding {
                            Some(ContentEncoding::Deflate) => {
                                // Compress body using gzip algorithm
                                let mut enc = ZlibEncoder::new(body.data.as_slice(), Compression::default());
                                let mut enc_body = Vec::new();
                                enc.read_to_end(&mut enc_body).unwrap();

                                // Prepare Content-Encoding header
                                content_encoding_header = Some(Header::from_str("Content-Encoding:deflate").unwrap());

                                Some(enc_body)
                            },
                            Some(ContentEncoding::Gzip) => {
                                // Compress body using deflate algorithm
                                let mut enc = GzEncoder::new(body.data.as_slice(), Compression::default());
                                let mut enc_body = Vec::new();
                                enc.read_to_end(&mut enc_body).unwrap();

                                // Prepare Content-Encoding header
                                content_encoding_header = Some(Header::from_str("Content-Encoding:gzip").unwrap());

                                Some(enc_body)
                            },
                            _ => None,
                        };

                        let mut res = Response::from_data(
                            if let Some(enc_body) = &enc_body_content {
                                enc_body.as_slice()
                            } else {
                                body.data.as_slice()
                            }
                        );

                        if let Some(content_type) = &body.content_type {
                            res = res.with_header(Header::from_str(format!("Content-Type:{}", content_type).as_str()).unwrap());
                        }

                        if let Some(header) = content_encoding_header {
                            res = res.with_header(header);
                        }

                        // Send success response
                        req.respond(res).unwrap();
                    }
                },
            };
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serialize_http_header() {
        let http_header = HttpHeader {
            field: String::from("Host"),
            value: String::from("localhost:3000"),
        };

        let json = serde_json::to_string(&http_header).unwrap();

        assert_eq!(json, r#"{"field":"Host","value":"localhost:3000"}"#);
    }

    #[test]
    fn it_deserialize_http_header() {
        let json = r#"{"field":"Host","value":"localhost:3000"}"#;

        let http_header: HttpHeader = serde_json::from_str(json).unwrap();

        assert_eq!(http_header, HttpHeader {
            field: String::from("Host"),
            value: String::from("localhost:3000"),
        });
    }
    
    #[test]
    fn it_serialize_empty_body_http_request() {
        let http_request = HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("GET"),
            path: String::from("/messages"),
            headers: vec![
                String::from("host"),
                String::from("content-type"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("Host"),
                    value: String::from("localhost:3000"),
                },
                HttpHeader {
                    field: String::from("Content-Type"),
                    value: String::from("application/json; charset=utf-8"),
                },
            ].into_iter()).collect(),
            body: String::from(""),
        };

        let json = serde_json::to_string(&http_request).unwrap();

        // Note: we cannot guarantee the order in which the entries of the 'headers' field will be
        //  serialized so we must include all possible results in the assertion statement bellow
        assert!(
            json.eq(r#"{"secure":false,"version":"1.1","method":"GET","path":"/messages","headers":{"host":{"field":"Host","value":"localhost:3000"},"content-type":{"field":"Content-Type","value":"application/json; charset=utf-8"}},"body":""}"#)
            ||
            json.eq(r#"{"secure":false,"version":"1.1","method":"GET","path":"/messages","headers":{"content-type":{"field":"Content-Type","value":"application/json; charset=utf-8"},"host":{"field":"Host","value":"localhost:3000"}},"body":""}"#),
            "HttpRequest struct not serialized as expected"
        );
    }

    #[test]
    fn it_deserialize_empty_body_http_request() {
        let json = r#"{"secure":false,"version":"1.1","method":"GET","path":"/messages","headers":{"host":{"field":"Host","value":"localhost:3000"},"content-type":{"field":"Content-Type","value":"application/json; charset=utf-8"}},"body":""}"#;

        let http_request: HttpRequest = serde_json::from_str(json).unwrap();

        assert_eq!(http_request, HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("GET"),
            path: String::from("/messages"),
            headers: vec![
                String::from("host"),
                String::from("content-type"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("Host"),
                    value: String::from("localhost:3000"),
                },
                HttpHeader {
                    field: String::from("Content-Type"),
                    value: String::from("application/json; charset=utf-8"),
                },
            ].into_iter()).collect(),
            body: String::from(""),
        });
    }

    #[test]
    fn it_serialize_http_request() {
        let http_request = HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("POST"),
            path: String::from("/messages/log"),
            headers: vec![
                String::from("host"),
                String::from("content-type"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("Host"),
                    value: String::from("localhost:3000"),
                },
                HttpHeader {
                    field: String::from("Content-Type"),
                    value: String::from("application/json; charset=utf-8"),
                },
            ].into_iter()).collect(),
            body: String::from(r#"{"message":"Test message"}"#),
        };

        let json = serde_json::to_string(&http_request).unwrap();

        // Note: we cannot guarantee the order in which the entries of the 'headers' field will be
        //  serialized so we must include all possible results in the assertion statement bellow
        assert!(
            json.eq(r#"{"secure":false,"version":"1.1","method":"POST","path":"/messages/log","headers":{"host":{"field":"Host","value":"localhost:3000"},"content-type":{"field":"Content-Type","value":"application/json; charset=utf-8"}},"body":"{\"message\":\"Test message\"}"}"#)
            ||
            json.eq(r#"{"secure":false,"version":"1.1","method":"POST","path":"/messages/log","headers":{"content-type":{"field":"Content-Type","value":"application/json; charset=utf-8"},"host":{"field":"Host","value":"localhost:3000"}},"body":"{\"message\":\"Test message\"}"}"#),
            "HttpRequest struct not serialized as expected"
        );
    }

    #[test]
    fn it_deserialize_http_request() {
        let json = r#"{"secure":false,"version":"1.1","method":"POST","path":"/messages/log","headers":{"host":{"field":"Host","value":"localhost:3000"},"content-type":{"field":"Content-Type","value":"application/json; charset=utf-8"}},"body":"{\"message\":\"Test message\"}"}"#;

        let http_request: HttpRequest = serde_json::from_str(json).unwrap();

        assert_eq!(http_request, HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("POST"),
            path: String::from("/messages/log"),
            headers: vec![
                String::from("host"),
                String::from("content-type"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("Host"),
                    value: String::from("localhost:3000"),
                },
                HttpHeader {
                    field: String::from("Content-Type"),
                    value: String::from("application/json; charset=utf-8"),
                },
            ].into_iter()).collect(),
            body: String::from(r#"{"message":"Test message"}"#),
        });
    }

    #[test]
    fn it_get_scheme_of_http_request() {
        let http_request = HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("GET"),
            path: String::from("/messages"),
            headers: vec![
                String::from("host"),
                String::from("content-type"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("Host"),
                    value: String::from("localhost:3000"),
                },
                HttpHeader {
                    field: String::from("Content-Type"),
                    value: String::from("application/json; charset=utf-8"),
                },
            ].into_iter()).collect(),
            body: String::from(""),
        };

        assert_eq!(http_request.scheme(), "http");
    }

    #[test]
    fn it_get_scheme_of_secure_http_request() {
        let http_request = HttpRequest {
            secure: true,
            version: String::from("1.1"),
            method: String::from("GET"),
            path: String::from("/messages"),
            headers: vec![
                String::from("host"),
                String::from("content-type"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("Host"),
                    value: String::from("localhost:3000"),
                },
                HttpHeader {
                    field: String::from("Content-Type"),
                    value: String::from("application/json; charset=utf-8"),
                },
            ].into_iter()).collect(),
            body: String::from(""),
        };

        assert_eq!(http_request.scheme(), "https");
    }

    #[test]
    fn it_get_url_of_http_request() {
        let http_request = HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("GET"),
            path: String::from("/messages"),
            headers: vec![
                String::from("host"),
                String::from("content-type"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("Host"),
                    value: String::from("localhost:3000"),
                },
                HttpHeader {
                    field: String::from("Content-Type"),
                    value: String::from("application/json; charset=utf-8"),
                },
            ].into_iter()).collect(),
            body: String::from(""),
        };

        assert_eq!(http_request.url(), Some(String::from("http://localhost:3000/messages")));
    }

    #[test]
    fn it_try_get_url_of_http_request_missing_host() {
        let http_request = HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("GET"),
            path: String::from("/messages"),
            headers: vec![
                String::from("content-type"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("Content-Type"),
                    value: String::from("application/json; charset=utf-8"),
                },
            ].into_iter()).collect(),
            body: String::from(""),
        };

        assert!(http_request.url().is_none(), "URL incorrectly obtained from an HTTP request missing the Host header");
    }

    #[test]
    fn it_get_http_request_from_json() {
        let json = r#"{"secure":false,"version":"1.1","method":"GET","path":"/messages","headers":{"host":{"field":"Host","value":"localhost:3000"},"content-type":{"field":"Content-Type","value":"application/json; charset=utf-8"}},"body":""}"#;

        let http_request = HttpRequest::from_json(json);

        assert!(!http_request.is_err(), "Error generating HTTP request from JSON");

        assert_eq!(http_request.unwrap(), HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("GET"),
            path: String::from("/messages"),
            headers: vec![
                String::from("host"),
                String::from("content-type"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("Host"),
                    value: String::from("localhost:3000"),
                },
                HttpHeader {
                    field: String::from("Content-Type"),
                    value: String::from("application/json; charset=utf-8"),
                },
            ].into_iter()).collect(),
            body: String::from(""),
        });
    }

    #[test]
    fn it_try_get_invalid_http_request_from_json() {
        let json = r#"{}"#;

        assert!(HttpRequest::from_json(json).is_err(), "HTTP request incorrectly obtained from an invalid JSON");
    }

    #[test]
    fn it_serialize_partial_http_request_no_opts() {
        let part_http_request = PartialHttpRequest {
            method: None,
            path: None,
            headers: None,
            body: None,
        };

        let json = serde_json::to_string(&part_http_request).unwrap();

        assert_eq!(json, r#"{}"#);
    }

    #[test]
    fn it_serialize_partial_http_request_all_opts() {
        let part_http_request = PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(String::from("/messages/log")),
            headers: Some(
                vec![
                    String::from("host"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    Some(HttpHeader {
                        field: String::from("Content-Type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Test message"}"#)),
        };

        let json = serde_json::to_string(&part_http_request).unwrap();

        // Note: we cannot guarantee the order in which the entries of the 'headers' field will be
        //  serialized so we must include all possible results in the assertion statement bellow
        assert!(
            json.eq(r#"{"method":"POST","path":"/messages/log","headers":{"host":null,"content-type":{"field":"Content-Type","value":"application/json; charset=utf-8"}},"body":"{\"message\":\"Test message\"}"}"#)
            ||
            json.eq(r#"{"method":"POST","path":"/messages/log","headers":{"content-type":{"field":"Content-Type","value":"application/json; charset=utf-8"},"host":null},"body":"{\"message\":\"Test message\"}"}"#),
            "PartialHttpRequest struct not serialized as expected"
        );
    }

    #[test]
    fn it_compare_partial_http_request_equal() {
        let http_request = HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("POST"),
            path: String::from("/messages/log"),
            headers: vec![
                String::from("host"),
                String::from("content-type"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("Host"),
                    value: String::from("localhost:3000"),
                },
                HttpHeader {
                    field: String::from("Content-Type"),
                    value: String::from("application/json; charset=utf-8"),
                },
            ].into_iter()).collect(),
            body: String::from(r#"{"message":"Test message"}"#),
        };

        let part_http_request = PartialHttpRequest {
            method: Some(String::from("POST")),
            path: None,
            headers: None,
            body: None,
        };

        assert!(http_request == part_http_request, "HTTP request does not match partial HTTP request (with method only)");

        let part_http_request = PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(String::from("/messages/log")),
            headers: None,
            body: None,
        };

        assert!(http_request == part_http_request, "HTTP request does not match partial HTTP request (with method and path)");

        let part_http_request = PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(String::from("/messages/log")),
            headers: Some(
                vec![
                    String::from("host"),
                ].into_iter().zip(vec![
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        };

        assert!(http_request == part_http_request, "HTTP request does not match partial HTTP request (with method, path and single no data header)");

        let part_http_request = PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(String::from("/messages/log")),
            headers: Some(
                vec![
                    String::from("host"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    Some(HttpHeader {
                        field: String::from("Content-Type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: None,
        };

        assert!(http_request == part_http_request, "HTTP request does not match partial HTTP request (with method, path and all headers)");

        let part_http_request = PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(String::from("/messages/log")),
            headers: Some(
                vec![
                    String::from("host"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    Some(HttpHeader {
                        field: String::from("Content-Type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Test message"}"#)),
        };

        assert!(http_request == part_http_request, "HTTP request does not match partial HTTP request (complete)");
    }

    #[test]
    fn it_compare_partial_http_request_not_equal() {
        let http_request = HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("POST"),
            path: String::from("/messages/log"),
            headers: vec![
                String::from("host"),
                String::from("content-type"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("Host"),
                    value: String::from("localhost:3000"),
                },
                HttpHeader {
                    field: String::from("Content-Type"),
                    value: String::from("application/json; charset=utf-8"),
                },
            ].into_iter()).collect(),
            body: String::from(r#"{"message":"Test message"}"#),
        };

        let part_http_request = PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(String::from("/messages/log")),
            headers: Some(
                vec![
                    String::from("host"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    Some(HttpHeader {
                        field: String::from("Content-Type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Test message"}"#)),
        };

        assert!(http_request != part_http_request, "HTTP request unexpectedly matches partial HTTP request (wrong method)");

        let part_http_request = PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(String::from("/messages/send")),
            headers: Some(
                vec![
                    String::from("host"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    Some(HttpHeader {
                        field: String::from("Content-Type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Test message"}"#)),
        };

        assert!(http_request != part_http_request, "HTTP request unexpectedly matches partial HTTP request (wrong path)");

        let part_http_request = PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(String::from("/messages/log")),
            headers: Some(
                vec![
                    String::from("host"),
                    String::from("content-type"),
                    String::from("accept-encoding"),
                ].into_iter().zip(vec![
                    None,
                    Some(HttpHeader {
                        field: String::from("Content-Type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                    Some(HttpHeader {
                        field: String::from("Accept-Encoding"),
                        value: String::from("gzip"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Test message"}"#)),
        };

        assert!(http_request != part_http_request, "HTTP request unexpectedly matches partial HTTP request (extra missing header)");

        let part_http_request = PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(String::from("/messages/log")),
            headers: Some(
                vec![
                    String::from("accept-encoding"),
                ].into_iter().zip(vec![
                    Some(HttpHeader {
                        field: String::from("Accept-Encoding"),
                        value: String::from("gzip"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Test message"}"#)),
        };

        assert!(http_request != part_http_request, "HTTP request unexpectedly matches partial HTTP request (single missing header)");

        let part_http_request = PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(String::from("/messages/log")),
            headers: Some(
                vec![
                    String::from("host"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    Some(HttpHeader {
                        field: String::from("Content-Type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Another test message"}"#)),
        };

        assert!(http_request != part_http_request, "HTTP request unexpectedly matches partial HTTP request (wrong body)");
    }

    #[test]
    fn it_convert_str_to_http_body() {
        let body = "Test body";
        let http_body = HttpBody::from(body);

        assert_eq!(http_body, HttpBody {
            data: Vec::from(&b"Test body"[..]),
            content_type: Some(String::from("text/plain; charset=utf-8")),
            content_encoding: None,
        });
    }

    #[test]
    fn it_convert_json_to_http_body() {
        let body = r#"{"status":"success","data":{"messageId":"mg9x9vCqYMg9YtKdDwQx"}}"#;
        let body_result = HttpBody::from_json(body);
        
        assert!(!body_result.is_err(), "Unexpected error converting JSON to HTTP body");
        
        let http_body = body_result.unwrap();

        assert_eq!(http_body, HttpBody {
            data: Vec::from(&b"{\"status\":\"success\",\"data\":{\"messageId\":\"mg9x9vCqYMg9YtKdDwQx\"}}"[..]),
            content_type: Some(String::from("application/json; charset=utf-8")),
            content_encoding: None,
        });
    }

    #[test]
    fn it_try_convert_invalid_json_to_http_body() {
        let body = "bla";
        let body_result = HttpBody::from_json(body);

        assert!(body_result.is_err(), "Unexpected result converting JSON to HTTP body: conversion did not fail");
        assert_eq!(body_result.err().unwrap(), "Invalid JSON");
    }

    #[test]
    fn it_create_http_body_with_content_encoding() {
        let mut http_body = HttpBody {
            data: Vec::from(&b"Test body"[..]),
            content_type: None,
            content_encoding: None,
        };
        http_body = http_body.with_content_encoding(ContentEncoding::Identify);

        assert_eq!(http_body, HttpBody {
            data: Vec::from(&b"Test body"[..]),
            content_type: None,
            content_encoding: Some(ContentEncoding::Identify),
        });
    }

    #[test]
    fn it_start_server_and_echo_get_http_request() {
        // Start HTTP server in echo mode
        let http_server = HttpServer::new(HttpServerMode::Echo, "localhost");
        http_server.start();

        let server_port = http_server.get_port();

        // Send an HTTP request and get the response
        let res = reqwest::blocking::get(&format!("http://localhost:{}/messages", server_port)).unwrap();

        // Parse returned HTTP request from response body
        let res_body = res.text().unwrap();
        let http_request = HttpRequest::from_json(&res_body).unwrap();

        assert_eq!(http_request, HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("GET"),
            path: String::from("/messages"),
            headers: vec![
                String::from("host"),
                String::from("accept"),
                String::from("accept-encoding"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("host"),
                    value: format!("localhost:{}", server_port),
                },
                HttpHeader {
                    field: String::from("accept"),
                    value: String::from("*/*"),
                },
                HttpHeader {
                    field: String::from("accept-encoding"),
                    value: String::from("gzip"),
                },
            ].into_iter()).collect(),
            body: String::from(""),
        });
    }

    #[test]
    fn it_start_server_and_echo_post_http_request() {
        // Start HTTP server in echo mode
        let http_server = HttpServer::new(HttpServerMode::Echo, "localhost");
        http_server.start();

        let server_port = http_server.get_port();

        // Send an HTTP request and get the response
        let res = reqwest::blocking::Client::new()
            .post(&format!("http://localhost:{}/messages/log", server_port))
            .body(r#"{"message":"Test message"}"#)
            .send()
            .unwrap();

        // Parse returned HTTP request from response body
        let res_body = res.text().unwrap();
        let http_request = HttpRequest::from_json(&res_body).unwrap();

        assert_eq!(http_request, HttpRequest {
            secure: false,
            version: String::from("1.1"),
            method: String::from("POST"),
            path: String::from("/messages/log"),
            headers: vec![
                String::from("host"),
                String::from("accept"),
                String::from("accept-encoding"),
                String::from("content-length"),
            ].into_iter().zip(vec![
                HttpHeader {
                    field: String::from("host"),
                    value: format!("localhost:{}", server_port),
                },
                HttpHeader {
                    field: String::from("accept"),
                    value: String::from("*/*"),
                },
                HttpHeader {
                    field: String::from("accept-encoding"),
                    value: String::from("gzip"),
                },
                HttpHeader {
                    field: String::from("content-length"),
                    value: String::from("26"),
                },
            ].into_iter()).collect(),
            body: String::from(r#"{"message":"Test message"}"#),
        });
    }

    #[test]
    fn it_start_server_and_send_error_response() {
        // Start HTTP server in error simulation mode
        let err_body = r#"{"status":"error","message":"Internal server error"}"#;

        let http_server = HttpServer::new(
            HttpServerMode::Error(
                500,
                Some(HttpBody::from_json(err_body).unwrap()),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Send (any) HTTP request and get the response
        let res = reqwest::blocking::get(&format!("http://localhost:{}/", server_port)).unwrap();

        assert_eq!(res.status().as_u16(), 500);

        // Read response body
        let res_body = res.text().unwrap();

        assert_eq!(res_body, err_body);
    }

    #[test]
    fn it_start_server_and_send_deflate_body_error_response() {
        // Start HTTP server in error simulation mode
        let err_body = r#"{"status":"error","message":"Internal server error"}"#;

        let http_server = HttpServer::new(
            HttpServerMode::Error(
                500,
                Some(HttpBody::from_json(err_body).unwrap().with_content_encoding(ContentEncoding::Deflate)),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Send (any) HTTP request and get the response
        let  http_client = reqwest::blocking::ClientBuilder::new()
            .gzip(false)
            .build()
            .unwrap();

        let res = http_client.get(&format!("http://localhost:{}/", server_port))
            .send()
            .unwrap();

        assert_eq!(res.status().as_u16(), 500);
        assert_eq!(res.headers().get(reqwest::header::CONTENT_ENCODING).unwrap(), "deflate");

        // Read response body
        let res_body = res.bytes().unwrap();

        assert_eq!(res_body.as_ref(), b"\x78\x9c\x01\x34\x00\xcb\xff\x7b\x22\x73\x74\x61\x74\x75\x73\x22\x3a\x22\x65\x72\x72\x6f\x72\x22\x2c\x22\x6d\x65\x73\x73\x61\x67\x65\x22\x3a\x22\x49\x6e\x74\x65\x72\x6e\x61\x6c\x20\x73\x65\x72\x76\x65\x72\x20\x65\x72\x72\x6f\x72\x22\x7d\xe4\x6e\x12\x9a");
    }

    #[test]
    fn it_start_server_and_send_gzip_body_error_response() {
        // Start HTTP server in error simulation mode
        let err_body = r#"{"status":"error","message":"Internal server error"}"#;

        let http_server = HttpServer::new(
            HttpServerMode::Error(
                500,
                Some(HttpBody::from_json(err_body).unwrap().with_content_encoding(ContentEncoding::Gzip)),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Send (any) HTTP request and get the response
        let  http_client = reqwest::blocking::ClientBuilder::new()
            .gzip(false)
            .build()
            .unwrap();

        let res = http_client.get(&format!("http://localhost:{}/", server_port))
            .send()
            .unwrap();

        assert_eq!(res.status().as_u16(), 500);
        assert_eq!(res.headers().get(reqwest::header::CONTENT_ENCODING).unwrap(), "gzip");

        // Read response body
        let res_body = res.bytes().unwrap();

        assert_eq!(res_body.as_ref(), b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\xff\x01\x34\x00\xcb\xff\x7b\x22\x73\x74\x61\x74\x75\x73\x22\x3a\x22\x65\x72\x72\x6f\x72\x22\x2c\x22\x6d\x65\x73\x73\x61\x67\x65\x22\x3a\x22\x49\x6e\x74\x65\x72\x6e\x61\x6c\x20\x73\x65\x72\x76\x65\x72\x20\x65\x72\x72\x6f\x72\x22\x7d\x59\xf8\x9b\x57\x34\x00\x00\x00");
    }

    #[test]
    fn it_start_server_and_send_no_body_error_response() {
        // Start HTTP server in error simulation mode
        let http_server = HttpServer::new(
            HttpServerMode::Error(
                503,
                None,
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Send (any) HTTP request and get the response
        let res = reqwest::blocking::get(&format!("http://localhost:{}/", server_port)).unwrap();

        assert_eq!(res.status().as_u16(), 503);

        // Read response body
        let res_body = res.text().unwrap();

        assert_eq!(res_body, String::new());
    }

    #[test]
    fn it_start_server_and_send_success_response() {
        // Start HTTP server in success simulation mode
        let body = r#"{"status":"success","data":{"messageId":"mg9x9vCqYMg9YtKdDwQx"}}"#;

        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(body).unwrap(),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Send (any) HTTP request and get the response
        let res = reqwest::blocking::get(&format!("http://localhost:{}/", server_port)).unwrap();

        assert!(res.status().is_success(), "Unexpected HTTP response: not success");

        // Read response body
        let res_body = res.text().unwrap();

        assert_eq!(res_body, body);
    }

    #[test]
    fn it_start_server_and_send_deflate_body_success_response() {
        // Start HTTP server in success simulation mode
        let body = r#"{"status":"success","data":{"messageId":"mg9x9vCqYMg9YtKdDwQx"}}"#;

        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(body).unwrap().with_content_encoding(ContentEncoding::Deflate),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Send (any) HTTP request and get the response
        let  http_client = reqwest::blocking::ClientBuilder::new()
            .gzip(false)
            .build()
            .unwrap();

        let res = http_client.get(&format!("http://localhost:{}/", server_port))
            .send()
            .unwrap();

        assert!(res.status().is_success(), "Unexpected HTTP response: not success");
        assert_eq!(res.headers().get(reqwest::header::CONTENT_ENCODING).unwrap(), "deflate");

        // Read response body
        let res_body = res.bytes().unwrap();

        assert_eq!(res_body.as_ref(), b"\x78\x9c\x01\x40\x00\xbf\xff\x7b\x22\x73\x74\x61\x74\x75\x73\x22\x3a\x22\x73\x75\x63\x63\x65\x73\x73\x22\x2c\x22\x64\x61\x74\x61\x22\x3a\x7b\x22\x6d\x65\x73\x73\x61\x67\x65\x49\x64\x22\x3a\x22\x6d\x67\x39\x78\x39\x76\x43\x71\x59\x4d\x67\x39\x59\x74\x4b\x64\x44\x77\x51\x78\x22\x7d\x7d\xcc\x82\x16\x16");
    }

    #[test]
    fn it_start_server_and_send_gzip_body_success_response() {
        // Start HTTP server in success simulation mode
        let body = r#"{"status":"success","data":{"messageId":"mg9x9vCqYMg9YtKdDwQx"}}"#;

        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(body).unwrap().with_content_encoding(ContentEncoding::Gzip),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Send (any) HTTP request and get the response
        let  http_client = reqwest::blocking::ClientBuilder::new()
            .gzip(false)
            .build()
            .unwrap();

        let res = http_client.get(&format!("http://localhost:{}/", server_port))
            .send()
            .unwrap();

        assert!(res.status().is_success(), "Unexpected HTTP response: not success");
        assert_eq!(res.headers().get(reqwest::header::CONTENT_ENCODING).unwrap(), "gzip");

        // Read response body
        let res_body = res.bytes().unwrap();

        assert_eq!(res_body.as_ref(), b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\xff\x01\x40\x00\xbf\xff\x7b\x22\x73\x74\x61\x74\x75\x73\x22\x3a\x22\x73\x75\x63\x63\x65\x73\x73\x22\x2c\x22\x64\x61\x74\x61\x22\x3a\x7b\x22\x6d\x65\x73\x73\x61\x67\x65\x49\x64\x22\x3a\x22\x6d\x67\x39\x78\x39\x76\x43\x71\x59\x4d\x67\x39\x59\x74\x4b\x64\x44\x77\x51\x78\x22\x7d\x7d\xbc\x96\xfa\x9b\x40\x00\x00\x00");
    }
    
    #[test]
    fn it_start_server_with_expected_req_send_success() {
        // Start HTTP server in success simulation mode
        let body = r#"{"status":"success","data":{"messageId":"mg9x9vCqYMg9YtKdDwQx"}}"#;

        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(String::from("/messages/log")),
            headers: Some(
                vec![
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Test message"}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Send an HTTP request and get the response
        let res = reqwest::blocking::Client::new()
            .post(&format!("http://localhost:{}/messages/log", server_port))
            .body(r#"{"message":"Test message"}"#)
            .header(reqwest::header::CONTENT_TYPE, "application/json; charset=utf-8")
            .send()
            .unwrap();

        assert!(res.status().is_success(), "Unexpected HTTP response: not success");
    }

    #[test]
    fn it_start_server_with_expected_req_send_error() {
        // Start HTTP server in success simulation mode
        let body = r#"{"status":"success","data":{"messageId":"mg9x9vCqYMg9YtKdDwQx"}}"#;

        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(String::from("/")),
            headers: None,
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Send an HTTP request and get the response
        let res = reqwest::blocking::Client::new()
            .post(&format!("http://localhost:{}/messages/log", server_port))
            .body(r#"{"message":"Test message"}"#)
            .header(reqwest::header::CONTENT_TYPE, "application/json; charset=utf-8")
            .send()
            .unwrap();

        assert_eq!(res.status().as_u16(), 400);

        // Read response body
        let res_body = res.text().unwrap();

        assert!(res_body.starts_with("Invalid request."), "Unexpected HTTP response error body");
    }
}