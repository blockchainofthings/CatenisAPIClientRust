use std::{
    borrow::Borrow,
    io::Read,
};
use bitcoin_hashes::{
    Hash, HashEngine, hex::ToHex, Hmac,
    HmacEngine,
    sha256,
};
use reqwest::{
    blocking::{
        Client as HttpClient,
        ClientBuilder as HttpClientBuilder,
        Request, Response
    },
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
    OffsetDateTime, Date,
};
use serde::de::DeserializeOwned;

use crate::*;
use base_client::BaseCatenisClient;

#[derive(Debug, Clone)]
pub struct CatenisClient {
    api_access_secret: String,
    device_id: String,
    base_api_url: Url,
    is_secure: bool,
    use_compression: bool,
    compress_threshold: usize,
    sign_date: Option<Date>,
    signing_key: Option<[u8; 32]>,
    http_client: Option<HttpClient>,
}

impl BaseCatenisClient for CatenisClient {
    fn get_api_access_secret_ref(&self) -> &String {
        &self.api_access_secret
    }

    fn get_sign_date_ref(&self) -> &Option<Date> {
        &self.sign_date
    }

    fn get_sign_date_mut_ref(&mut self) -> &mut Option<Date> {
        &mut self.sign_date
    }

    fn get_signing_key_mut_ref(&mut self) -> &mut Option<[u8; 32]> {
        &mut self.signing_key
    }
}

impl CatenisClient {
    // Definition of public methods

    pub fn new(api_access_secret: &str, device_id: &str) -> Result<Self>
    {
        let base_url = Url::parse(DEFAULT_BASE_URL)?;
        let api_version = DEFAULT_API_VERSION;
        let is_secure = true;
        let use_compression = true;
        let compress_threshold: usize = 1024;

        Ok(CatenisClient {
            api_access_secret: String::from(api_access_secret),
            device_id: String::from(device_id),
            base_api_url: base_url.join(&Self::merge_url_params(API_BASE_URL_PATH, &[("version", api_version.to_string())]))?,
            is_secure,
            use_compression,
            compress_threshold,
            sign_date: None,
            signing_key: None,
            http_client: Some(Self::new_http_client(use_compression)?),
        })
    }

