use std::{
    borrow::Borrow,
    fmt,
    fmt::{
        Display, Formatter,
    },
    io::Read,
};
use bitcoin_hashes::{
    Hash, HashEngine, hex::ToHex, Hmac,
    HmacEngine,
    sha256,
};
use reqwest::{
    header::{
        ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, CONTENT_ENCODING,
        HeaderMap, HeaderName, HeaderValue, HOST,
    },
    Url,
};
use async_compression::{
    Level,
    tokio_02::bufread::DeflateEncoder
};
use tokio::io::AsyncReadExt;
use time::{
    OffsetDateTime, Date, Duration,
};
use serde::de::DeserializeOwned;
use crate::*;

impl<'a> CatenisClient<'a> {
    // Definition of public methods

    pub fn new_async(api_access_secret: &'a str, device_id: &'a str) -> Result<Self>
    {
        let base_url = Url::parse(DEFAULT_BASE_URL)?;
        let api_version = DEFAULT_API_VERSION;
        let use_compression = true;
        let compress_threshold: usize = 1024;

        Ok(CatenisClient {
            api_access_secret,
            device_id,
            base_api_url: base_url.join(&Self::merge_url_params(API_BASE_URL_PATH, &[("version", api_version.to_string())]))?,
            use_compression,
            compress_threshold,
            sign_date: None,
            signing_key: None,
            http_client: None,
            http_client_async: Some(Self::new_http_client_async(use_compression)?),
        })
    }

    pub fn new_async_with_options<I>(api_access_secret: &'a str, device_id: &'a str, opts: I) -> Result<Self>
        where
            I: IntoIterator,
            <I as IntoIterator>::Item: Borrow<ClientOptions<'a>>
    {
        let mut base_url = Url::parse(DEFAULT_BASE_URL)?;
        let mut api_version = DEFAULT_API_VERSION;
        let mut use_compression = true;
        let mut compress_threshold: usize = 1024;

        for opt in opts.into_iter() {
            match opt.borrow() {
                ClientOptions::Host(host) => {
                    match Self::parse_host_with_port(host) {
                        (Some(host), port) => {
                            // Replace host
                            base_url.set_host(Some(&host)).unwrap_or(());

                            if let Some(port) = port {
                                // Replace port
                                base_url.set_port(Some(port)).unwrap_or(());
                            }
                        },
                        (None, _) => {
                            return Err(Error::new_client_error(Some("Invalid host"), None::<error::GenericError>));
                        }
                    }
                }
                ClientOptions::Environment(env) => {
                    if let Environment::Sandbox = env {
                        if let Some(host) = base_url.host_str() {
                            // Add proper subdomain to host
                            let orig_host = String::from(host);

                            base_url.set_host(Some(&(String::from("sandbox.") + &orig_host)))?;
                        } else {
                            return Err(Error::new_client_error(Some("Inconsistent URL: missing host"), None::<error::GenericError>));
                        }
                    }
                }
                ClientOptions::Secure(secure) => {
                    if !*secure {
                        // Replace scheme
                        if let Err(_) = base_url.set_scheme("http") {
                            return Err(Error::new_client_error(Some("Error resetting URL scheme"), None::<error::GenericError>));
                        }
                    }
                }
                ClientOptions::Version(version) => {
                    api_version = *version;
                }
                ClientOptions::UseCompression(compress) => {
                    use_compression = *compress;
                }
                ClientOptions::CompressThreshold(threshold) => {
                    compress_threshold = *threshold;
                }
            }
        }

        Ok(CatenisClient {
            api_access_secret,
            device_id,
            base_api_url: base_url.join(&Self::merge_url_params(API_BASE_URL_PATH, &[("version", api_version.to_string())]))?,
            use_compression,
            compress_threshold,
            sign_date: None,
            signing_key: None,
            http_client: None,
            http_client_async: Some(Self::new_http_client_async(use_compression)?),
        })
    }

