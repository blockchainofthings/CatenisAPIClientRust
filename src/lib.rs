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
    blocking as blk_reqwest,
    header::{
        ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, CONTENT_ENCODING,
        HeaderMap, HeaderName, HeaderValue, HOST,
    },
    Url,
};
use flate2::{
    Compression,
    bufread::{
        DeflateEncoder,
    }
};
use time::{
    OffsetDateTime, Date, Duration,
};
use serde::de::DeserializeOwned;

mod error;
mod date_time;
mod api;
mod notification;
#[cfg(feature = "async")]
mod async_impl;

use error::GenericError;

pub use error::{
    Error, Result,
};
pub use date_time::UtcDateTime;
pub use api::*;
pub use notification::*;

const X_BCOT_TIMESTAMP: &str = "x-bcot-timestamp";
const DEFAULT_BASE_URL: &str = "https://catenis.io/";
const API_BASE_URL_PATH: &str = "api/:version/";
const DEFAULT_API_VERSION: ApiVersion = ApiVersion(0, 10);
const SIGNATURE_VALIDITY_DAYS: u8 = 7;
const TIME_VARIATION_SECS: u8 = 5;

type KVList<'a> = &'a [(&'a str, &'a str)];

pub enum Environment {
    Prod,
    Sandbox,
}

#[derive(Copy, Clone)]
pub struct ApiVersion(u16, u16);

impl Display for ApiVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

pub enum ClientOptions<'a> {
    Host(&'a str),
    Environment(Environment),
    Secure(bool),
    Version(ApiVersion),
    UseCompression(bool),
    CompressThreshold(usize),
}

#[derive(Debug)]
pub struct CatenisClient<'a> {
    api_access_secret: &'a str,
    device_id: &'a str,
    base_api_url: Url,
    is_secure: bool,
    use_compression: bool,
    compress_threshold: usize,
    sign_date: Option<Date>,
    signing_key: Option<[u8; 32]>,
    http_client: Option<blk_reqwest::Client>,
    http_client_async: Option<reqwest::Client>,
}

impl<'a> CatenisClient<'a> {
    // Definition of public methods

    pub fn new(api_access_secret: &'a str, device_id: &'a str) -> Result<Self>
    {
        let base_url = Url::parse(DEFAULT_BASE_URL)?;
        let api_version = DEFAULT_API_VERSION;
        let is_secure = true;
        let use_compression = true;
        let compress_threshold: usize = 1024;

        Ok(CatenisClient {
            api_access_secret,
            device_id,
            base_api_url: base_url.join(&Self::merge_url_params(API_BASE_URL_PATH, &[("version", api_version.to_string())]))?,
            is_secure,
            use_compression,
            compress_threshold,
            sign_date: None,
            signing_key: None,
            http_client: Some(Self::new_http_client(use_compression)?),
            http_client_async: None,
        })
    }

