use reqwest::{
    blocking::{
        Response,
    },
    StatusCode,
};
use serde::Deserialize;
use serde_json;
use std::{
    error, fmt, result,
};

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct CatenisErrorResponse {
    pub status: String,
    pub message: String,
}

/// Result type returned by Catenis API client functions.
pub type Result<T> = result::Result<T, Error>;
pub(crate) type GenericError = Box<dyn error::Error + Send + Sync>;

/// Represents an error that took place when calling a Catenis API client function.
///
/// That error can be of two different types:
///
/// # Client error
///
/// An error that takes place in the client itself, and could be of many different kinds.
///
/// # Catenis API error
///
/// A response returned by the Catenis API server reporting an error condition while processing the
/// client request.
///
/// ## Example
///
/// ```no_run
/// # use catenis_api_client::{
/// #     CatenisClient, ClientOptions, Environment, Result,
/// # };
/// #
/// # fn main() -> Result<()> {
/// # let mut ctn_client = CatenisClient::new_with_options(
/// #     Some((
/// #         "drc3XdxNtzoucpw9xiRp",
/// #         "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
/// #     ).into()),
/// #     &[
/// #         ClientOptions::Environment(Environment::Sandbox),
/// #     ],
/// # )?;
/// // Simulate a Catenis API error trying to read a message with an invalid message ID
/// let error = ctn_client.read_message("xxxxx", None).err().unwrap();
///
/// assert!(error.is_api_error());
///
/// let api_error = error.api_error_info().unwrap();
///
/// assert_eq!(api_error.status_code(), 400);
/// assert_eq!(api_error.status_message().unwrap(), "Bad Request");
/// assert_eq!(api_error.catenis_message().unwrap(), "[400] - Invalid message ID");
///
/// assert_eq!(error.to_string(), "Catenis API error: [400] - Invalid message ID");
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Error {
    inner: Box<Inner>
}

#[derive(Debug)]
struct Inner {
    kind: ErrorKind,
    source: Option<GenericError>,
}

#[derive(Debug)]
enum ErrorKind {
    Client(Option<String>),
    Api(ApiErrorInfo),
}

/// Detailed information about a received Catenis API error.
#[derive(Debug)]
pub struct ApiErrorInfo {
    http_status_code: StatusCode,
    text_message: Option<String>,
    ctn_message: Option<String>,
}

impl ApiErrorInfo {
    /// The received HTTP response's status code.
    pub fn status_code(&self) -> u16 {
        self.http_status_code.as_u16()
    }

    /// The generic reason for the received HTTP response's status code.
    pub fn status_message(&self) -> Option<&str> {
        self.http_status_code.canonical_reason()
    }

    /// The contents of the received HTTP response's body.
    pub fn body_message(&self) -> Option<&str> {
        if let Some(msg) = &self.text_message {
            Some(msg.as_str())
        } else {
            None
        }
    }

    /// The error message returned by the Catenis system.
    pub fn catenis_message(&self) -> Option<&str> {
        if let Some(msg) = &self.ctn_message {
            Some(msg.as_str())
        } else {
            None
        }
    }

    /// The formatted message representing the API error.
    pub fn error_message(&self) -> String {
        let description = if let Some(msg) = self.catenis_message() {
            msg
        } else if let Some(msg) = self.body_message() {
            msg
        } else if let Some(msg) = self.status_message() {
            msg
        } else {
            ""
        };

        format!("[{}] - {}", self.status_code(), description)
    }
}

impl Error {
    pub(crate) fn new_client_error<E>(message: Option<&str>, source: Option<E>) -> Self
        where
            E: Into<GenericError>
    {
        Error {
            inner: Box::new(Inner {
                kind: ErrorKind::Client(message.map(|s| String::from(s))),
                source: source.map(Into::into),
            })
        }
    }