    pub async fn log_message_async(&mut self, message: &str, options: Option<LogMessageOptions>) -> Result<LogMessageResult> {
        let body = LogMessageRequest {
            message: String::from(message),
            options
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request_async("messages/log", body_json, None::<KVList>, None::<KVList>).await?;

        let res = self.sign_and_send_request_async(req).await?;

        Ok(Self::parse_response_async::<LogMessageResponse>(res).await?.data)
    }

    // Definition of private methods

    async fn send_request_async(&mut self, req: reqwest::Request) -> Result<reqwest::Response> {
        self.check_sign_and_send_request_async(req, false).await
    }

    async fn sign_and_send_request_async(&mut self, req: reqwest::Request) -> Result<reqwest::Response> {
        self.check_sign_and_send_request_async(req, true).await
    }

    async fn check_sign_and_send_request_async(&mut self, mut req: reqwest::Request, sign_request: bool) -> Result<reqwest::Response> {
        if sign_request {
            self.sign_request_async(&mut req)?;
        }

        let res = self.http_client_async.as_ref()
            .expect("Trying to access uninitialized HTTP client")
            .execute(req)
            .await
            .map_err::<Error, _>(Into::into)?;

        if res.status().is_success() {
            Ok(res)
        } else {
            Err(Error::from_http_response_async(res).await)
        }
    }

    fn get_request_async<I, J, K, V>(&self, endpoint_url_path: &str, url_params: Option<I>, query_params: Option<J>) -> Result<reqwest::Request>
        where
            I: IntoIterator,
            J: IntoIterator,
            K: AsRef<str>,
            V: AsRef<str>,
            <I as IntoIterator>::Item: Borrow<(K, V)>,
            <J as IntoIterator>::Item: Borrow<(K, V)>
    {
        let mut endpoint_url_path = String::from(endpoint_url_path);

        if let Some(params) = url_params {
            endpoint_url_path = Self::merge_url_params(&endpoint_url_path, params);
        }

        let mut req_builder = self.http_client_async.as_ref()
            .expect("Trying to access uninitialized HTTP client")
            .get(self.base_api_url.join(&endpoint_url_path)?);

        if let Some(params) = query_params {
            req_builder = req_builder.query(&Self::assemble_query_params(params));
        }

        req_builder.build()
            .map_err(Into::into)
    }

    async fn post_request_async<I, J, K, V>(&self, endpoint_url_path: &str, body: String, url_params: Option<I>, query_params: Option<J>) -> Result<reqwest::Request>
        where
            I: IntoIterator,
            J: IntoIterator,
            K: AsRef<str>,
            V: AsRef<str>,
            <I as IntoIterator>::Item: Borrow<(K, V)>,
            <J as IntoIterator>::Item: Borrow<(K, V)>
    {
        let mut endpoint_url_path = String::from(endpoint_url_path);

        if let Some(params) = url_params {
            endpoint_url_path = Self::merge_url_params(&endpoint_url_path, params);
        }

        let mut req_builder = self.http_client_async.as_ref()
            .expect("Trying to access uninitialized HTTP client")
            .post(self.base_api_url.join(&endpoint_url_path)?);

        if body.len() > 0 {
            // Prepare to add body to request
            req_builder = req_builder.header(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            if self.use_compression && body.len() >= self.compress_threshold {
                // Add compressed body
                req_builder = req_builder.body(Self::compress_body_async(body).await?)
                    .header(CONTENT_ENCODING, HeaderValue::from_static("deflate"));
            } else {
                // Add plain body
                req_builder = req_builder.body(body)
            }
        }

        if let Some(params) = query_params {
            req_builder = req_builder.query(&Self::assemble_query_params(params));
        }

        req_builder.build()
            .map_err(Into::into)
    }

    fn sign_request_async(&mut self, req: &mut reqwest::Request) -> Result<()> {
        let mut new_headers = HeaderMap::new();
        let now;
        let timestamp;

        // Identify new headers that need to be added to HTTP request
        {
            let headers = req.headers();

            if !headers.contains_key(HOST) {
                // Prepare to add missing 'host' header to HTTP request
                if let Some(host) = Self::get_host_with_port(req.url()) {
                    new_headers.insert(HOST, host.parse()?);
                } else {
                    return Err(Error::new_client_error(Some("Inconsistent HTTP request: URL missing host"), None::<error::GenericError>));
                }
            }

            // Prepare to add custom 'x-bcot-timestamp' header to HTTP request
            now = OffsetDateTime::now_utc();
            timestamp = now.format("%Y%m%dT%H%M%SZ");
            new_headers.insert(X_BCOT_TIMESTAMP, timestamp.parse()?);
        }

        // Add headers to HTTP request
        {
            for (key, value) in new_headers.iter() {
                let val = value.clone();

                req.headers_mut().insert(key, val);
            }
        }

        // Prepare to sign HTTP request

        // 1. Assemble conformed request

        // 1.1. Add HTTP verb
        let mut conformed_request: String = req.method().to_string() + "\n";

        // 1.2. Add URL path
        conformed_request = conformed_request + &Self::get_url_path_with_query(req.url()) + "\n";

        // 1.3. Assemble and add essential headers
        {
            let essential_headers_list = [
                HOST,
                HeaderName::from_static(X_BCOT_TIMESTAMP)
            ];
            let mut essential_headers = String::from("");
            let headers = req.headers();

            for header_name in essential_headers_list.iter() {
                essential_headers = essential_headers + header_name.as_str() + ":" + headers.get(header_name).unwrap().to_str()? + "\n";
            }

            conformed_request = conformed_request + &essential_headers + "\n";
        }

        // 1.4. Hash HTTP request payload and add it
        let payload = if let Some(body) = req.body_mut() {
            body.as_bytes().expect("Unable to access request body; body not buffered")
        } else {
            b""
        };

        conformed_request = conformed_request + &sha256::Hash::hash(payload).to_hex() + "\n";

        // 2. Update sign date and signing key
        self.update_sign_date_and_key(&now);

        // 3. Assemble string to sign
        let scope = self.sign_date.unwrap().format("%Y%m%d") + "/ctn1_request";
        let string_to_sign = String::from("CTN1-HMAC-SHA256\n") + &timestamp + "\n"
            + &scope + "\n"
            + &sha256::Hash::hash(conformed_request.as_bytes()).to_hex() + "\n";

        // 4. Generate signature
        let mut hmac_engine = HmacEngine::<sha256::Hash>::new(&self.signing_key.unwrap());
        hmac_engine.input(string_to_sign.as_bytes());
        let signature = Hmac::<sha256::Hash>::from_engine(hmac_engine).to_hex();

        // Add 'authorization' header to HTTP request
        let value = String::from("CTN1-HMAC-SHA256 Credential=") + self.device_id + "/"
            + &scope + ",Signature=" + &signature;

        req.headers_mut().insert(AUTHORIZATION, value.parse()?);

        Ok(())
    }

    // Definition of private associated ("static") functions

    async fn parse_response_async<T: DeserializeOwned>(res: reqwest::Response) -> Result<T> {
        let body = res.text().await
            .map_err::<Error, _>(|e| Error::new_client_error(Some("Inconsistent Catenis API response"), Some(e)))?;

        serde_json::from_str(&body)
            .map_err::<Error, _>(|e| Error::new_client_error(Some("Inconsistent Catenis API response"), Some(e)))
    }

    fn new_http_client_async(use_compression: bool) -> reqwest::Result<reqwest::Client> {
        let mut client_builder = reqwest::ClientBuilder::new();

        // Prepare to add default HTTP headers
        let mut headers = HeaderMap::new();

        if use_compression {
            headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip"));

            // Make sure that compressed response body is automatically decompressed
            client_builder = client_builder.no_gzip();
        } else {
            // Make sure that response is not compressed
            client_builder = client_builder.no_gzip();
        }

        client_builder
            .default_headers(headers)
            .build()
    }

    async fn compress_body_async(body: String) -> Result<Vec<u8>> {
        let mut enc = DeflateEncoder::with_quality(body.as_bytes(), Level::Default);
        let mut enc_body = Vec::new();
        enc.read_to_end(&mut enc_body).await?;

        Ok(enc_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_compress_data_async() {
        use async_compression::tokio_02::bufread::{DeflateEncoder, DeflateDecoder};

        let str_2_decode = String::from("This is only a test");
        println!("String to compress: {}", str_2_decode);

        let mut enc = DeflateEncoder::with_quality(str_2_decode.as_bytes(), Level::Default);
        let mut enc_data = Vec::new();
        enc.read_to_end(&mut enc_data).await.unwrap();
        println!("Compressed data: {:?}", enc_data);

        let mut dec = DeflateDecoder::new(enc_data.as_slice());
        let mut orig_str = String::new();
        dec.read_to_string(&mut orig_str).await.unwrap();
        println!("Decompressed data: {}", orig_str);

        assert_eq!(str_2_decode, orig_str);
    }

    #[tokio::test]
    async fn it_call_log_message_async_api_method() {
        let api_access_secret = "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3";
        let device_id = "drc3XdxNtzoucpw9xiRp";

        let mut ctn_client = CatenisClient::new_async_with_options(
            api_access_secret,
            device_id,
            &[
                ClientOptions::Host("localhost:3000"),
                ClientOptions::Secure(false),
                ClientOptions::UseCompression(false)
            ],
        ).unwrap();

        println!(">>>>>> Instantiated Catenis API client (CUSTOM): {:?}", ctn_client);

        let result = ctn_client.log_message_async("Test message #3 (2020-11-20)", None).await;

        println!(">>>>>> Log Message result: {:?}", result);
    }
}