    pub fn new_with_options<'a, I>(api_access_secret: &str, device_id: &str, opts: I) -> Result<Self>
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
            api_access_secret: String::from(api_access_secret),
            device_id: String::from(device_id),
            base_api_url: base_url.join(&Self::merge_url_params(API_BASE_URL_PATH, &[("version", api_version.to_string())]))?,
            is_secure,
            use_compression,
            compress_threshold,
            sign_date: None,
            signing_key: None,
            http_client: Some(Self::new_http_client(use_compression)?),
        })
    }

    pub fn new_ws_notify_channel(&self, notify_event: NotificationEvent) -> WsNotifyChannel {
        WsNotifyChannel::new(self, notify_event)
    }

    pub fn log_message(&mut self, message: &str, options: Option<LogMessageOptions>) -> Result<LogMessageResult> {
        let body = LogMessageRequest {
            message: String::from(message),
            options
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request(
            "messages/log",
            body_json,
            None::<KVList>,
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<LogMessageResponse>(res)?.data)
    }

    pub fn log_chunked_message(&mut self, message: ChunkedMessage, options: Option<LogMessageOptions>) -> Result<LogMessageResult> {
        let body = LogChunkedMessageRequest {
            message,
            options
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request(
            "messages/log",
            body_json,
            None::<KVList>,
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<LogMessageResponse>(res)?.data)
    }

    pub fn send_message(&mut self, message: &str, target_device: DeviceId, options: Option<SendMessageOptions>) -> Result<SendMessageResult> {
        let body = SendMessageRequest {
            message: String::from(message),
            target_device,
            options
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request(
            "messages/send",
            body_json,
            None::<KVList>,
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<SendMessageResponse>(res)?.data)
    }

    pub fn send_chunked_message(&mut self, message: ChunkedMessage, target_device: DeviceId, options: Option<SendMessageOptions>) -> Result<SendMessageResult> {
        let body = SendChunkedMessageRequest {
            message,
            target_device,
            options
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request(
            "messages/send",
            body_json,
            None::<KVList>,
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<SendMessageResponse>(res)?.data)
    }

    pub fn read_message(&mut self, message_id: &str, options: Option<ReadMessageOptions>) -> Result<ReadMessageResult> {
        // Prepare query parameters
        let mut params_vec = Vec::new();
        let encoding;
        let continuation_token;
        let data_chunk_size;
        let async_;
        let mut query_params = None;

        if let Some(opt) = options {
            if let Some(val) = opt.encoding {
                encoding = val.to_string();

                params_vec.push(("encoding", encoding.as_str()));
            }

            if let Some(val) = opt.continuation_token {
                continuation_token = val.to_string();

                params_vec.push(("continuationToken", continuation_token.as_str()));
            }

            if let Some(val) = opt.data_chunk_size {
                data_chunk_size = val.to_string();

                params_vec.push(("dataChunkSize", data_chunk_size.as_str()));
            }

            if let Some(val) = opt.async_ {
                async_ = val.to_string();

                params_vec.push(("async", async_.as_str()));
            }
        }

        if params_vec.len() > 0 {
            query_params = Some(params_vec.as_slice());
        }

        let req = self.get_request(
            "messages/:message_id",
            Some(&[
                ("message_id", message_id),
            ]),
            query_params,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<ReadMessageResponse>(res)?.data)
    }

    pub fn retrieve_message_container(&mut self, message_id: &str) -> Result<RetrieveMessageContainerResult> {
        let req = self.get_request(
            "messages/:message_id/container",
            Some(&[
                ("message_id", message_id),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<RetrieveMessageContainerResponse>(res)?.data)
    }

    pub fn retrieve_message_origin(&self, message_id: &str, msg_to_sign: Option<&str>) -> Result<RetrieveMessageOriginResult> {
        // Prepare query parameters
        let mut params_vec = Vec::new();
        let mut query_params = None;

        if let Some(msg) = msg_to_sign {
            params_vec.push(("msgToSign", msg));
        }

        if params_vec.len() > 0 {
            query_params = Some(params_vec.as_slice());
        }

        let req = self.get_request(
            "messages/:message_id/origin",
            Some(&[
                ("message_id", message_id),
            ][..]),
            query_params,
        )?;

        let res = self.send_request(req)?;

        Ok(Self::parse_response::<RetrieveMessageOriginResponse>(res)?.data)
    }

    pub fn retrieve_message_progress(&mut self, message_id: &str) -> Result<RetrieveMessageProgressResult> {
        let req = self.get_request(
            "messages/:message_id/progress",
            Some(&[
                ("message_id", message_id),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<RetrieveMessageProgressResponse>(res)?.data)
    }

    pub fn list_messages(&mut self, options: Option<ListMessagesOptions>) -> Result<ListMessagesResult> {
        // Prepare query parameters
        let mut params_vec = Vec::new();
        let action;
        let direction;
        let from_device_ids;
        let from_device_prod_unique_ids;
        let to_device_ids;
        let to_device_prod_unique_ids;
        let read_state;
        let start_date;
        let end_date;
        let limit;
        let skip;
        let mut query_params = None;

        if let Some(opt) = options {
            if let Some(val) = opt.action {
                action = val.to_string();

                params_vec.push(("action", action.as_str()));
            }

            if let Some(val) = opt.direction {
                direction = val.to_string();

                params_vec.push(("direction", direction.as_str()));
            }

            {
                let mut ids = Vec::new();
                let mut prod_unique_ids = Vec::new();

                if let Some(vec) = opt.from_devices {
                    for device_id in vec {
                        if let Some(true) = device_id.is_prod_unique_id {
                            prod_unique_ids.push(device_id.id);
                        } else {
                            ids.push(device_id.id);
                        }
                    }
                }

                if ids.len() > 0 {
                    from_device_ids = String::from(ids.join(","));

                    params_vec.push(("fromDeviceIds", from_device_ids.as_str()));
                }

                if prod_unique_ids.len() > 0 {
                    from_device_prod_unique_ids = String::from(prod_unique_ids.join(","));

                    params_vec.push(("fromDeviceProdUniqueIds", from_device_prod_unique_ids.as_str()));
                }
            }

            {
                let mut ids = Vec::new();
                let mut prod_unique_ids = Vec::new();

                if let Some(vec) = opt.to_devices {
                    for device_id in vec {
                        if let Some(true) = device_id.is_prod_unique_id {
                            prod_unique_ids.push(device_id.id);
                        } else {
                            ids.push(device_id.id);
                        }
                    }
                }

                if ids.len() > 0 {
                    to_device_ids = String::from(ids.join(","));

                    params_vec.push(("toDeviceIds", to_device_ids.as_str()));
                }

                if prod_unique_ids.len() > 0 {
                    to_device_prod_unique_ids = String::from(prod_unique_ids.join(","));

                    params_vec.push(("toDeviceProdUniqueIds", to_device_prod_unique_ids.as_str()));
                }
            }

            if let Some(val) = opt.read_state {
                read_state = val.to_string();

                params_vec.push(("readState", read_state.as_str()));
            }

            if let Some(val) = opt.start_date {
                start_date = val.to_string();

                params_vec.push(("startDate", start_date.as_str()));
            }

            if let Some(val) = opt.end_date {
                end_date = val.to_string();

                params_vec.push(("endDate", end_date.as_str()));
            }

            if let Some(val) = opt.limit {
                limit = val.to_string();

                params_vec.push(("limit", limit.as_str()));
            }

            if let Some(val) = opt.skip {
                skip = val.to_string();

                params_vec.push(("skip", skip.as_str()));
            }
        }

        if params_vec.len() > 0 {
            query_params = Some(params_vec.as_slice());
        }

        let req = self.get_request(
            "messages",
            None::<KVList>,
            query_params,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<ListMessagesResponse>(res)?.data)
    }

    pub fn issue_asset(&mut self, asset_info: NewAssetInfo, amount: f64, holding_device: Option<DeviceId>) -> Result<IssueAssetResult> {
        let body = IssueAssetRequest {
            asset_info,
            amount,
            holding_device
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request(
            "assets/issue",
            body_json,
            None::<KVList>,
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<IssueAssetResponse>(res)?.data)
    }

    pub fn reissue_asset(&mut self, asset_id: &str, amount: f64, holding_device: Option<DeviceId>) -> Result<ReissueAssetResult> {
        let body = ReissueAssetRequest {
            amount,
            holding_device
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request(
            "assets/:asset_id/issue",
            body_json,
            Some(&[
                ("asset_id", asset_id)
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<ReissueAssetResponse>(res)?.data)
    }

    pub fn transfer_asset(&mut self, asset_id: &str, amount: f64, receiving_device: DeviceId) -> Result<TransferAssetResult> {
        let body = TransferAssetRequest {
            amount,
            receiving_device
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request(
            "assets/:asset_id/transfer",
            body_json,
            Some(&[
                ("asset_id", asset_id)
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<TransferAssetResponse>(res)?.data)
    }

    pub fn retrieve_asset_info(&mut self, asset_id: &str) -> Result<RetrieveAssetInfoResult> {
        let req = self.get_request(
            "assets/:asset_id",
            Some(&[
                ("asset_id", asset_id),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<RetrieveAssetInfoResponse>(res)?.data)
    }

    pub fn get_asset_balance(&mut self, asset_id: &str) -> Result<GetAssetBalanceResult> {
        let req = self.get_request(
            "assets/:asset_id/balance",
            Some(&[
                ("asset_id", asset_id),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<GetAssetBalanceResponse>(res)?.data)
    }

    pub fn list_owned_assets(&mut self, limit: Option<u16>, skip: Option<usize>) -> Result<ListOwnedAssetsResult> {
        // Prepare query parameters
        let mut params_vec = Vec::new();
        let limit_str;
        let skip_str;
        let mut query_params = None;

        if let Some(val) = limit {
            limit_str = val.to_string();

            params_vec.push(("limit", limit_str.as_str()));
        }

        if let Some(val) = skip {
            skip_str = val.to_string();

            params_vec.push(("skip", skip_str.as_str()));
        }

        if params_vec.len() > 0 {
            query_params = Some(params_vec.as_slice());
        }

        let req = self.get_request(
            "assets/owned",
            None::<KVList>,
            query_params,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<ListOwnedAssetsResponse>(res)?.data)
    }

    pub fn list_issued_assets(&mut self, limit: Option<u16>, skip: Option<usize>) -> Result<ListIssuedAssetsResult> {
        // Prepare query parameters
        let mut params_vec = Vec::new();
        let limit_str;
        let skip_str;
        let mut query_params = None;

        if let Some(val) = limit {
            limit_str = val.to_string();

            params_vec.push(("limit", limit_str.as_str()));
        }

        if let Some(val) = skip {
            skip_str = val.to_string();

            params_vec.push(("skip", skip_str.as_str()));
        }

        if params_vec.len() > 0 {
            query_params = Some(params_vec.as_slice());
        }

        let req = self.get_request(
            "assets/issued",
            None::<KVList>,
            query_params,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<ListIssuedAssetsResponse>(res)?.data)
    }

    pub fn retrieve_asset_issuance_history(
        &mut self,
        asset_id: &str,
        start_date: Option<UtcDateTime>,
        end_date: Option<UtcDateTime>,
        limit: Option<u16>,
        skip: Option<usize>,
    ) -> Result<RetrieveAssetIssuanceHistoryResult> {
        // Prepare query parameters
        let mut params_vec = Vec::new();
        let start_date_str;
        let end_date_str;
        let limit_str;
        let skip_str;
        let mut query_params = None;

        if let Some(val) = start_date {
            start_date_str = val.to_string();

            params_vec.push(("startDate", start_date_str.as_str()));
        }

        if let Some(val) = end_date {
            end_date_str = val.to_string();

            params_vec.push(("endDate", end_date_str.as_str()));
        }

        if let Some(val) = limit {
            limit_str = val.to_string();

            params_vec.push(("limit", limit_str.as_str()));
        }

        if let Some(val) = skip {
            skip_str = val.to_string();

            params_vec.push(("skip", skip_str.as_str()));
        }

        if params_vec.len() > 0 {
            query_params = Some(params_vec.as_slice());
        }

        let req = self.get_request(
            "assets/:asset_id/issuance",
            Some(&[
                ("asset_id", asset_id),
            ]),
            query_params,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<RetrieveAssetIssuanceHistoryResponse>(res)?.data)
    }

    pub fn list_asset_holders(&mut self, asset_id: &str, limit: Option<u16>, skip: Option<usize>) -> Result<ListAssetHoldersResult> {
        // Prepare query parameters
        let mut params_vec = Vec::new();
        let limit_str;
        let skip_str;
        let mut query_params = None;

        if let Some(val) = limit {
            limit_str = val.to_string();

            params_vec.push(("limit", limit_str.as_str()));
        }

        if let Some(val) = skip {
            skip_str = val.to_string();

            params_vec.push(("skip", skip_str.as_str()));
        }

        if params_vec.len() > 0 {
            query_params = Some(params_vec.as_slice());
        }

        let req = self.get_request(
            "assets/:asset_id/holders",
            Some(&[
                ("asset_id", asset_id),
            ]),
            query_params,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<ListAssetHoldersResponse>(res)?.data)
    }

    pub fn list_permission_events(&mut self) -> Result<ListPermissionEventsResult> {
        let req = self.get_request(
            "permission/events",
            None::<KVList>,
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<ListPermissionEventsResponse>(res)?.data)
    }

    pub fn retrieve_permission_rights(&mut self, event: PermissionEvent) -> Result<RetrievePermissionRightsResult> {
        let req = self.get_request(
            "permission/events/:event_name/rights",
            Some(&[
                ("event_name", event.to_string().as_str()),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<RetrievePermissionRightsResponse>(res)?.data)
    }

    pub fn set_permission_rights(&mut self, event: PermissionEvent, rights: AllPermissionRightsUpdate) -> Result<SetPermissionRightsResult> {
        let body_json = serde_json::to_string(&rights)?;
        let req = self.post_request(
            "permission/events/:event_name/rights",
            body_json,
            Some(&[
                ("event_name", event.to_string().as_str()),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<SetPermissionRightsResponse>(res)?.data)
    }

    pub fn check_effective_permission_right(&mut self, event: PermissionEvent, device: DeviceId) -> Result<CheckEffectivePermissionRightResult> {
        // Prepare query parameters
        let mut params_vec = Vec::new();
        let is_prod_unique_id;
        let mut query_params = None;

        if let Some(val) = device.is_prod_unique_id {
            is_prod_unique_id = val.to_string();

            params_vec.push(("isProdUniqueId", is_prod_unique_id.as_str()));
        }

        if params_vec.len() > 0 {
            query_params = Some(params_vec.as_slice());
        }

        let req = self.get_request(
            "permission/events/:event_name/rights/:device_id",
            Some(&[
                ("event_name", event.to_string().as_str()),
                ("device_id", device.id.as_str()),
            ]),
            query_params,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<CheckEffectivePermissionRightResponse>(res)?.data)
    }

    pub fn retrieve_device_identification_info(&mut self, device: DeviceId) -> Result<RetrieveDeviceIdentificationInfoResult> {
        // Prepare query parameters
        let mut params_vec = Vec::new();
        let is_prod_unique_id;
        let mut query_params = None;

        if let Some(val) = device.is_prod_unique_id {
            is_prod_unique_id = val.to_string();

            params_vec.push(("isProdUniqueId", is_prod_unique_id.as_str()));
        }

        if params_vec.len() > 0 {
            query_params = Some(params_vec.as_slice());
        }

        let req = self.get_request(
            "devices/:device_id",
            Some(&[
                ("device_id", device.id.as_str()),
            ]),
            query_params,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<RetrieveDeviceIdentificationInfoResponse>(res)?.data)
    }

    pub fn list_notification_events(&mut self) -> Result<ListNotificationEventsResult> {
        let req = self.get_request(
            "notification/events",
            None::<KVList>,
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req)?;

        Ok(Self::parse_response::<ListNotificationEventsResponse>(res)?.data)
    }

    // Definition of private methods

    fn send_request(&self, req: Request) -> Result<Response> {
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

    fn sign_and_send_request(&mut self, mut req: Request) -> Result<Response> {
        self.sign_request(&mut req)?;
        self.send_request(req)
    }

    fn get_request<I, K, V, I2, K2, V2>(&self, endpoint_url_path: &str, url_params: Option<I>, query_params: Option<I2>) -> Result<Request>
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

    fn post_request<I, K, V, I2, K2, V2>(&self, endpoint_url_path: &str, body: String, url_params: Option<I>, query_params: Option<I2>) -> Result<Request>
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

    pub(crate) fn get_ws_request<I, K, V>(&self, endpoint_url_path: &str, url_params: Option<I>) -> Result<Request>
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

    pub(crate) fn sign_request(&mut self, req: &mut Request) -> Result<()> {
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
        let value = String::from("CTN1-HMAC-SHA256 Credential=") + self.device_id.as_str() + "/"
            + &scope + ",Signature=" + &signature;

        req.headers_mut().insert(AUTHORIZATION, value.parse()?);

        Ok(())
    }

    // Definition of private associated ("static") functions

    fn parse_response<T: DeserializeOwned>(res: Response) -> Result<T> {
        let body = res.text()
            .map_err::<Error, _>(|e| Error::new_client_error(Some("Inconsistent Catenis API response"), Some(e)))?;

        serde_json::from_str(&body)
            .map_err::<Error, _>(|e| Error::new_client_error(Some("Inconsistent Catenis API response"), Some(e)))
    }

    fn new_http_client(use_compression: bool) -> reqwest::Result<HttpClient> {
        let mut client_builder = HttpClientBuilder::new();

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

    fn compress_body(body: String) -> Result<Vec<u8>> {
        let mut enc = DeflateEncoder::new(body.as_bytes(), Compression::default());
        let mut enc_body = Vec::new();
        enc.read_to_end(&mut enc_body)?;

        Ok(enc_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    use bitcoin_hashes::Hash;
    use bitcoin_hashes::hex;
    use bitcoin_hashes::hex::FromHex;
    use bitcoin_hashes::sha256;
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};

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

        let ctn_client = CatenisClient::new_with_options(
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

    #[test]
    fn it_call_read_message_api_method() {
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

        let result = ctn_client.read_message("orxtXbWS7c7pQCko9ReN", None);

        println!(">>>>>> Read Message result: {:?}", result);
    }
}