    pub(crate) fn new_api_error(http_status_code: StatusCode, text_message: Option<&str>, ctn_message: Option<&str>) -> Self {
        Error {
            inner: Box::new(Inner {
                kind: ErrorKind::Api(ApiErrorInfo {
                    http_status_code,
                    text_message: text_message.map(|s| String::from(s)),
                    ctn_message: ctn_message.map(|s| String::from(s)),
                }),
                source: None,
            })
        }
    }

    pub(crate) fn from_http_response(res: Response) -> Self {
        let http_status_code = res.status();

        if http_status_code.is_success() {
            return Self::new_client_error(Some("Trying to process successful http response as an error"), None::<GenericError>);
        }

        // Try to retrieve response body
        let mut res_body = None;
        let inner_res_body;

        if let Ok(text) = res.text() {
            inner_res_body = text;
            res_body = Some(inner_res_body);
        }

        let mut ctn_message = None;
        let mut text_message = None;
        let inner_ctn_message;
        let inner_text_message;

        if let Some(body) = res_body {
            // Try to parse Catenis error response from body
            if let Ok(err_res) = serde_json::from_str::<CatenisErrorResponse>(&body) {
                inner_ctn_message = err_res.message;
                ctn_message = Some(inner_ctn_message.as_str());
            }

            if let None = ctn_message {
                // No valid Catenis error response. Prepare to pass body message
                inner_text_message = body;
                text_message = Some(inner_text_message.as_str());
            }
        }

        Self::new_api_error(http_status_code, text_message, ctn_message)
    }

    /// Indicates whether this is a Catenis API error.
    pub fn is_api_error(&self) -> bool {
        if let ErrorKind::Api(_) = self.inner.kind {
            true
        } else {
            false
        }
    }

