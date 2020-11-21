use reqwest::Response;
use crate::error::*;

impl Error {
    pub(crate) async fn from_http_response_async(res: reqwest::Response) -> Self {
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