    pub fn new_with_options<I>(api_access_secret: &'a str, device_id: &'a str, opts: I) -> Result<Self>
        where
            I: IntoIterator,
            <I as IntoIterator>::Item: Borrow<ClientOptions<'a>>
    {
        let mut base_url = Url::parse(DEFAULT_BASE_URL)?;
        let mut api_version = DEFAULT_API_VERSION;
        let mut is_secure = true;
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
                            return Err(Error::new_client_error(Some("Invalid host"), None::<GenericError>));
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
                            return Err(Error::new_client_error(Some("Inconsistent URL: missing host"), None::<GenericError>));
                        }
                    }
                }
                ClientOptions::Secure(secure) => {
                    is_secure = *secure;

                    if !is_secure {
                        // Replace scheme
                        if let Err(_) = base_url.set_scheme("http") {
                            return Err(Error::new_client_error(Some("Error resetting URL scheme"), None::<GenericError>));
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
            is_secure,
            use_compression,
            compress_threshold,
            sign_date: None,
            signing_key: None,
            http_client: Some(Self::new_http_client(use_compression)?),
            http_client_async: None,
        })
    }

    pub fn log_message(&mut self, message: &str, options: Option<LogMessageOptions>) -> Result<LogMessageResult> {
        let body = LogMessageRequest {
            message: String::from(message),
            options
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request("messages/log", body_json, None::<KVList>, None::<KVList>)?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<LogMessageResponse>(res)?.data)
    }

    pub fn new_ws_notify_channel(&'a mut self, notify_event: NotificationEvent) -> WsNotifyChannel {
        WsNotifyChannel::new(self, notify_event)
    }

    // Definition of private methods

    fn send_request(&mut self, req: blk_reqwest::Request) -> Result<blk_reqwest::Response> {
        self.check_sign_and_send_request(req, false)
    }

    fn sign_and_send_request(&mut self, req: blk_reqwest::Request) -> Result<blk_reqwest::Response> {
        self.check_sign_and_send_request(req, true)
    }

    fn check_sign_and_send_request(&mut self, mut req: blk_reqwest::Request, sign_request: bool) -> Result<blk_reqwest::Response> {
        if sign_request {
            self.sign_request(&mut req)?;
        }

        let res = self.http_client.as_ref()
            .expect("Trying to access uninitialized HTTP client")
            .execute(req)
            .map_err::<Error, _>(Into::into)?;

        if res.status().is_success() {
            Ok(res)
        } else {
            Err(Error::from_http_response(res))
        }
    }

    fn get_request<I, K, V, I2, K2, V2>(&self, endpoint_url_path: &str, url_params: Option<I>, query_params: Option<I2>) -> Result<blk_reqwest::Request>
        where
            I: IntoIterator,
            K: AsRef<str>,
            V: AsRef<str>,
            <I as IntoIterator>::Item: Borrow<(K, V)>,
            I2: IntoIterator,
            K2: AsRef<str>,
            V2: AsRef<str>,
            <I2 as IntoIterator>::Item: Borrow<(K2, V2)>,
    {
        let mut endpoint_url_path = String::from(endpoint_url_path);

        if let Some(params) = url_params {
            endpoint_url_path = Self::merge_url_params(&endpoint_url_path, params);
        }

        let mut req_builder = self.http_client.as_ref()
            .expect("Trying to access uninitialized HTTP client")
            .get(self.base_api_url.join(&endpoint_url_path)?);

        if let Some(params) = query_params {
            req_builder = req_builder.query(&Self::assemble_query_params(params));
        }

        req_builder.build()
            .map_err(Into::into)
    }

    fn post_request<I, K, V, I2, K2, V2>(&self, endpoint_url_path: &str, body: String, url_params: Option<I>, query_params: Option<I2>) -> Result<blk_reqwest::Request>
        where
            I: IntoIterator,
            K: AsRef<str>,
            V: AsRef<str>,
            <I as IntoIterator>::Item: Borrow<(K, V)>,
            I2: IntoIterator,
            K2: AsRef<str>,
            V2: AsRef<str>,
            <I2 as IntoIterator>::Item: Borrow<(K2, V2)>,
    {
        let mut endpoint_url_path = String::from(endpoint_url_path);

        if let Some(params) = url_params {
            endpoint_url_path = Self::merge_url_params(&endpoint_url_path, params);
        }

        let mut req_builder = self.http_client.as_ref()
            .expect("Trying to access uninitialized HTTP client")
            .post(self.base_api_url.join(&endpoint_url_path)?);

        if body.len() > 0 {
            // Prepare to add body to request
            req_builder = req_builder.header(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            if self.use_compression && body.len() >= self.compress_threshold {
                // Add compressed body
                req_builder = req_builder.body(Self::compress_body(body)?)
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

    fn get_ws_request<I, K, V>(&self, endpoint_url_path: &str, url_params: Option<I>) -> Result<blk_reqwest::Request>
        where
            I: IntoIterator,
            K: AsRef<str>,
            V: AsRef<str>,
            <I as IntoIterator>::Item: Borrow<(K, V)>,
    {
        let mut req = self.get_request(endpoint_url_path, url_params, None::<KVList>)?;

        // Replace URL scheme as appropriate
        if let Err(_) = req.url_mut().set_scheme(if self.is_secure {"wss"} else {"ws"}) {
            return Err(Error::new_client_error(Some("Error resetting URL scheme"), None::<GenericError>));
        }

        Ok(req)
    }

    fn sign_request(&mut self, req: &mut blk_reqwest::Request) -> Result<()> {
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
                    return Err(Error::new_client_error(Some("Inconsistent HTTP request: URL missing host"), None::<GenericError>));
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

    fn update_sign_date_and_key(&mut self, now: &OffsetDateTime) {
        let lower_bound_sign_date = (now.clone() - Duration::seconds(TIME_VARIATION_SECS as i64)).date() - Duration::days(SIGNATURE_VALIDITY_DAYS as i64);

        let need_to_update = if let None = self.sign_date {
            true
        } else if self.sign_date.unwrap() < lower_bound_sign_date {
            true
        } else {
            false
        };

        if need_to_update {
            self.sign_date = Some(now.date());

            // Generate new signing key
            let inner_key = String::from("CTN1") + self.api_access_secret;
            let mut hmac_engine = HmacEngine::<sha256::Hash>::new(inner_key.as_bytes());
            hmac_engine.input(self.sign_date.unwrap().format("%Y%m%d").as_bytes());
            let date_key = &Hmac::<sha256::Hash>::from_engine(hmac_engine)[..];

            let mut hmac_engine = HmacEngine::<sha256::Hash>::new(date_key);
            hmac_engine.input(b"ctn1_request");

            self.signing_key = Some(*Hmac::<sha256::Hash>::from_engine(hmac_engine).as_inner());
        }
    }

    // Definition of private associated ("static") functions

    fn parse_response<T: DeserializeOwned>(res: blk_reqwest::Response) -> Result<T> {
        let body = res.text()
            .map_err::<Error, _>(|e| Error::new_client_error(Some("Inconsistent Catenis API response"), Some(e)))?;

        serde_json::from_str(&body)
            .map_err::<Error, _>(|e| Error::new_client_error(Some("Inconsistent Catenis API response"), Some(e)))
    }

    fn new_http_client(use_compression: bool) -> reqwest::Result<blk_reqwest::Client> {
        let mut client_builder = blk_reqwest::ClientBuilder::new();

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

    fn merge_url_params<I, K, V>(url_path: &str, params: I) -> String
        where
            I: IntoIterator,
            K: AsRef<str>,
            V: AsRef<str>,
            <I as IntoIterator>::Item: Borrow<(K, V)>
    {
        let mut merged_url_path = String::from(url_path);

        for pair in params.into_iter() {
            let param = pair.borrow().0.as_ref();
            let value = pair.borrow().1.as_ref();
            merged_url_path = merged_url_path.replace(&(String::from(":") + param), value);
        }

        merged_url_path
    }

    #[inline]
    fn assemble_query_params<I, K, V>(query_params: I) -> Vec<(String, String)>
        where
            I: IntoIterator,
            K: AsRef<str>,
            V: AsRef<str>,
            <I as IntoIterator>::Item: Borrow<(K, V)>
    {
        let mut query_params_list = Vec::new();

        for pair in query_params.into_iter() {
            let param = String::from(pair.borrow().0.as_ref());
            let value = String::from(pair.borrow().1.as_ref());
            query_params_list.push((param, value));
        }

        query_params_list
    }

    fn parse_host_with_port(host: &str) -> (Option<String>, Option<u16>) {
        if let Ok(url) = Url::parse(&(String::from("http://") + host)) {
            let host = if let Some(val) = url.host_str() {
                let host = String::from(val);
                Some(host)
            } else {
                None
            };
            let port = if let Some(val) = url.port() { Some(val) } else { None };

            (host, port)
        } else {
            (None, None)
        }
    }

    fn get_host_with_port(url: &reqwest::Url) -> Option<String> {
        if let Some(host) = url.host_str() {
            let mut host = String::from(host);

            if let Some(port) = url.port() {
                host = host + ":" + &port.to_string();
            }

            Some(host)
        } else {
            None
        }
    }

    fn get_url_path_with_query(url: &reqwest::Url) -> String {
        let mut path = String::from(url.path());

        if let Some(query) = url.query() {
            path = path + "?" + query;
        }

        path
    }

    fn compress_body(body: String) -> Result<Vec<u8>> {
        let mut enc = DeflateEncoder::new(body.as_bytes(), Compression::default());
        let mut enc_body = Vec::new();
        enc.read_to_end(&mut enc_body)?;

        Ok(enc_body)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin_hashes::Hash;
    use bitcoin_hashes::hex;
    use bitcoin_hashes::hex::FromHex;
    use bitcoin_hashes::sha256;
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};

    use crate::*;
    use std::io::Read;

    #[derive(Debug, Serialize, Deserialize)]
    struct Customer {
        first_name: Option<String>,
        last_name: Option<String>,
        age: Option<u8>,
        country: Option<String>,
    }

    #[test]
    fn it_parses_json() {
        let json = json!({
            "last_name": "Castro",
            "country": "Brasil"
        }).to_string();

        //let cust : Customer = serde_json::from_str(json.as_str()).unwrap();
        let cust: Value = serde_json::from_str(json.as_str()).unwrap();

        println!("Resulting customer object: {:?}", cust);
    }

    #[test]
    fn it_does_hashes() {
        let hash: sha256::Hash = hex::FromHex::from_hex("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
        let hash2 = sha256::Hash::from_hex("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
        let hash3 = sha256::Hash::from_str("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();

        println!("Generated hash: {:?}", sha256::Hash::hash("".as_bytes()));
        println!("Hash bytes (into_inner): {:?}", hash.into_inner());
        println!("Hash bytes (as_inner): {:?}", hash.as_inner());
        println!("Hash bytes (as_ref): {:?}", hash.as_ref());
        assert_eq!(hash, hash2);
        assert_eq!(hash, hash3);
    }

    #[test]
    fn it_formats_date_time() {
        let str_time = OffsetDateTime::now_utc().format("%Y%m%dT%H%M%SZ");

        println!("Formatted date & time: {}", str_time);
    }

    #[test]
    fn it_replaces_string() {
        let s = String::from("This is a {!thing}");

        let new_s = s.replace("{!thing}", "test");

        println!("Original string: {}", s);
        println!("Processed string: {}", new_s);
    }

    #[test]
    fn it_init_options() {
        let host = String::from("localhost:3000");
        let _opts = [
            ClientOptions::Host(&host)
        ];
    }

    #[test]
    fn it_parse_hostname() {
        let url = Url::parse("http://localhost.com:3000").unwrap();

        println!("Parsed hostname URL: {:?}", url);
        println!("Parsed hostname URL: {:?}", url.host());
        println!("Parsed hostname URL: {:?}", url.port());
    }

    #[test]
    fn it_merge_url_params() {
        let merged_url = CatenisClient::merge_url_params(
            "/api/:version/messages/:messageid",
            &[
                ("version", "0.10"),
                ("messageid", "abcdefg")
            ],
        );

        println!("Merged URL: {}", merged_url);
    }

    #[test]
    fn it_compress_data() {
        use flate2::bufread::DeflateDecoder;

        let str_2_decode = String::from("This is only a test");
        println!("String to compress: {}", str_2_decode);

        let mut enc = DeflateEncoder::new(str_2_decode.as_bytes(), Compression::default());
        let mut enc_data = Vec::new();
        enc.read_to_end(&mut enc_data).unwrap();
        println!("Compressed data: {:?}", enc_data);

        let mut dec = DeflateDecoder::new(enc_data.as_slice());
        let mut orig_str = String::new();
        dec.read_to_string(&mut orig_str).unwrap();
        println!("Decompressed data: {}", orig_str);

        assert_eq!(str_2_decode, orig_str);
    }

    #[test]
    fn it_call_get_api_method() {
        let api_access_secret = "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3";
        let device_id = "drc3XdxNtzoucpw9xiRp";

        let ctn_client = CatenisClient::new(api_access_secret, device_id);

        println!(">>>>>> Instantiated Catenis API client: {:?}", ctn_client);

        let mut ctn_client = CatenisClient::new_with_options(
            api_access_secret,
            device_id,
            &[
                ClientOptions::Host("localhost:3000"),
                ClientOptions::Secure(false),
                ClientOptions::UseCompression(false)
            ],
        ).unwrap();

        println!(">>>>>> Instantiated Catenis API client (CUSTOM): {:?}", ctn_client);

        let mut req = ctn_client.get_request(
            "messages/:messageid",
            Some(&[("messageid", "mdDzw65xix5CJRMCTDKd")]),
            None::<KVList>, /*Some(&[
                ("encoding", "utf8"),
                ("async", "false")
            ])*/
        ).unwrap();

        println!(">>>>>> API GET method request: {:?}", req);

        ctn_client.sign_request(&mut req).unwrap();

        println!(">>>>>> API GET method request (SIGNED): {:?}", req);

        let res_result = ctn_client.http_client
            .expect("Trying to access uninitialized HTTP client")
            .execute(req);

        println!(">>>>>> API GET method response: {:?}", res_result);

        if let Ok(res) = res_result {
            let mut parse_json = false;

            if let Some(val) = res.headers().get(CONTENT_TYPE) {
                if let Ok(s) = val.to_str() {
                    parse_json = s.contains("json")
                }
            }

            if parse_json {
                if let Ok(json) = res.json::<serde_json::Value>() {
                    println!(">>>>>> API GET method response body: {}", json);
                }
            } else if let Ok(body) = res.text() {
                println!(">>>>>> API GET method response body: {}", body);
            }
        }
    }

    #[derive(Debug, Serialize)]
    struct SetPermRights {
        system: Option<String>,
    }

    #[test]
    fn it_call_post_api_method() {
        let api_access_secret = "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3";
        let device_id = "drc3XdxNtzoucpw9xiRp";

        let ctn_client = CatenisClient::new(api_access_secret, device_id);

        println!(">>>>>> Instantiated Catenis API client: {:?}", ctn_client);

        let mut ctn_client = CatenisClient::new_with_options(
            api_access_secret,
            device_id,
            &[
                ClientOptions::Host("localhost:3000"),
                ClientOptions::Secure(false),
                ClientOptions::UseCompression(false)
            ],
        ).unwrap();

        println!(">>>>>> Instantiated Catenis API client (CUSTOM): {:?}", ctn_client);

        let perm_rights = SetPermRights {
            system: Some(String::from("allow"))
        };
        let body = json!(perm_rights).to_string();

        let mut req = ctn_client.post_request(
            "permission/events/:eventName/rights",
            body,
            Some(&[("eventName", "receive-msg")]),
            None::<KVList>,
        ).unwrap();

        println!(">>>>>> API POST method request: {:?}", req);
        println!(">>>>>> API POST method request body: {:?}", req.body());

        ctn_client.sign_request(&mut req).unwrap();

        println!(">>>>>> API POST method request (SIGNED): {:?}", req);

        let res_result = ctn_client.http_client
            .expect("Trying to access uninitialized HTTP client")
            .execute(req);

        println!(">>>>>> API POST method response: {:?}", res_result);

        if let Ok(res) = res_result {
            let mut parse_json = false;

            if let Some(val) = res.headers().get(CONTENT_TYPE) {
                if let Ok(s) = val.to_str() {
                    parse_json = s.contains("json")
                }
            }

            if parse_json {
                if let Ok(json) = res.json::<serde_json::Value>() {
                    println!(">>>>>> API POST method response body: {}", json);
                }
            } else if let Ok(body) = res.text() {
                println!(">>>>>> API POST method response body: {}", body);
            }
        }
    }

    #[test]
    fn it_call_log_message_api_method() {
        let api_access_secret = "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3";
        let device_id = "drc3XdxNtzoucpw9xiRp";

        let mut ctn_client = CatenisClient::new_with_options(
            api_access_secret,
            device_id,
            &[
                ClientOptions::Host("localhost:3000"),
                ClientOptions::Secure(false),
                ClientOptions::UseCompression(false)
            ],
        ).unwrap();

        println!(">>>>>> Instantiated Catenis API client (CUSTOM): {:?}", ctn_client);

        let result = ctn_client.log_message("Test message #2 (2020-11-20)", None);

        println!(">>>>>> Log Message result: {:?}", result);
    }

    #[test]
    fn it_get_notify_channel() {
        let api_access_secret = "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3";
        let device_id = "drc3XdxNtzoucpw9xiRp";

        let mut ctn_client = CatenisClient::new_with_options(
            api_access_secret,
            device_id,
            &[
                ClientOptions::Host("localhost:3000"),
                ClientOptions::Secure(false),
                ClientOptions::UseCompression(false)
            ],
        ).unwrap();

        let notify_channel = ctn_client.new_ws_notify_channel(NotificationEvent::NewMsgReceived);

        println!(">>>>>> WS Notify channel: {:?}", notify_channel);
    }
}