    /// Retrieves information about the Catenis API error.
    pub fn api_error_info(&self) -> Option<&ApiErrorInfo> {
        if let ErrorKind::Api(error_info) = &self.inner.kind {
            Some(error_info)
        } else {
            None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner.kind {
            ErrorKind::Client(client_msg) => {
                let mut error_message = String::from("Catenis client error");

                if let Some(msg) = client_msg {
                    error_message = error_message + ": " + msg;
                }

                if let Some(source_error) = &self.inner.source {
                    error_message = error_message + ": " + &source_error.to_string();
                }

                write!(f, "{}", error_message)
            }
            ErrorKind::Api(error_info) => write!(f, "Catenis API error: {}", error_info.error_message())
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.inner.source.as_ref().map(|e| &**e as _)
    }
}

// Auto-conversion from different Error types

#[doc(hidden)]
pub trait ConvertibleError {}

// Just add a new line for the desired Error type below
impl ConvertibleError for std::io::Error {}
impl ConvertibleError for reqwest::Error {}
impl ConvertibleError for reqwest::header::InvalidHeaderValue {}
impl ConvertibleError for reqwest::header::ToStrError {}
impl ConvertibleError for url::ParseError {}
impl ConvertibleError for serde_json::Error {}
impl ConvertibleError for tungstenite::Error {}
impl ConvertibleError for regex::Error {}
impl ConvertibleError for std::num::ParseFloatError {}
impl ConvertibleError for time::ParseError {}

impl<E> From<E> for Error
    where
        E: ConvertibleError + Into<GenericError>
{
    fn from(err: E) -> Error {
        Error::new_client_error(None, Some(err))
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use crate::{
        test_helper:: {
            http_server::{
                HttpServer, HttpServerMode, HttpBody,
            },
        },
    };

    use super::*;

    #[test]
    fn it_deserialize_catenis_error_response() {
        let json = r#"{"status":"error","message":"Internal server error"}"#;

        let catenis_error_response: CatenisErrorResponse = serde_json::from_str(json).unwrap();

        assert_eq!(catenis_error_response, CatenisErrorResponse {
            status: String::from("error"),
            message: String::from("Internal server error"),
        });
    }

    fn gen_result_io_error() -> std::result::Result<i32, io::Error> {
        Err(io::Error::new(io::ErrorKind::Other, "Custom I/O error"))
    }

    fn local_proc() -> Result<i32> {
        gen_result_io_error()?;

        Ok(0)
    }

    #[test]
    fn it_convert_error() {
        let proc_result = local_proc();

        assert_eq!(proc_result.is_err(), true);
        assert_eq!(proc_result.err().unwrap().to_string(), "Catenis client error: Custom I/O error");
    }

    #[test]
    fn it_generate_client_error_no_source() {
        let err = Error::new_client_error(None, None::<GenericError>);

        assert_eq!(err.is_api_error(), false);
        assert_eq!(err.to_string(), "Catenis client error");
    }

    #[test]
    fn it_generate_custom_client_error_no_source() {
        let err = Error::new_client_error(Some("Sample client error description"), None::<GenericError>);

        assert_eq!(err.is_api_error(), false);
        assert_eq!(err.to_string(), "Catenis client error: Sample client error description");
    }

    #[test]
    fn it_generate_client_error_with_source() {
        let source_err = gen_result_io_error().err().unwrap();
        let err = Error::new_client_error(None, Some(source_err));

        assert_eq!(err.is_api_error(), false);
        assert_eq!(err.to_string(), "Catenis client error: Custom I/O error");
    }

    #[test]
    fn it_generate_custom_client_error_with_source() {
        let source_err = gen_result_io_error().err().unwrap();
        let err = Error::new_client_error(Some("Sample client error description"), Some(source_err));

        assert_eq!(err.is_api_error(), false);
        assert_eq!(err.to_string(), "Catenis client error: Sample client error description: Custom I/O error");
    }

    #[test]
    fn it_generate_api_error_not_ctn() {
        let err = Error::new_api_error(StatusCode::BAD_REQUEST, None, None);

        assert_eq!(err.is_api_error(), true);
        assert_eq!(err.to_string(), "Catenis API error: [400] - Bad Request");
    }

    #[test]
    fn it_generate_custom_api_error_not_ctn() {
        let err = Error::new_api_error(StatusCode::BAD_REQUEST, Some("Custom HTTP error message"), None);

        assert_eq!(err.is_api_error(), true);
        assert_eq!(err.to_string(), "Catenis API error: [400] - Custom HTTP error message");
    }

    #[test]
    fn it_generate_api_error_ctn() {
        let err = Error::new_api_error(StatusCode::BAD_REQUEST, None, Some("Sample Catenis error message"));

        assert_eq!(err.is_api_error(), true);
        assert_eq!(err.to_string(), "Catenis API error: [400] - Sample Catenis error message");
    }

    #[test]
    fn it_generate_custom_api_error_ctn() {
        let err = Error::new_api_error(StatusCode::BAD_REQUEST, Some("Custom HTTP error message"), Some("Sample Catenis error message"));

        assert_eq!(err.is_api_error(), true);
        assert_eq!(err.to_string(), "Catenis API error: [400] - Sample Catenis error message");
    }

    #[test]
    fn it_generate_from_http_response() {
        // Simulate Catenis API error

        // Start HTTP server in error simulation mode
        let http_server = HttpServer::new(
            HttpServerMode::Error(
                500,
                Some(HttpBody::from_json(r#"{"status":"error","message":"Internal server error"}"#).unwrap()),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Send (any) HTTP request and get the response
        let res = reqwest::blocking::get(&format!("http://localhost:{}/", server_port)).unwrap();

        assert_eq!(res.status().is_success(), false);

        let err = Error::from_http_response(res);

        assert_eq!(err.is_api_error(), true);
        assert_eq!(err.to_string(), "Catenis API error: [500] - Internal server error");
    }

    #[test]
    fn it_try_generate_from_http_response() {
        // Simulate successful Catenis API response

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

        assert_eq!(res.status().is_success(), true);

        let err = Error::from_http_response(res);

        assert_eq!(err.is_api_error(), false);
        assert_eq!(err.to_string(), "Catenis client error: Trying to process successful http response as an error");
    }
}