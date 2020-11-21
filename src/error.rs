use reqwest::{
    blocking as blk_reqwest,
    StatusCode,
};
use serde::Deserialize;
use serde_json;
use std::{
    error, fmt, result,
};

#[derive(Debug, Deserialize)]
pub struct CatenisErrorResponse {
    pub status: String,
    pub message: String,
}

pub type Result<T> = result::Result<T, Error>;
pub(crate) type GenericError = Box<dyn error::Error + Send + Sync>;

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

#[derive(Debug)]
pub struct ApiErrorInfo {
    http_status_code: StatusCode,
    text_message: Option<String>,
    ctn_message: Option<String>,
}

impl ApiErrorInfo {
    pub fn status_code(&self) -> u16 {
        self.http_status_code.as_u16()
    }

    pub fn status_message(&self) -> Option<&str> {
        self.http_status_code.canonical_reason()
    }

    pub fn body_message(&self) -> Option<&str> {
        if let Some(msg) = &self.text_message {
            Some(msg.as_str())
        } else {
            None
        }
    }

    pub fn catenis_message(&self) -> Option<&str> {
        if let Some(msg) = &self.ctn_message {
            Some(msg.as_str())
        } else {
            None
        }
    }

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

    pub(crate) fn from_http_response(res: blk_reqwest::Response) -> Self {
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

    pub fn is_api_error(&self) -> bool {
        if let ErrorKind::Api(_) = self.inner.kind {
            true
        } else {
            false
        }
    }

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

pub trait ConvertibleError {}

// Just add a new line for the desired Error type below
impl ConvertibleError for std::io::Error {}
impl ConvertibleError for reqwest::Error {}
impl ConvertibleError for reqwest::header::InvalidHeaderValue {}
impl ConvertibleError for reqwest::header::ToStrError {}
impl ConvertibleError for url::ParseError {}
impl ConvertibleError for serde_json::Error {}

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

    use super::*;

    fn gen_io_error() -> std::result::Result<i32, io::Error> {
        Err(io::Error::new(io::ErrorKind::Other, "Oh no!"))
    }

    fn local_proc() -> Result<i32> {
        //gen_io_error()?;
        let err_msg = String::from("Only a test");

        return Err(Error::new_client_error(Some(err_msg.as_str()), None::<GenericError>));

        Ok(5)
    }

    #[test]
    fn it_convert_errors() {
        let result = local_proc();

        println!("Local processing result: {}", result.err().unwrap());
    }

    #[test]
    fn it_generate_api_error() {
        Error::new_api_error(StatusCode::BAD_REQUEST, Some("Sample text message"), Some("Sample Catenis error message"));
    }
}