use reqwest::{
    Response,
};

use crate::error::*;

impl Error {
    pub(crate) async fn from_http_response_async(res: Response) -> Self {
        let http_status_code = res.status();

        if http_status_code.is_success() {
            return Self::new_client_error(Some("Trying to process successful http response as an error"), None::<GenericError>);
        }

        // Try to retrieve response body
        let mut res_body = None;
        let inner_res_body;

        if let Ok(text) = res.text().await {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_helper::{
            http_server::{
                HttpServer, HttpServerMode, HttpBody,
            },
        },
    };

    #[tokio::test]
    async fn it_generate_from_http_response() {
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
        let res = reqwest::get(&format!("http://localhost:{}/", server_port)).await.unwrap();

        assert_eq!(res.status().is_success(), false);

        let err = Error::from_http_response_async(res).await;

        assert_eq!(err.is_api_error(), true);
        assert_eq!(err.to_string(), "Catenis API error: [500] - Internal server error");
    }

    #[tokio::test]
    async fn it_try_generate_from_http_response() {
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
        let res = reqwest::get(&format!("http://localhost:{}/", server_port)).await.unwrap();

        assert_eq!(res.status().is_success(), true);

        let err = Error::from_http_response_async(res).await;

        assert_eq!(err.is_api_error(), false);
        assert_eq!(err.to_string(), "Catenis client error: Trying to process successful http response as an error");
    }
}