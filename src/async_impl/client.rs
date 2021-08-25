use std::{
    borrow::Borrow,
};
use bitcoin_hashes::{
    Hash, HashEngine, hex::ToHex, Hmac,
    HmacEngine,
    sha256,
};
use reqwest::{
    Client as HttpClient,
    ClientBuilder as HttpClientBuilder,
    Request, Response,
    header::{
        ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, CONTENT_ENCODING,
        HeaderMap, HeaderName, HeaderValue, HOST,
    },
    Url,
};
use async_compression::{
    Level,
    tokio_02::bufread::ZlibEncoder,
};
use tokio::io::AsyncReadExt;
use time::{
    Date,
};
use serde::de::DeserializeOwned;

use crate::*;
use crate::api::*;
use base_client::BaseCatenisClient;
use super::notification::WsNotifyChannel;

/// Represents an asynchronous Catenis API client.
#[derive(Debug, Clone)]
pub struct CatenisClient {
    device_credentials: Option<DeviceCredentials>,
    base_api_url: Url,
    is_secure: bool,
    use_compression: bool,
    compress_threshold: usize,
    sign_date: Option<Date>,
    signing_key: Option<[u8; 32]>,
    http_client: HttpClient,
}

impl BaseCatenisClient for CatenisClient {
    fn get_device_id_ref(&self) -> Result<&String> {
        if let Some(credentials) = &self.device_credentials {
            Ok(&credentials.device_id)
        } else {
            Err(Error::new_client_error(Some("Missing virtual device credentials"), None::<GenericError>))
        }
    }

    fn get_api_access_secret_ref(&self) -> Result<&String> {
        if let Some(credentials) = &self.device_credentials {
            Ok(&credentials.api_access_secret)
        } else {
            Err(Error::new_client_error(Some("Missing virtual device credentials"), None::<GenericError>))
        }
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

    /// Instantiate a new asynchronous Catenis API client object with default option settings.
    ///
    /// > **Note**: if no virtual device credentials are passed, the resulting client object should
    /// only be used to call **public** API methods.
    pub fn new(device_credentials: Option<DeviceCredentials>) -> Result<Self>
    {
        let base_url = Url::parse(DEFAULT_BASE_URL)?;
        let api_version = DEFAULT_API_VERSION;
        let is_secure = true;
        let use_compression = true;
        let compress_threshold: usize = 1024;

        Ok(CatenisClient {
            device_credentials,
            base_api_url: base_url.join(&Self::merge_url_params(API_BASE_URL_PATH, &[("version", api_version.to_string())]))?,
            is_secure,
            use_compression,
            compress_threshold,
            sign_date: None,
            signing_key: None,
            http_client: Self::new_http_client(use_compression)?,
        })
    }

    /// Instantiate a new asynchronous Catenis API client object with alternative option settings.
    ///
    /// > **Note**: if no virtual device credentials are passed, the resulting client object should
    /// only be used to call **public** API methods.
    pub fn new_with_options<'a, I>(device_credentials: Option<DeviceCredentials>, opts: I) -> Result<Self>
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
            device_credentials,
            base_api_url: base_url.join(&Self::merge_url_params(API_BASE_URL_PATH, &[("version", api_version.to_string())]))?,
            is_secure,
            use_compression,
            compress_threshold,
            sign_date: None,
            signing_key: None,
            http_client: Self::new_http_client(use_compression)?,
        })
    }

    /// Instantiate a new asynchronous WebSocket notification channel object for a given Catenis
    /// notification event.
    ///
    /// # Example
    /// ```no_run
    /// use catenis_api_client::{
    ///     async_impl,
    ///     ClientOptions, Environment, Result,
    ///     api::NotificationEvent,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let ctn_client = async_impl::CatenisClient::new_with_options(
    ///     Some((
    ///         "drc3XdxNtzoucpw9xiRp",
    ///         concat!(
    ///             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    ///             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    ///         ),
    ///     ).into()),
    ///     &[
    ///         ClientOptions::Environment(Environment::Sandbox),
    ///     ],
    /// )?;
    ///
    /// // Instantiate asynchronous WebSocket notification channel object for New Message Received
    /// //  notification event
    /// let notify_channel = ctn_client.new_ws_notify_channel(NotificationEvent::NewMsgReceived);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_ws_notify_channel(&self, notify_event: NotificationEvent) -> WsNotifyChannel {
        WsNotifyChannel::new(self, notify_event)
    }

    /// Call *Log Message* API method.
    ///
    /// # Examples
    ///
    /// Log a message in a single call:
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.log_message(
    ///     Message::Whole(String::from("My message")),
    ///     Some(LogMessageOptions {
    ///         encoding: Some(Encoding::UTF8),
    ///         encrypt: Some(true),
    ///         off_chain: Some(true),
    ///         storage: Some(Storage::Auto),
    ///         async_: None,
    ///     }),
    /// ).await?;
    ///
    /// println!("ID of logged message: {}", result.message_id.unwrap());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Log a message in chunks:
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let message = [
    ///     "First part of message",
    ///     "Second part of message",
    ///     "Third and last part of message"
    /// ];
    ///
    /// for idx in 0..message.len() {
    ///     let mut continuation_token = None;
    ///
    ///     let result = ctn_client.log_message(
    ///         Message::Chunk(ChunkedMessage {
    ///             data: Some(String::from(message[idx])),
    ///             is_final: Some(idx < message.len() - 1),
    ///             continuation_token: if let Some(token) = &continuation_token {
    ///                 Some(String::from(token))
    ///             } else {
    ///                 None
    ///             },
    ///         }),
    ///         Some(LogMessageOptions {
    ///             encoding: Some(Encoding::UTF8),
    ///             encrypt: Some(true),
    ///             off_chain: Some(true),
    ///             storage: Some(Storage::Auto),
    ///             async_: None,
    ///         }),
    ///     ).await?;
    ///
    ///     if let Some(token) = result.continuation_token {
    ///         continuation_token = Some(token);
    ///     } else {
    ///         println!("ID of logged message: {}", result.message_id.unwrap());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn log_message(&mut self, message: Message, options: Option<LogMessageOptions>) -> Result<LogMessageResult> {
        let body = LogMessageRequest {
            message,
            options
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request(
            "messages/log",
            body_json,
            None::<KVList>,
            None::<KVList>,
        ).await?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<LogMessageResponse>(res).await?.data)
    }

    /// Call *Send Message* API method.
    ///
    /// # Examples
    ///
    /// Send a message in a single call:
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.send_message(
    ///     Message::Whole(String::from("My message")),
    ///     DeviceId {
    ///         id: String::from("d8YpQ7jgPBJEkBrnvp58"),
    ///         is_prod_unique_id: None,
    ///     },
    ///     Some(SendMessageOptions {
    ///         encoding: Some(Encoding::UTF8),
    ///         encrypt: Some(true),
    ///         off_chain: Some(true),
    ///         storage: Some(Storage::Auto),
    ///         read_confirmation: Some(true),
    ///         async_: None,
    ///     }),
    /// ).await?;
    ///
    /// println!("ID of sent message: {}", result.message_id.unwrap());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Send a message in chunks:
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let message = [
    ///     "First part of message",
    ///     "Second part of message",
    ///     "Third and last part of message"
    /// ];
    ///
    /// for idx in 0..message.len() {
    ///     let mut continuation_token = None;
    ///
    ///     let result = ctn_client.send_message(
    ///         Message::Chunk(ChunkedMessage {
    ///             data: Some(String::from(message[idx])),
    ///             is_final: Some(idx < message.len() - 1),
    ///             continuation_token: if let Some(token) = &continuation_token {
    ///                 Some(String::from(token))
    ///             } else {
    ///                 None
    ///             },
    ///         }),
    ///         DeviceId {
    ///             id: String::from("d8YpQ7jgPBJEkBrnvp58"),
    ///             is_prod_unique_id: None,
    ///         },
    ///         Some(SendMessageOptions {
    ///             encoding: Some(Encoding::UTF8),
    ///             encrypt: Some(true),
    ///             off_chain: Some(true),
    ///             storage: Some(Storage::Auto),
    ///             read_confirmation: Some(true),
    ///             async_: None,
    ///         }),
    ///     ).await?;
    ///
    ///     if let Some(token) = result.continuation_token {
    ///         continuation_token = Some(token);
    ///     } else {
    ///         println!("ID of sent message: {}", result.message_id.unwrap());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_message(&mut self, message: Message, target_device: DeviceId, options: Option<SendMessageOptions>) -> Result<SendMessageResult> {
        let body = SendMessageRequest {
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
        ).await?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<SendMessageResponse>(res).await?.data)
    }

    /// Call *Read Message* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.read_message(
    ///     "o3muoTnnD6cXYyarYY38",
    ///     Some(ReadMessageOptions {
    ///         encoding: Some(Encoding::UTF8),
    ///         continuation_token: None,
    ///         data_chunk_size: None,
    ///         async_: None,
    ///     }),
    /// ).await?;
    ///
    /// println!("Read message: {}", result.msg_data.unwrap());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_message(&mut self, message_id: &str, options: Option<ReadMessageOptions>) -> Result<ReadMessageResult> {
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

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<ReadMessageResponse>(res).await?.data)
    }

    /// Call *Retrieve Message Container* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.retrieve_message_container(
    ///     "o3muoTnnD6cXYyarYY38",
    /// ).await?;
    ///
    /// if let Some(off_chain) = result.off_chain {
    ///     println!("IPFS CID of Catenis off-chain message envelope: {}", off_chain.cid);
    /// }
    ///
    /// if let Some(blockchain) = result.blockchain {
    ///     println!("ID of blockchain transaction containing the message: {}", blockchain.txid);
    /// }
    ///
    /// if let Some(external_storage) = result.external_storage {
    ///     println!("IPFS reference to message: {}", external_storage.ipfs);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve_message_container(&mut self, message_id: &str) -> Result<RetrieveMessageContainerResult> {
        let req = self.get_request(
            "messages/:message_id/container",
            Some(&[
                ("message_id", message_id),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<RetrieveMessageContainerResponse>(res).await?.data)
    }

    /// Call *Retrieve Message Origin* API method.
    ///
    /// > **Note**: this is a public API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     None,
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.retrieve_message_origin(
    ///     "o3muoTnnD6cXYyarYY38",
    ///     Some("Any text to be signed"),
    /// ).await?;
    ///
    /// if let Some(tx) = result.tx {
    ///     println!("Catenis message transaction info: {:?}", tx);
    /// }
    ///
    /// if let Some(off_chain_msg_env) = result.off_chain_msg_envelope {
    ///     println!("Off-chain message envelope info: {:?}", off_chain_msg_env);
    /// }
    ///
    /// if let Some(proof) = result.proof {
    ///     println!("Origin proof info: {:?}", proof);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve_message_origin(&self, message_id: &str, msg_to_sign: Option<&str>) -> Result<RetrieveMessageOriginResult> {
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

        let res = self.send_request(req).await?;

        Ok(Self::parse_response::<RetrieveMessageOriginResponse>(res).await?.data)
    }

    /// Call *Retrieve Message Progress* API method.
    ///
    /// **Note**: this method should be passed an *ephemeral message ID*: either a *provisional
    /// message ID* or a *cached message ID*.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.retrieve_message_progress(
    ///     "pTZCgjKYEyHfu4rNPWct",
    /// ).await?;
    ///
    /// println!("Number of bytes processed so far: {}", result.progress.bytes_processed);
    ///
    /// if result.progress.done {
    ///     if let Some(true) = result.progress.success {
    ///         // Get result
    ///         println!("Asynchronous processing result: {:?}", result.result.unwrap());
    ///     } else {
    ///         // Process error
    ///         let error = result.progress.error.unwrap();
    ///
    ///         println!("Asynchronous processing error: [{}] - {}", error.code, error.message);
    ///     }
    /// } else {
    ///     // Asynchronous processing not done yet. Continue pooling
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve_message_progress(&mut self, message_id: &str) -> Result<RetrieveMessageProgressResult> {
        let req = self.get_request(
            "messages/:message_id/progress",
            Some(&[
                ("message_id", message_id),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<RetrieveMessageProgressResponse>(res).await?.data)
    }

    /// Call *List Messages* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.list_messages(
    ///     Some(ListMessagesOptions {
    ///         action: Some(MessageActionOption::Send),
    ///         direction: Some(MessageDirectionOption::Inbound),
    ///         from_devices: None,
    ///         to_devices: None,
    ///         read_state: Some(MessageReadStateOption::Unread),
    ///         start_date: Some("2020-12-22T00:00:00Z".into()),
    ///         end_date: None,
    ///         limit: None,
    ///         skip: None,
    ///     }),
    /// ).await?;
    ///
    /// if result.msg_count > 0 {
    ///     println!("Returned messages: {:?}", result.messages);
    ///
    ///     if result.has_more {
    ///         println!("Not all messages have been returned");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_messages(&mut self, options: Option<ListMessagesOptions>) -> Result<ListMessagesResult> {
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
                    from_device_ids = ids.join(",");

                    params_vec.push(("fromDeviceIds", from_device_ids.as_str()));
                }

                if prod_unique_ids.len() > 0 {
                    from_device_prod_unique_ids = prod_unique_ids.join(",");

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
                    to_device_ids = ids.join(",");

                    params_vec.push(("toDeviceIds", to_device_ids.as_str()));
                }

                if prod_unique_ids.len() > 0 {
                    to_device_prod_unique_ids = prod_unique_ids.join(",");

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

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<ListMessagesResponse>(res).await?.data)
    }

    /// Call *Issue Asset* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.issue_asset(
    ///     NewAssetInfo {
    ///         name: String::from("XYZ001"),
    ///         description: Some(String::from("My first test asset")),
    ///         can_reissue: true,
    ///         decimal_places: 2,
    ///     },
    ///     1_500.0,
    ///     None,
    /// ).await?;
    ///
    /// println!("ID of newly issued asset: {}", result.asset_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn issue_asset(&mut self, asset_info: NewAssetInfo, amount: f64, holding_device: Option<DeviceId>) -> Result<IssueAssetResult> {
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
        ).await?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<IssueAssetResponse>(res).await?.data)
    }

    /// Call *Reissue Asset* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.reissue_asset(
    ///     "aBy2ovnucyWaSB6Tro9x",
    ///     650.25,
    ///     Some(DeviceId {
    ///         id: String::from("d8YpQ7jgPBJEkBrnvp58"),
    ///         is_prod_unique_id: None,
    ///     }),
    /// ).await?;
    ///
    /// println!("Total existent asset balance (after issuance): {}", result.total_existent_balance);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reissue_asset(&mut self, asset_id: &str, amount: f64, holding_device: Option<DeviceId>) -> Result<ReissueAssetResult> {
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
        ).await?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<ReissueAssetResponse>(res).await?.data)
    }

    /// Call *Transfer Asset* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.transfer_asset(
    ///     "aBy2ovnucyWaSB6Tro9x",
    ///     50.75,
    ///     DeviceId {
    ///         id: String::from("d8YpQ7jgPBJEkBrnvp58"),
    ///         is_prod_unique_id: None,
    ///     },
    /// ).await?;
    ///
    /// println!("Remaining asset balance: {}", result.remaining_balance);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn transfer_asset(&mut self, asset_id: &str, amount: f64, receiving_device: DeviceId) -> Result<TransferAssetResult> {
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
        ).await?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<TransferAssetResponse>(res).await?.data)
    }

    /// Call *Retrieve Asset Info* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.retrieve_asset_info(
    ///     "aBy2ovnucyWaSB6Tro9x",
    /// ).await?;
    ///
    /// println!("Asset info: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve_asset_info(&mut self, asset_id: &str) -> Result<RetrieveAssetInfoResult> {
        let req = self.get_request(
            "assets/:asset_id",
            Some(&[
                ("asset_id", asset_id),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<RetrieveAssetInfoResponse>(res).await?.data)
    }

    /// Call *Get Asset Balance* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.get_asset_balance(
    ///     "aBy2ovnucyWaSB6Tro9x",
    /// ).await?;
    ///
    /// println!("Current asset balance: {}", result.total);
    /// println!("Amount not yet confirmed: {}", result.unconfirmed);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_asset_balance(&mut self, asset_id: &str) -> Result<GetAssetBalanceResult> {
        let req = self.get_request(
            "assets/:asset_id/balance",
            Some(&[
                ("asset_id", asset_id),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<GetAssetBalanceResponse>(res).await?.data)
    }

    /// Call *List Owned Assets* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.list_owned_assets(
    ///     Some(200),
    ///     Some(0),
    /// ).await?;
    ///
    /// for owned_asset in result.owned_assets {
    ///     println!("Asset ID: {}", owned_asset.asset_id);
    ///     println!(" - current asset balance: {}", owned_asset.balance.total);
    ///     println!(" - amount not yet confirmed: {}\n", owned_asset.balance.unconfirmed);
    /// }
    ///
    /// if result.has_more {
    ///     println!("Not all owned assets have been returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_owned_assets(&mut self, limit: Option<u16>, skip: Option<usize>) -> Result<ListOwnedAssetsResult> {
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

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<ListOwnedAssetsResponse>(res).await?.data)
    }

    /// Call *List Issued Assets* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.list_issued_assets(
    ///     Some(200),
    ///     Some(0),
    /// ).await?;
    ///
    /// for issued_asset in result.issued_assets {
    ///     println!("Asset ID: {}", issued_asset.asset_id);
    ///     println!(" - total existent balance: {}\n", issued_asset.total_existent_balance);
    /// }
    ///
    /// if result.has_more {
    ///     println!("Not all issued assets have been returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_issued_assets(&mut self, limit: Option<u16>, skip: Option<usize>) -> Result<ListIssuedAssetsResult> {
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

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<ListIssuedAssetsResponse>(res).await?.data)
    }

    /// Call *Retrieve Asset Issuance History* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.retrieve_asset_issuance_history(
    ///     "aBy2ovnucyWaSB6Tro9x",
    ///     Some("2020-12-01T00:00:00Z".into()),
    ///     None,
    ///     Some(200),
    ///     Some(0),
    /// ).await?;
    ///
    /// for issuance_events in result.issuance_events {
    ///     println!("Issuance date: {}", issuance_events.date);
    ///     println!(" - issued amount: {}", issuance_events.amount);
    ///     println!(
    ///         " - device to which issued amount had been assigned: {:?}\n",
    ///         issuance_events.holding_device
    ///     );
    /// }
    ///
    /// if result.has_more {
    ///     println!("Not all asset issuance events have been returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve_asset_issuance_history(
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

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<RetrieveAssetIssuanceHistoryResponse>(res).await?.data)
    }

    /// Call *List Asset Holders* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.list_asset_holders(
    ///     "aBy2ovnucyWaSB6Tro9x",
    ///     Some(200),
    ///     Some(0),
    /// ).await?;
    ///
    /// for asset_holder in result.asset_holders {
    ///     if let Some(holder) = asset_holder.holder {
    ///         println!("Asset holder ID: {}", holder.device_id);
    ///         println!(" - detailed holder info: {:?}", holder);
    ///         println!(
    ///             " - amount of asset currently held by device: {}",
    ///             asset_holder.balance.total
    ///         );
    ///         println!(" - amount not yet confirmed: {}\n", asset_holder.balance.unconfirmed);
    ///     } else {
    ///         println!("Migrated asset:");
    ///         println!(" - total migrated amount: {}", asset_holder.balance.total);
    ///         println!(" - amount not yet confirmed: {}", asset_holder.balance.unconfirmed);
    ///     }
    /// }
    ///
    /// if result.has_more {
    ///     println!("Not all asset holders have been returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_asset_holders(&mut self, asset_id: &str, limit: Option<u16>, skip: Option<usize>) -> Result<ListAssetHoldersResult> {
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

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<ListAssetHoldersResponse>(res).await?.data)
    }

    /// Call *Export Asset* API method.
    ///
    /// # Examples
    ///
    /// Estimate export cost (in foregin blockchain's native coin):
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     CatenisClient, ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.export_asset(
    ///     "aH2AkrrL55GcThhPNa3J",
    ///     ForeignBlockchain::Ethereum,
    ///     NewForeignTokenInfo {
    ///         name: String::from("Catenis test token #10"),
    ///         symbol: String::from("CTK10"),
    ///     },
    ///     Some(ExportAssetOptions {
    ///         consumption_profile: None,
    ///         estimate_only: Some(true),
    ///     }),
    /// ).await?;
    ///
    /// println!(
    ///     "Estimated foreign blockchain transaction execution price: {}",
    ///     result.estimated_price.unwrap()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Export asset:
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     CatenisClient, ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.export_asset(
    ///     "aH2AkrrL55GcThhPNa3J",
    ///     ForeignBlockchain::Ethereum,
    ///     NewForeignTokenInfo {
    ///         name: String::from("Catenis test token #10"),
    ///         symbol: String::from("CTK10"),
    ///     },
    ///     None,
    /// ).await?;
    ///
    /// println!("Pending asset export: {:?}", result);
    ///
    /// // Start polling for asset export outcome
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_asset(
        &mut self,
        asset_id: &str,
        foreign_blockchain: ForeignBlockchain,
        token: NewForeignTokenInfo,
        options: Option<ExportAssetOptions>,
    ) -> Result<ExportAssetResult> {
        let body = ExportAssetRequest {
            token,
            options
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request(
            "assets/:asset_id/export/:foreign_blockchain",
            body_json,
            Some(&[
                ("asset_id", asset_id),
                ("foreign_blockchain", foreign_blockchain.to_string().as_str()),
            ]),
            None::<KVList>,
        ).await?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<ExportAssetResponse>(res).await?.data)
    }

    /// Call *Migrate Asset* API method.
    ///
    /// # Examples
    ///
    /// Estimate migration cost (in foreign blockchain's native coin):
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     CatenisClient, ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.migrate_asset(
    ///     "aH2AkrrL55GcThhPNa3J",
    ///     ForeignBlockchain::Ethereum,
    ///     AssetMigration::Info(AssetMigrationInfo {
    ///         direction: AssetMigrationDirection::Outward,
    ///         amount: 50.0,
    ///         dest_address: Some(String::from("0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943")),
    ///     }),
    ///     Some(MigrateAssetOptions {
    ///         consumption_profile: None,
    ///         estimate_only: Some(true),
    ///     }),
    /// ).await?;
    ///
    /// println!(
    ///     "Estimated foreign blockchain transaction execution price: {}",
    ///     result.estimated_price.unwrap()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Migrate asset:
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     CatenisClient, ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.migrate_asset(
    ///     "aH2AkrrL55GcThhPNa3J",
    ///     ForeignBlockchain::Ethereum,
    ///     AssetMigration::Info(AssetMigrationInfo {
    ///         direction: AssetMigrationDirection::Outward,
    ///         amount: 50.0,
    ///         dest_address: Some(String::from("0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943")),
    ///     }),
    ///     None,
    /// ).await?;
    ///
    /// println!("Pending asset migration: {:?}", result);
    ///
    /// // Start polling for asset migration outcome
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Reprocess a (failed) migration:
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     CatenisClient, ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.migrate_asset(
    ///     "aH2AkrrL55GcThhPNa3J",
    ///     ForeignBlockchain::Ethereum,
    ///     AssetMigration::ID(String::from("gq8x3efLpEXTkGQchHTb")),
    ///     None,
    /// ).await?;
    ///
    /// println!("Pending asset migration: {:?}", result);
    ///
    /// // Start polling for asset migration outcome
    /// # Ok(())
    /// # }
    /// ```
    pub async fn migrate_asset(
        &mut self,
        asset_id: &str,
        foreign_blockchain: ForeignBlockchain,
        migration: AssetMigration,
        options: Option<MigrateAssetOptions>,
    ) -> Result<MigrateAssetResult> {
        let body = MigrateAssetRequest {
            migration,
            options
        };
        let body_json = serde_json::to_string(&body)?;
        let req = self.post_request(
            "assets/:asset_id/migrate/:foreign_blockchain",
            body_json,
            Some(&[
                ("asset_id", asset_id),
                ("foreign_blockchain", foreign_blockchain.to_string().as_str()),
            ]),
            None::<KVList>,
        ).await?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<MigrateAssetResponse>(res).await?.data)
    }

    /// Call *Asset Export Outcome* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     CatenisClient, ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.asset_export_outcome(
    ///     "aH2AkrrL55GcThhPNa3J",
    ///     ForeignBlockchain::Ethereum,
    /// ).await?;
    ///
    /// match result.status {
    ///     AssetExportStatus::Success => {
    ///         // Asset successfully exported
    ///         println!("Foreign token ID (address): {}", result.token.id.unwrap());
    ///     },
    ///     AssetExportStatus::Pending => {
    ///         // Final asset export state not yet reached
    ///     },
    ///     AssetExportStatus::Error => {
    ///         // Asset export has failed. Process error
    ///         println!(
    ///             "Error executing foreign blockchain transaction: {}",
    ///             result.foreign_transaction.error.unwrap()
    ///         );
    ///     },
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn asset_export_outcome(
        &mut self,
        asset_id: &str,
        foreign_blockchain: ForeignBlockchain,
    ) -> Result<AssetExportOutcomeResult> {
        let req = self.get_request(
            "assets/:asset_id/export/:foreign_blockchain",
            Some(&[
                ("asset_id", asset_id),
                ("foreign_blockchain", foreign_blockchain.to_string().as_str()),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<AssetExportOutcomeResponse>(res).await?.data)
    }

    /// Call *Asset Migration Outcome* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     CatenisClient, ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.asset_migration_outcome(
    ///     "gq8x3efLpEXTkGQchHTb",
    /// ).await?;
    ///
    /// match result.status {
    ///     AssetMigrationStatus::Success => {
    ///         // Asset amount successfully migrated
    ///         println!("Asset amount successfully migrated");
    ///     },
    ///     AssetMigrationStatus::Pending => {
    ///         // Final asset migration state not yet reached
    ///     },
    ///     _ => {
    ///         // Asset migration has failed. Process error
    ///         if let Some(error) = result.catenis_service.error {
    ///             println!("Error executing Catenis service: {}", error);
    ///         }
    ///
    ///         if let Some(error) = result.foreign_transaction.error {
    ///             println!("Error executing foreign blockchain transaction: {}", error);
    ///         }
    ///     },
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn asset_migration_outcome(
        &mut self,
        migration_id: &str,
    ) -> Result<AssetMigrationOutcomeResult> {
        let req = self.get_request(
            "assets/migrations/:migration_id",
            Some(&[
                ("migration_id", migration_id),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<AssetMigrationOutcomeResponse>(res).await?.data)
    }

    /// Call *List Exported Assets* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     CatenisClient, ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.list_exported_assets(
    ///     Some(ListExportedAssetsOptions {
    ///         asset_id: None,
    ///         foreign_blockchain: Some(ForeignBlockchain::Ethereum),
    ///         token_symbol: None,
    ///         status: Some(vec![AssetExportStatus::Success]),
    ///         negate_status: None,
    ///         start_date: Some("2021-08-01T00:00:00Z".into()),
    ///         end_date: None,
    ///         limit: None,
    ///         skip: None,
    ///     }),
    /// ).await?;
    ///
    /// if result.exported_assets.len() > 0 {
    ///     println!("Returned asset exports: {:?}", result.exported_assets);
    ///
    ///     if result.has_more {
    ///         println!("Not all asset exports have been returned");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_exported_assets(&mut self, options: Option<ListExportedAssetsOptions>) -> Result<ListExportedAssetsResult> {
        // Prepare query parameters
        let mut params_vec = Vec::new();
        let asset_id;
        let foreign_blockchain;
        let token_symbol;
        let status;
        let negate_status;
        let start_date;
        let end_date;
        let limit;
        let skip;
        let mut query_params = None;

        if let Some(opt) = options {
            if let Some(val) = opt.asset_id {
                asset_id = val;

                params_vec.push(("assetId", asset_id.as_str()));
            }

            if let Some(val) = opt.foreign_blockchain {
                foreign_blockchain = val.to_string();

                params_vec.push(("foreignBlockchain", foreign_blockchain.as_str()));
            }

            if let Some(val) = opt.token_symbol {
                token_symbol = val;

                params_vec.push(("tokenSymbol", token_symbol.as_str()));
            }

            if let Some(val) = opt.status {
                let mut list_str = Vec::new();

                for item in val {
                    list_str.push(item.to_string());
                }

                if list_str.len() > 0 {
                    status = list_str.join(",");

                    params_vec.push(("status", status.as_str()));
                }
            }

            if let Some(val) = opt.negate_status {
                negate_status = if val {"true"} else {"false"};

                params_vec.push(("negateStatus", negate_status));
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
            "assets/exported",
            None::<KVList>,
            query_params,
        )?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<ListExportedAssetsResponse>(res).await?.data)
    }

    /// Call *List Asset Migrations* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     CatenisClient, ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.list_asset_migrations(
    ///     Some(ListAssetMigrationsOptions {
    ///         asset_id: None,
    ///         foreign_blockchain: Some(ForeignBlockchain::Ethereum),
    ///         direction: Some(AssetMigrationDirection::Outward),
    ///         status: Some(vec![
    ///             AssetMigrationStatus::Interrupted,
    ///             AssetMigrationStatus::Error,
    ///         ]),
    ///         negate_status: None,
    ///         start_date: Some("2021-08-01T00:00:00Z".into()),
    ///         end_date: None,
    ///         limit: None,
    ///         skip: None,
    ///     }),
    /// ).await?;
    ///
    /// if result.asset_migrations.len() > 0 {
    ///     println!("Returned asset migrations: {:?}", result.asset_migrations);
    ///
    ///     if result.has_more {
    ///         println!("Not all asset migrations have been returned");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_asset_migrations(&mut self, options: Option<ListAssetMigrationsOptions>) -> Result<ListAssetMigrationsResult> {
        // Prepare query parameters
        let mut params_vec = Vec::new();
        let asset_id;
        let foreign_blockchain;
        let direction;
        let status;
        let negate_status;
        let start_date;
        let end_date;
        let limit;
        let skip;
        let mut query_params = None;

        if let Some(opt) = options {
            if let Some(val) = opt.asset_id {
                asset_id = val;

                params_vec.push(("assetId", asset_id.as_str()));
            }

            if let Some(val) = opt.foreign_blockchain {
                foreign_blockchain = val.to_string();

                params_vec.push(("foreignBlockchain", foreign_blockchain.as_str()));
            }

            if let Some(val) = opt.direction {
                direction = val.to_string();

                params_vec.push(("direction", direction.as_str()));
            }

            if let Some(val) = opt.status {
                let mut list_str = Vec::new();

                for item in val {
                    list_str.push(item.to_string());
                }

                if list_str.len() > 0 {
                    status = list_str.join(",");

                    params_vec.push(("status", status.as_str()));
                }
            }

            if let Some(val) = opt.negate_status {
                negate_status = if val {"true"} else {"false"};

                params_vec.push(("negateStatus", negate_status));
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
            "assets/migrations",
            None::<KVList>,
            query_params,
        )?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<ListAssetMigrationsResponse>(res).await?.data)
    }

    /// Call *List Permission Events* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.list_permission_events().await?;
    ///
    /// for (event, description) in result {
    ///     println!("Permission event: {} - {}", event.to_string(), description);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_permission_events(&mut self) -> Result<ListPermissionEventsResult> {
        let req = self.get_request(
            "permission/events",
            None::<KVList>,
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<ListPermissionEventsResponse>(res).await?.data)
    }

    /// Call *Retrieve Permission Rights* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.retrieve_permission_rights(
    ///     PermissionEvent::ReceiveMsg,
    /// ).await?;
    ///
    /// println!("Default (system) permission right: {:?}", result.system);
    ///
    /// if let Some(rights_setting) = result.catenis_node {
    ///     if let Some(catenis_node_idxs) = rights_setting.allow {
    ///         println!(
    ///             "Index of Catenis nodes to which permission is granted: {:?}",
    ///             catenis_node_idxs
    ///         );
    ///     }
    ///
    ///     if let Some(catenis_node_idxs) = rights_setting.deny {
    ///         println!(
    ///             "Index of Catenis nodes to which permission is not granted: {:?}",
    ///             catenis_node_idxs
    ///         );
    ///     }
    /// }
    ///
    /// if let Some(rights_setting) = result.client {
    ///     if let Some(client_ids) = rights_setting.allow {
    ///         println!("ID of clients to which permission is granted: {:?}", client_ids);
    ///     }
    ///
    ///     if let Some(client_ids) = rights_setting.deny {
    ///         println!("ID of clients to which permission is not granted: {:?}", client_ids);
    ///     }
    /// }
    ///
    /// if let Some(rights_setting) = result.device {
    ///     if let Some(devices) = rights_setting.allow {
    ///         println!("Devices to which permission is granted: {:?}", devices);
    ///     }
    ///
    ///     if let Some(devices) = rights_setting.deny {
    ///         println!("Devices to which permission is not granted: {:?}", devices);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve_permission_rights(&mut self, event: PermissionEvent) -> Result<RetrievePermissionRightsResult> {
        let req = self.get_request(
            "permission/events/:event_name/rights",
            Some(&[
                ("event_name", event.to_string().as_str()),
            ]),
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<RetrievePermissionRightsResponse>(res).await?.data)
    }

    /// Call *Set Permission Rights* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.set_permission_rights(
    ///     PermissionEvent::ReceiveMsg,
    ///     AllPermissionRightsUpdate {
    ///         system: Some(PermissionRight::Deny),
    ///         catenis_node: Some(PermissionRightsUpdate {
    ///             allow: Some(vec![
    ///                 String::from("self"),
    ///             ]),
    ///             deny: None,
    ///             none: None,
    ///         }),
    ///         client: Some(PermissionRightsUpdate {
    ///             allow: Some(vec![
    ///                 String::from("self"),
    ///                 String::from("c3gBoX45xk3yAmenyDRD"),
    ///             ]),
    ///             deny: None,
    ///             none: None,
    ///         }),
    ///         device: Some(DevicePermissionRightsUpdate {
    ///             allow: None,
    ///             deny: Some(vec![
    ///                 DeviceId {
    ///                     id: String::from("self"),
    ///                     is_prod_unique_id: None,
    ///                 },
    ///                 DeviceId {
    ///                     id: String::from("ABCD001"),
    ///                     is_prod_unique_id: Some(true),
    ///                 },
    ///             ]),
    ///             none: None,
    ///         }),
    ///     },
    /// ).await?;
    ///
    /// println!("Permission rights successfully set");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_permission_rights(&mut self, event: PermissionEvent, rights: AllPermissionRightsUpdate) -> Result<SetPermissionRightsResult> {
        let body_json = serde_json::to_string(&rights)?;
        let req = self.post_request(
            "permission/events/:event_name/rights",
            body_json,
            Some(&[
                ("event_name", event.to_string().as_str()),
            ]),
            None::<KVList>,
        ).await?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<SetPermissionRightsResponse>(res).await?.data)
    }

    /// Call *Check Effective Permission Right* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.check_effective_permission_right(
    ///     PermissionEvent::ReceiveMsg,
    ///     DeviceId {
    ///         id: String::from("d8YpQ7jgPBJEkBrnvp58"),
    ///         is_prod_unique_id: None,
    ///     },
    /// ).await?;
    ///
    /// let (device_id, right) = result.iter().next().unwrap();
    ///
    /// println!("Effective right for device {}: {:?}", device_id, right);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check_effective_permission_right(&mut self, event: PermissionEvent, device: DeviceId) -> Result<CheckEffectivePermissionRightResult> {
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

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<CheckEffectivePermissionRightResponse>(res).await?.data)
    }

    /// Call *Retrieve Device Identification Info* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.retrieve_device_identification_info(
    ///     DeviceId {
    ///         id: String::from("self"),
    ///         is_prod_unique_id: None,
    ///     },
    /// ).await?;
    ///
    /// println!("Device's Catenis node ID info: {:?}", result.catenis_node);
    /// println!("Device's client ID info: {:?}", result.client);
    /// println!("Device's own ID info: {:?}", result.device);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve_device_identification_info(&mut self, device: DeviceId) -> Result<RetrieveDeviceIdentificationInfoResult> {
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

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<RetrieveDeviceIdentificationInfoResponse>(res).await?.data)
    }

    /// Call *List Notification Events* API method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use catenis_api_client::{
    /// #     async_impl,
    /// #     ClientOptions, Environment, Result,
    ///     api::*,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let mut ctn_client = async_impl::CatenisClient::new_with_options(
    /// #     Some((
    /// #         "drc3XdxNtzoucpw9xiRp",
    /// #         concat!(
    /// #             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
    /// #             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
    /// #         ),
    /// #     ).into()),
    /// #     &[
    /// #         ClientOptions::Environment(Environment::Sandbox),
    /// #     ],
    /// # )?;
    /// #
    /// let result = ctn_client.list_notification_events().await?;
    ///
    /// for (event, description) in result {
    ///     println!("Notification event: {} - {}", event.to_string(), description);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_notification_events(&mut self) -> Result<ListNotificationEventsResult> {
        let req = self.get_request(
            "notification/events",
            None::<KVList>,
            None::<KVList>,
        )?;

        let res = self.sign_and_send_request(req).await?;

        Ok(Self::parse_response::<ListNotificationEventsResponse>(res).await?.data)
    }

    // Definition of private methods

    async fn send_request(&self, req: Request) -> Result<Response> {
        let res = self.http_client
            .execute(req)
            .await
            .map_err::<Error, _>(Into::into)?;

        if res.status().is_success() {
            Ok(res)
        } else {
            Err(Error::from_http_response_async(res).await)
        }
    }

    async fn sign_and_send_request(&mut self, mut req: Request) -> Result<Response> {
        self.sign_request(&mut req)?;
        self.send_request(req).await
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

        let mut req_builder = self.http_client
            .get(self.base_api_url.join(&endpoint_url_path)?);

        if let Some(params) = query_params {
            req_builder = req_builder.query(&Self::assemble_query_params(params));
        }

        req_builder.build()
            .map_err(Into::into)
    }

    async fn post_request<I, K, V, I2, K2, V2>(&self, endpoint_url_path: &str, body: String, url_params: Option<I>, query_params: Option<I2>) -> Result<Request>
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

        let mut req_builder = self.http_client
            .post(self.base_api_url.join(&endpoint_url_path)?);

        if body.len() > 0 {
            // Prepare to add body to request
            req_builder = req_builder.header(CONTENT_TYPE, HeaderValue::from_static("application/json; charset=utf-8"));

            if self.use_compression && body.len() >= self.compress_threshold {
                // Add compressed body
                req_builder = req_builder.body(Self::compress_body(body).await?)
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
            now = now!();
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
        self.check_update_sign_date_and_key(&now)?;

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
        let value = String::from("CTN1-HMAC-SHA256 Credential=") + self.get_device_id_ref()?.as_str() + "/"
            + &scope + ",Signature=" + &signature;

        req.headers_mut().insert(AUTHORIZATION, value.parse()?);

        Ok(())
    }

    // Definition of private associated ("static") functions

    async fn parse_response<T: DeserializeOwned>(res: Response) -> Result<T> {
        let body = res.text().await
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
            client_builder = client_builder.gzip(true);
        } else {
            // Make sure that response is not compressed
            client_builder = client_builder.gzip(false);
        }

        client_builder
            .default_headers(headers)
            .build()
    }

    async fn compress_body(body: String) -> Result<Vec<u8>> {
        let mut enc = ZlibEncoder::with_quality(body.as_bytes(), Level::Default);
        let mut enc_body = Vec::new();
        enc.read_to_end(&mut enc_body).await?;

        Ok(enc_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_helper::{
            http_server::{
                HttpServer, HttpRequest, PartialHttpRequest, HttpServerMode,
                HttpBody, ContentEncoding, HttpHeader,
            },
        },
    };
    use super::CatenisClient;

    #[tokio::test]
    async fn it_issue_api_error() {
        // Simulate error 'Read Message' API method response

        // Start HTTP server in error simulation node
        let http_server = HttpServer::new(
            HttpServerMode::Error(
                400,
                Some(HttpBody::from_json(r#"{"status":"error","message":"Invalid message ID"}"#).unwrap()),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                 "d8YpQ7jgPBJEkBrnvp58",
                 "267a687115b9752f2eec5be849b570b29133528f928868d811bad5e48e97a1d62d432bab44803586b2ac35002ec6f0eeaa98bec79b64f2f69b9cb0935b4df2c4",
             ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.read_message(
            "xxxxxxxx",
            None,
        ).await;

        assert!(result.is_err(), "Returned success from sending request");
        assert_eq!(result.err().unwrap().to_string(), "Catenis API error: [400] - Invalid message ID");
    }

    #[tokio::test]
    async fn it_log_message_whole() {
        // Simulate successful 'Log Message' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "messageId": "oX2mJHwFWp752beHbNDK"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/messages/log", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Test message #1","options":{"encoding":"utf8","encrypt":true,"offChain":true}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.log_message(
            Message::Whole(String::from("Test message #1")),
            Some(LogMessageOptions {
                encoding: Some(Encoding::UTF8),
                encrypt: Some(true),
                off_chain: Some(true),
                storage: None,
                async_: None,
            }),
        ).await.unwrap();

        assert_eq!(result, LogMessageResult {
            continuation_token: None,
            message_id: Some(String::from("oX2mJHwFWp752beHbNDK")),
            provisional_message_id: None,
        });
    }

    #[tokio::test]
    async fn it_log_message_whole_async() {
        // Simulate successful 'Log Message' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "provisionalMessageId": "pGGqH67dWoZ74qx3RTK8"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/messages/log", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Test message #1","options":{"encoding":"utf8","encrypt":true,"offChain":true,"async":true}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.log_message(
            Message::Whole(String::from("Test message #1")),
            Some(LogMessageOptions {
                encoding: Some(Encoding::UTF8),
                encrypt: Some(true),
                off_chain: Some(true),
                storage: None,
                async_: Some(true),
            }),
        ).await.unwrap();

        assert_eq!(result, LogMessageResult {
            continuation_token: None,
            message_id: None,
            provisional_message_id: Some(String::from("pGGqH67dWoZ74qx3RTK8")),
        });
    }

    #[tokio::test]
    async fn it_log_message_chunk() {
        // Simulate successful 'Log Message' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "continuationToken": "kCgPK4rPFyMKei4YsibY"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/messages/log", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":{"data":"Test message #1 (part 1)","isFinal":false},"options":{"encoding":"utf8","encrypt":true,"offChain":true}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.log_message(
            Message::Chunk(ChunkedMessage {
                data: Some(String::from("Test message #1 (part 1)")),
                is_final: Some(false),
                continuation_token: None,
            }),
            Some(LogMessageOptions {
                encoding: Some(Encoding::UTF8),
                encrypt: Some(true),
                off_chain: Some(true),
                storage: None,
                async_: None,
            }),
        ).await.unwrap();

        assert_eq!(result, LogMessageResult {
            continuation_token: Some(String::from("kCgPK4rPFyMKei4YsibY")),
            message_id: None,
            provisional_message_id: None,
        });
    }

    #[tokio::test]
    async fn it_send_message_whole() {
        // Simulate successful 'Send Message' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "messageId": "o3muoTnnD6cXYyarYY38"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/messages/send", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Test message #1","targetDevice":{"id":"d8YpQ7jgPBJEkBrnvp58"},"options":{"encoding":"utf8","encrypt":true,"offChain":true,"readConfirmation":false}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.send_message(
            Message::Whole(String::from("Test message #1")),
            DeviceId {
                id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                is_prod_unique_id: None,
            },
            Some(SendMessageOptions {
                encoding: Some(Encoding::UTF8),
                encrypt: Some(true),
                off_chain: Some(true),
                storage: None,
                read_confirmation: Some(false),
                async_: None,
            }),
        ).await.unwrap();

        assert_eq!(result, SendMessageResult {
            continuation_token: None,
            message_id: Some(String::from("o3muoTnnD6cXYyarYY38")),
            provisional_message_id: None,
        });
    }

    #[tokio::test]
    async fn it_send_message_whole_async() {
        // Simulate successful 'Send Message' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "provisionalMessageId": "pfX7fAXyskwHRrP29vEb"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/messages/send", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":"Test message #1","targetDevice":{"id":"d8YpQ7jgPBJEkBrnvp58"},"options":{"encoding":"utf8","encrypt":true,"offChain":true,"readConfirmation":false,"async":true}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.send_message(
            Message::Whole(String::from("Test message #1")),
            DeviceId {
                id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                is_prod_unique_id: None,
            },
            Some(SendMessageOptions {
                encoding: Some(Encoding::UTF8),
                encrypt: Some(true),
                off_chain: Some(true),
                storage: None,
                read_confirmation: Some(false),
                async_: Some(true),
            }),
        ).await.unwrap();

        assert_eq!(result, SendMessageResult {
            continuation_token: None,
            message_id: None,
            provisional_message_id: Some(String::from("pfX7fAXyskwHRrP29vEb")),
        });
    }

    #[tokio::test]
    async fn it_send_message_chunk() {
        // Simulate successful 'Send Message' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "continuationToken": "kwfLun4YjqKfjaNmEMey"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/messages/send", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"message":{"data":"Test message #1 (part 1)","isFinal":false},"targetDevice":{"id":"d8YpQ7jgPBJEkBrnvp58"},"options":{"encoding":"utf8","encrypt":true,"offChain":true,"readConfirmation":false}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.send_message(
            Message::Chunk(ChunkedMessage {
                data: Some(String::from("Test message #1 (part 1)")),
                is_final: Some(false),
                continuation_token: None,
            }),
            DeviceId {
                id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                is_prod_unique_id: None,
            },
            Some(SendMessageOptions {
                encoding: Some(Encoding::UTF8),
                encrypt: Some(true),
                off_chain: Some(true),
                storage: None,
                read_confirmation: Some(false),
                async_: None,
            }),
        ).await.unwrap();

        assert_eq!(result, SendMessageResult {
            continuation_token: Some(String::from("kwfLun4YjqKfjaNmEMey")),
            message_id: None,
            provisional_message_id: None,
        });
    }

    #[tokio::test]
    async fn it_read_message() {
        // Simulate successful 'Read Message' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "msgInfo": {
      "action": "send",
      "from": {
        "deviceId": "drc3XdxNtzoucpw9xiRp",
        "name": "TstDev1",
        "prodUniqueId": "ABC123"
      }
    },
    "msgData": "Test message #1"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/o3muoTnnD6cXYyarYY38?encoding=utf8", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "d8YpQ7jgPBJEkBrnvp58",
                "267a687115b9752f2eec5be849b570b29133528f928868d811bad5e48e97a1d62d432bab44803586b2ac35002ec6f0eeaa98bec79b64f2f69b9cb0935b4df2c4",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.read_message(
            "o3muoTnnD6cXYyarYY38",
            Some(ReadMessageOptions {
                encoding: Some(Encoding::UTF8),
                continuation_token: None,
                data_chunk_size: None,
                async_: None,
            }),
        ).await.unwrap();

        assert_eq!(result, ReadMessageResult {
            msg_info: Some(MessageInfo {
                action: RecordMessageAction::Send,
                from: Some(DeviceInfo {
                    device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                    name: Some(String::from("TstDev1")),
                    prod_unique_id: Some(String::from("ABC123")),
                }),
            }),
            msg_data: Some(String::from("Test message #1")),
            continuation_token: None,
            cached_message_id: None,
        });
    }

    #[tokio::test]
    async fn it_read_message_async() {
        // Simulate successful 'Read Message' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "cachedMessageId": "h9W32ic3TxDwusgCQByw"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/o3muoTnnD6cXYyarYY38?encoding=utf8&async=true", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "d8YpQ7jgPBJEkBrnvp58",
                "267a687115b9752f2eec5be849b570b29133528f928868d811bad5e48e97a1d62d432bab44803586b2ac35002ec6f0eeaa98bec79b64f2f69b9cb0935b4df2c4",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.read_message(
            "o3muoTnnD6cXYyarYY38",
            Some(ReadMessageOptions {
                encoding: Some(Encoding::UTF8),
                continuation_token: None,
                data_chunk_size: None,
                async_: Some(true),
            }),
        ).await.unwrap();

        assert_eq!(result, ReadMessageResult {
            msg_info: None,
            msg_data: None,
            continuation_token: None,
            cached_message_id: Some(String::from("h9W32ic3TxDwusgCQByw")),
        });
    }

    #[tokio::test]
    async fn it_read_message_in_chunks() {
        // Simulate successful 'Read Message' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "msgInfo": {
      "action": "send",
      "from": {
        "deviceId": "drc3XdxNtzoucpw9xiRp",
        "name": "TstDev1",
        "prodUniqueId": "ABC123"
      }
    },
    "msgData": "Test messa",
    "continuationToken": "kSeBcY95kWQCJeRnuxxt"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/oydotwHEvsZFjrbduoSD?encoding=utf8&dataChunkSize=10", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "d8YpQ7jgPBJEkBrnvp58",
                "267a687115b9752f2eec5be849b570b29133528f928868d811bad5e48e97a1d62d432bab44803586b2ac35002ec6f0eeaa98bec79b64f2f69b9cb0935b4df2c4",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.read_message(
            "oydotwHEvsZFjrbduoSD",
            Some(ReadMessageOptions {
                encoding: Some(Encoding::UTF8),
                continuation_token: None,
                data_chunk_size: Some(10),
                async_: None,
            }),
        ).await.unwrap();

        assert_eq!(result, ReadMessageResult {
            msg_info: Some(MessageInfo {
                action: RecordMessageAction::Send,
                from: Some(DeviceInfo {
                    device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                    name: Some(String::from("TstDev1")),
                    prod_unique_id: Some(String::from("ABC123")),
                }),
            }),
            msg_data: Some(String::from("Test messa")),
            continuation_token: Some(String::from("kSeBcY95kWQCJeRnuxxt")),
            cached_message_id: None,
        });
    }

    #[tokio::test]
    async fn it_retrieve_message_container() {
        // Simulate successful 'Retrieve Message Container' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "blockchain": {
      "txid": "2b04137557772b9265349956bc8a1974f095ade819d6e058656f100357522bc9",
      "isConfirmed": false
    }
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/mNAgNarEy6a52X57sXWe/container", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_message_container(
            "mNAgNarEy6a52X57sXWe",
        ).await.unwrap();

        assert_eq!(result, RetrieveMessageContainerResult {
            off_chain: None,
            blockchain: Some(BlockchainContainer {
                txid: String::from("2b04137557772b9265349956bc8a1974f095ade819d6e058656f100357522bc9"),
                is_confirmed: false,
            }),
            external_storage: None,
        });
    }

    #[tokio::test]
    async fn it_retrieve_ipfs_message_container() {
        // Simulate successful 'Retrieve Message Container' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "blockchain": {
      "txid": "eec2551b30d851052b8487339879d8e991ce026021f6c9a4c3d23a98a16e5cfc",
      "isConfirmed": false
    },
    "externalStorage": {
      "ipfs": "Qmdbu9b6tp1uFNENfJsTG5w8ZyzfqL6hRfs5A35PFGMkrK"
    }
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/mrdwKkhNtDSMjBJSHCsb/container", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_message_container(
            "mrdwKkhNtDSMjBJSHCsb",
        ).await.unwrap();

        assert_eq!(result, RetrieveMessageContainerResult {
            off_chain: None,
            blockchain: Some(BlockchainContainer {
                txid: String::from("eec2551b30d851052b8487339879d8e991ce026021f6c9a4c3d23a98a16e5cfc"),
                is_confirmed: false,
            }),
            external_storage: Some(IpfsStorage {
                ipfs: String::from("Qmdbu9b6tp1uFNENfJsTG5w8ZyzfqL6hRfs5A35PFGMkrK"),
            }),
        });
    }

    #[tokio::test]
    async fn it_retrieve_off_chain_message_container() {
        // Simulate successful 'Retrieve Message Container' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "offChain": {
      "cid": "QmStQXCLiPtza3dGiXvU54mFdCK1ymGRUbzkTsotZCRhNu"
    },
    "blockchain": {
      "txid": "c3bc4e5bbd971e6f0be281225d020363ebeb068e65b0631a4a58b07f1d003d49",
      "isConfirmed": true
    },
    "externalStorage": {
      "ipfs": "QmWmxXdrrKfJExacaRJsquhwjkZ1UGnPXBZRamTVTy9E26"
    }
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/oX2mJHwFWp752beHbNDK/container", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_message_container(
            "oX2mJHwFWp752beHbNDK",
        ).await.unwrap();

        assert_eq!(result, RetrieveMessageContainerResult {
            off_chain: Some(OffChainContainer {
                cid: String::from("QmStQXCLiPtza3dGiXvU54mFdCK1ymGRUbzkTsotZCRhNu"),
            }),
            blockchain: Some(BlockchainContainer {
                txid: String::from("c3bc4e5bbd971e6f0be281225d020363ebeb068e65b0631a4a58b07f1d003d49"),
                is_confirmed: true,
            }),
            external_storage: Some(IpfsStorage {
                ipfs: String::from("QmWmxXdrrKfJExacaRJsquhwjkZ1UGnPXBZRamTVTy9E26"),
            }),
        });
    }

    #[tokio::test]
    async fn it_retrieve_message_origin() {
        // Simulate successful 'Retrieve Message Origin' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "tx": {
      "txid": "2b04137557772b9265349956bc8a1974f095ade819d6e058656f100357522bc9",
      "type": "Log Message",
      "originDevice": {
        "address": "bcrt1qt5kzpg0z05ypdhkcmcu2ve7exjmweqydhkdyar",
        "deviceId": "drc3XdxNtzoucpw9xiRp",
        "name": "TstDev1",
        "prodUniqueId": "ABC123",
        "ownedBy": {
          "company": "Blockchain of Things",
          "contact": "Cláudio de Castro",
          "domains": [
            "blockchainofthings.com",
            "catenis.io"
          ]
        }
      }
    }
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/mNAgNarEy6a52X57sXWe/origin", DEFAULT_API_VERSION.to_string())),
            headers: None,
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let ctn_client = CatenisClient::new_with_options(
            None,
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_message_origin(
            "mNAgNarEy6a52X57sXWe",
            None,
        ).await.unwrap();

        assert_eq!(result, RetrieveMessageOriginResult {
            tx: Some(BlockchainTransaction {
                txid: String::from("2b04137557772b9265349956bc8a1974f095ade819d6e058656f100357522bc9"),
                type_: TransactionType::LogMessage,
                batch_doc: None,
                origin_device: Some(OriginDeviceInfo {
                    address: String::from("bcrt1qt5kzpg0z05ypdhkcmcu2ve7exjmweqydhkdyar"),
                    device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                    name: Some(String::from("TstDev1")),
                    prod_unique_id: Some(String::from("ABC123")),
                    owned_by: DeviceOwner {
                        company: Some(String::from("Blockchain of Things")),
                        contact: Some(String::from("Cláudio de Castro")),
                        name: None,
                        domains: Some(vec![
                            String::from("blockchainofthings.com"),
                            String::from("catenis.io"),
                        ]),
                    },
                }),
            }),
            off_chain_msg_envelope: None,
            proof: None,
        });
    }

    #[tokio::test]
    async fn it_retrieve_message_origin_with_proof() {
        // Simulate successful 'Retrieve Message Origin' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "tx": {
      "txid": "2b04137557772b9265349956bc8a1974f095ade819d6e058656f100357522bc9",
      "type": "Log Message",
      "originDevice": {
        "address": "bcrt1qt5kzpg0z05ypdhkcmcu2ve7exjmweqydhkdyar",
        "deviceId": "drc3XdxNtzoucpw9xiRp",
        "name": "TstDev1",
        "prodUniqueId": "ABC123",
        "ownedBy": {
          "company": "Blockchain of Things",
          "contact": "Cláudio de Castro",
          "domains": [
            "blockchainofthings.com",
            "catenis.io"
          ]
        }
      }
    },
    "proof": {
      "message": "This is only a test",
      "signature": "J8Xa/oOfkTd1KmuavLSuV7s18p1YFTlQLNd4dgon+3u6UGk5/pYJswYzlQ/sIpuh7V1Oz3eFijOqh+08JUyIoIw="
    }
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/mNAgNarEy6a52X57sXWe/origin?msgToSign=This+is+only+a+test", DEFAULT_API_VERSION.to_string())),
            headers: None,
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let ctn_client = CatenisClient::new_with_options(
            None,
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_message_origin(
            "mNAgNarEy6a52X57sXWe",
            Some("This is only a test"),
        ).await.unwrap();

        assert_eq!(result, RetrieveMessageOriginResult {
            tx: Some(BlockchainTransaction {
                txid: String::from("2b04137557772b9265349956bc8a1974f095ade819d6e058656f100357522bc9"),
                type_: TransactionType::LogMessage,
                batch_doc: None,
                origin_device: Some(OriginDeviceInfo {
                    address: String::from("bcrt1qt5kzpg0z05ypdhkcmcu2ve7exjmweqydhkdyar"),
                    device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                    name: Some(String::from("TstDev1")),
                    prod_unique_id: Some(String::from("ABC123")),
                    owned_by: DeviceOwner {
                        company: Some(String::from("Blockchain of Things")),
                        contact: Some(String::from("Cláudio de Castro")),
                        name: None,
                        domains: Some(vec![
                            String::from("blockchainofthings.com"),
                            String::from("catenis.io"),
                        ]),
                    },
                }),
            }),
            off_chain_msg_envelope: None,
            proof: Some(ProofInfo {
                message: String::from("This is only a test"),
                signature: String::from("J8Xa/oOfkTd1KmuavLSuV7s18p1YFTlQLNd4dgon+3u6UGk5/pYJswYzlQ/sIpuh7V1Oz3eFijOqh+08JUyIoIw="),
            }),
        });
    }

    #[tokio::test]
    async fn it_retrieve_off_chain_message_origin() {
        // Simulate successful 'Retrieve Message Origin' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "tx": {
      "txid": "c3bc4e5bbd971e6f0be281225d020363ebeb068e65b0631a4a58b07f1d003d49",
      "type": "Settle Off-Chain Messages",
      "batchDoc": {
        "cid": "Qmc55NfZ18gKkP7rgDXsmrSWndgDcEuEXmMGTPrBF3EqTw"
      }
    },
    "offChainMsgEnvelope": {
      "cid": "QmStQXCLiPtza3dGiXvU54mFdCK1ymGRUbzkTsotZCRhNu",
      "type": "Log Message",
      "originDevice": {
        "pubKeyHash": "0aaf59f937a7e291af500937c28a249857f37cc4",
        "deviceId": "drc3XdxNtzoucpw9xiRp",
        "name": "TstDev1",
        "prodUniqueId": "ABC123",
        "ownedBy": {
          "company": "Blockchain of Things",
          "contact": "Cláudio de Castro",
          "domains": [
            "blockchainofthings.com",
            "catenis.io"
          ]
        }
      }
    }
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/oX2mJHwFWp752beHbNDK/origin", DEFAULT_API_VERSION.to_string())),
            headers: None,
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let ctn_client = CatenisClient::new_with_options(
            None,
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_message_origin(
            "oX2mJHwFWp752beHbNDK",
            None,
        ).await.unwrap();

        assert_eq!(result, RetrieveMessageOriginResult {
            tx: Some(BlockchainTransaction {
                txid: String::from("c3bc4e5bbd971e6f0be281225d020363ebeb068e65b0631a4a58b07f1d003d49"),
                type_: TransactionType::SettleOffChainMessages,
                batch_doc: Some(BatchDocRef {
                    cid: String::from("Qmc55NfZ18gKkP7rgDXsmrSWndgDcEuEXmMGTPrBF3EqTw"),
                }),
                origin_device: None,
            }),
            off_chain_msg_envelope: Some(OffChainMsgEnvelope {
                cid: String::from("QmStQXCLiPtza3dGiXvU54mFdCK1ymGRUbzkTsotZCRhNu"),
                type_: OffChainMessageType::LogMessage,
                origin_device: Some(OffChainOriginDeviceInfo {
                    pub_key_hash: String::from("0aaf59f937a7e291af500937c28a249857f37cc4"),
                    device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                    name: Some(String::from("TstDev1")),
                    prod_unique_id: Some(String::from("ABC123")),
                    owned_by: DeviceOwner {
                        company: Some(String::from("Blockchain of Things")),
                        contact: Some(String::from("Cláudio de Castro")),
                        name: None,
                        domains: Some(vec![
                            String::from("blockchainofthings.com"),
                            String::from("catenis.io"),
                        ]),
                    },
                }),
            }),
            proof: None,
        });
    }

    #[tokio::test]
    async fn it_retrieve_off_chain_message_origin_with_proof() {
        // Simulate successful 'Retrieve Message Origin' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "tx": {
      "txid": "c3bc4e5bbd971e6f0be281225d020363ebeb068e65b0631a4a58b07f1d003d49",
      "type": "Settle Off-Chain Messages",
      "batchDoc": {
        "cid": "Qmc55NfZ18gKkP7rgDXsmrSWndgDcEuEXmMGTPrBF3EqTw"
      }
    },
    "offChainMsgEnvelope": {
      "cid": "QmStQXCLiPtza3dGiXvU54mFdCK1ymGRUbzkTsotZCRhNu",
      "type": "Log Message",
      "originDevice": {
        "pubKeyHash": "0aaf59f937a7e291af500937c28a249857f37cc4",
        "deviceId": "drc3XdxNtzoucpw9xiRp",
        "name": "TstDev1",
        "prodUniqueId": "ABC123",
        "ownedBy": {
          "company": "Blockchain of Things",
          "contact": "Cláudio de Castro",
          "domains": [
            "blockchainofthings.com",
            "catenis.io"
          ]
        }
      }
    },
    "proof": {
      "message": "This is only a test",
      "signature": "IJqbOc0MCqplwYWIfMHLXU4dcGPcPwmxWx+SmF+ulguNaDSJrOTogYlyiLF+8UdeV1CqeyWj49da70w8VGXjWsI="
    }
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/oX2mJHwFWp752beHbNDK/origin?msgToSign=This+is+only+a+test", DEFAULT_API_VERSION.to_string())),
            headers: None,
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let ctn_client = CatenisClient::new_with_options(
            None,
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_message_origin(
            "oX2mJHwFWp752beHbNDK",
            Some("This is only a test"),
        ).await.unwrap();

        assert_eq!(result, RetrieveMessageOriginResult {
            tx: Some(BlockchainTransaction {
                txid: String::from("c3bc4e5bbd971e6f0be281225d020363ebeb068e65b0631a4a58b07f1d003d49"),
                type_: TransactionType::SettleOffChainMessages,
                batch_doc: Some(BatchDocRef {
                    cid: String::from("Qmc55NfZ18gKkP7rgDXsmrSWndgDcEuEXmMGTPrBF3EqTw"),
                }),
                origin_device: None,
            }),
            off_chain_msg_envelope: Some(OffChainMsgEnvelope {
                cid: String::from("QmStQXCLiPtza3dGiXvU54mFdCK1ymGRUbzkTsotZCRhNu"),
                type_: OffChainMessageType::LogMessage,
                origin_device: Some(OffChainOriginDeviceInfo {
                    pub_key_hash: String::from("0aaf59f937a7e291af500937c28a249857f37cc4"),
                    device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                    name: Some(String::from("TstDev1")),
                    prod_unique_id: Some(String::from("ABC123")),
                    owned_by: DeviceOwner {
                        company: Some(String::from("Blockchain of Things")),
                        contact: Some(String::from("Cláudio de Castro")),
                        name: None,
                        domains: Some(vec![
                            String::from("blockchainofthings.com"),
                            String::from("catenis.io"),
                        ]),
                    },
                }),
            }),
            proof: Some(ProofInfo {
                message: String::from("This is only a test"),
                signature: String::from("IJqbOc0MCqplwYWIfMHLXU4dcGPcPwmxWx+SmF+ulguNaDSJrOTogYlyiLF+8UdeV1CqeyWj49da70w8VGXjWsI="),
            }),
        });
    }

    #[tokio::test]
    async fn it_retrieve_log_message_progress() {
        // Simulate successful 'Retrieve Message Progress' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "action": "log",
    "progress": {
      "bytesProcessed": 15,
      "done": true,
      "success": true,
      "finishDate": "2020-12-22T20:39:14.945Z"
    },
    "result": {
      "messageId": "m9eZa29ezn69dj8PWyby"
    }
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/pTZCgjKYEyHfu4rNPWct/progress", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_message_progress(
            "pTZCgjKYEyHfu4rNPWct",
        ).await.unwrap();

        assert_eq!(result, RetrieveMessageProgressResult {
            action: MessageAction::Log,
            progress: MessageProcessProgress {
                bytes_processed: 15,
                done: true,
                success: Some(true),
                error: None,
                finish_date: Some("2020-12-22T20:39:14.945Z".into()),
            },
            result: Some(MessageProcessSuccess {
                message_id: String::from("m9eZa29ezn69dj8PWyby"),
                continuation_token: None,
            }),
        });
    }

    #[tokio::test]
    async fn it_retrieve_read_message_progress() {
        // Simulate successful 'Retrieve Message Progress' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "action": "read",
    "progress": {
      "bytesProcessed": 15,
      "done": true,
      "success": true,
      "finishDate": "2020-12-22T20:59:23.468Z"
    },
    "result": {
      "messageId": "mq8hdaBLsnrfPx922tgt",
      "continuationToken": "kLg9NA2zGMdeoTG72Ncs"
    }
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages/hE6dX6vdtwwDx9tbdtS7/progress", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_message_progress(
            "hE6dX6vdtwwDx9tbdtS7",
        ).await.unwrap();

        assert_eq!(result, RetrieveMessageProgressResult {
            action: MessageAction::Read,
            progress: MessageProcessProgress {
                bytes_processed: 15,
                done: true,
                success: Some(true),
                error: None,
                finish_date: Some("2020-12-22T20:59:23.468Z".into()),
            },
            result: Some(MessageProcessSuccess {
                message_id: String::from("mq8hdaBLsnrfPx922tgt"),
                continuation_token: Some(String::from("kLg9NA2zGMdeoTG72Ncs")),
            }),
        });
    }

    #[tokio::test]
    async fn it_list_messages() {
        // Simulate successful 'List Messages' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "messages": [
      {
        "messageId": "o3zWcDpLGzzKACEarFTP",
        "action": "send",
        "direction": "inbound",
        "from": {
          "deviceId": "d8YpQ7jgPBJEkBrnvp58",
          "name": "TstDev2"
        },
        "read": false,
        "date": "2020-12-22T17:26:54.868Z"
      },
      {
        "messageId": "oydotwHEvsZFjrbduoSD",
        "action": "send",
        "direction": "outbound",
        "to": {
          "deviceId": "d8YpQ7jgPBJEkBrnvp58",
          "name": "TstDev2"
        },
        "readConfirmationEnabled": false,
        "date": "2020-12-22T17:31:58.137Z"
      },
      {
        "messageId": "oBWDDXEYBa7aShx2ckfE",
        "action": "log",
        "read": false,
        "date": "2020-12-22T17:44:36.776Z"
      },
      {
        "messageId": "mNAgNarEy6a52X57sXWe",
        "action": "log",
        "read": false,
        "date": "2020-12-22T17:44:59.755Z"
      }
    ],
    "msgCount": 4,
    "hasMore": true
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/messages?startDate=2020-12-22T00%3A00%3A00.000Z&limit=4", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.list_messages(
            Some(ListMessagesOptions {
                action: None,
                direction: None,
                from_devices: None,
                to_devices: None,
                read_state: None,
                start_date: Some("2020-12-22T00:00:00Z".into()),
                end_date: None,
                limit: Some(4),
                skip: None,
            })
        ).await.unwrap();

        assert_eq!(result, ListMessagesResult {
            messages: vec![
                MessageEntry {
                    message_id: String::from("o3zWcDpLGzzKACEarFTP"),
                    action: RecordMessageAction::Send,
                    direction: Some(MessageDirection::Inbound),
                    from: Some(DeviceInfo {
                        device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                        name: Some(String::from("TstDev2")),
                        prod_unique_id: None,
                    }),
                    to: None,
                    read_confirmation_enabled: None,
                    read: Some(false),
                    date: "2020-12-22T17:26:54.868Z".into(),
                },
                MessageEntry {
                    message_id: String::from("oydotwHEvsZFjrbduoSD"),
                    action: RecordMessageAction::Send,
                    direction: Some(MessageDirection::Outbound),
                    from: None,
                    to: Some(DeviceInfo {
                        device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                        name: Some(String::from("TstDev2")),
                        prod_unique_id: None,
                    }),
                    read_confirmation_enabled: Some(false),
                    read: None,
                    date: "2020-12-22T17:31:58.137Z".into(),
                },
                MessageEntry {
                    message_id: String::from("oBWDDXEYBa7aShx2ckfE"),
                    action: RecordMessageAction::Log,
                    direction: None,
                    from: None,
                    to: None,
                    read_confirmation_enabled: None,
                    read: Some(false),
                    date: "2020-12-22T17:44:36.776Z".into(),
                },
                MessageEntry {
                    message_id: String::from("mNAgNarEy6a52X57sXWe"),
                    action: RecordMessageAction::Log,
                    direction: None,
                    from: None,
                    to: None,
                    read_confirmation_enabled: None,
                    read: Some(false),
                    date: "2020-12-22T17:44:59.755Z".into(),
                },
            ],
            msg_count: 4,
            has_more: true,
        });
    }

    #[tokio::test]
    async fn it_issue_asset() {
        // Simulate successful 'Issue Asset' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "assetId": "aBy2ovnucyWaSB6Tro9x"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/assets/issue", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"assetInfo":{"name":"RCT-001","description":"Test asset #1","canReissue":true,"decimalPlaces":2},"amount":1500.0}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.issue_asset(
            NewAssetInfo {
                name: String::from("RCT-001"),
                description: Some(String::from("Test asset #1")),
                can_reissue: true,
                decimal_places: 2,
            },
            1500.0,
            None,
        ).await.unwrap();

        assert_eq!(result, IssueAssetResult {
            asset_id: String::from("aBy2ovnucyWaSB6Tro9x"),
        });
    }

    #[tokio::test]
    async fn it_reissue_asset() {
        // Simulate successful 'Reissue Asset' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "totalExistentBalance": 1612.5
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/assets/aBy2ovnucyWaSB6Tro9x/issue", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"amount":112.5,"holdingDevice":{"id":"d8YpQ7jgPBJEkBrnvp58"}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.reissue_asset(
            "aBy2ovnucyWaSB6Tro9x",
            112.5,
            Some(DeviceId {
                id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                is_prod_unique_id: None,
            }),
        ).await.unwrap();

        assert_eq!(result, ReissueAssetResult {
            total_existent_balance: 1612.5,
        });
    }

    #[tokio::test]
    async fn it_transfer_asset() {
        // Simulate successful 'Transfer Asset' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "remainingBalance": 1445.75
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/assets/aBy2ovnucyWaSB6Tro9x/transfer", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"amount":54.25,"receivingDevice":{"id":"d8YpQ7jgPBJEkBrnvp58"}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.transfer_asset(
            "aBy2ovnucyWaSB6Tro9x",
            54.25,
            DeviceId {
                id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                is_prod_unique_id: None,
            },
        ).await.unwrap();

        assert_eq!(result, TransferAssetResult {
            remaining_balance: 1445.75,
        });
    }

    #[tokio::test]
    async fn it_retrieve_asset_info() {
        // Simulate successful 'Retrieve Asset Info' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "assetId": "aBy2ovnucyWaSB6Tro9x",
    "name": "RCT-001",
    "description": "Test asset #1",
    "canReissue": true,
    "decimalPlaces": 2,
    "issuer": {
      "deviceId": "drc3XdxNtzoucpw9xiRp",
      "name": "TstDev1",
      "prodUniqueId": "ABC123"
    },
    "totalExistentBalance": 1612.5
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/assets/aBy2ovnucyWaSB6Tro9x", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_asset_info(
            "aBy2ovnucyWaSB6Tro9x",
        ).await.unwrap();

        assert_eq!(result, RetrieveAssetInfoResult {
            asset_id: String::from("aBy2ovnucyWaSB6Tro9x"),
            name: String::from("RCT-001"),
            description: String::from("Test asset #1"),
            can_reissue: true,
            decimal_places: 2,
            issuer: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: Some(String::from("TstDev1")),
                prod_unique_id: Some(String::from("ABC123")),
            },
            total_existent_balance: 1612.5,
        });
    }

    #[tokio::test]
    async fn it_get_asset_balance() {
        // Simulate successful 'Get Asset Balance' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "total": 1445.75,
    "unconfirmed": 0
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/assets/aBy2ovnucyWaSB6Tro9x/balance", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.get_asset_balance(
            "aBy2ovnucyWaSB6Tro9x",
        ).await.unwrap();

        assert_eq!(result, GetAssetBalanceResult {
            total: 1445.75,
            unconfirmed: 0.0,
        });
    }

    #[tokio::test]
    async fn it_list_owned_assets() {
        // Simulate successful 'List Owned Assets' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "ownedAssets": [
      {
        "assetId": "aA2XY6SLvcNAjkQSrQXR",
        "balance": {
          "total": 150,
          "unconfirmed": 0
        }
      },
      {
        "assetId": "aBy2ovnucyWaSB6Tro9x",
        "balance": {
          "total": 1445.75,
          "unconfirmed": 0
        }
      },
      {
        "assetId": "aLdqZdNAaxisqySiXtQZ",
        "balance": {
          "total": 150,
          "unconfirmed": 0
        }
      }
    ],
    "hasMore": true
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/assets/owned?limit=3&skip=38", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.list_owned_assets(
            Some(3),
            Some(38),
        ).await.unwrap();

        assert_eq!(result, ListOwnedAssetsResult {
            owned_assets: vec![
                OwnedAssetEntry {
                    asset_id: String::from("aA2XY6SLvcNAjkQSrQXR"),
                    balance: AssetBalance {
                        total: 150.0,
                        unconfirmed: 0.0,
                    },
                },
                OwnedAssetEntry {
                    asset_id: String::from("aBy2ovnucyWaSB6Tro9x"),
                    balance: AssetBalance {
                        total: 1445.75,
                        unconfirmed: 0.0,
                    },
                },
                OwnedAssetEntry {
                    asset_id: String::from("aLdqZdNAaxisqySiXtQZ"),
                    balance: AssetBalance {
                        total: 150.0,
                        unconfirmed: 0.0,
                    },
                },
            ],
            has_more: true,
        });
    }

    #[tokio::test]
    async fn it_list_issued_assets() {
        // Simulate successful 'List Issued Assets' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "issuedAssets": [
      {
        "assetId": "aN8YerJ9XpgKeWogYiih",
        "totalExistentBalance": 25
      },
      {
        "assetId": "aBy2ovnucyWaSB6Tro9x",
        "totalExistentBalance": 1612.5
      },
      {
        "assetId": "a4wk2EXWEKHzyoK7ZYgS",
        "totalExistentBalance": 1500
      }
    ],
    "hasMore": false
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/assets/issued?skip=157", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.list_issued_assets(
            None,
            Some(157),
        ).await.unwrap();

        assert_eq!(result, ListIssuedAssetsResult {
            issued_assets: vec![
                IssuedAssetEntry {
                    asset_id: String::from("aN8YerJ9XpgKeWogYiih"),
                    total_existent_balance: 25.0,
                },
                IssuedAssetEntry {
                    asset_id: String::from("aBy2ovnucyWaSB6Tro9x"),
                    total_existent_balance: 1612.5,
                },
                IssuedAssetEntry {
                    asset_id: String::from("a4wk2EXWEKHzyoK7ZYgS"),
                    total_existent_balance: 1500.0,
                },
            ],
            has_more: false,
        });
    }

    #[tokio::test]
    async fn it_retrieve_asset_issuance_history() {
        // Simulate successful 'Retrieve Asset Issuance History' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "issuanceEvents": [
      {
        "amount": 1500,
        "holdingDevice": {
          "deviceId": "drc3XdxNtzoucpw9xiRp",
          "name": "TstDev1",
          "prodUniqueId": "ABC123"
        },
        "date": "2020-12-23T10:51:45.935Z"
      },
      {
        "amount": 112.5,
        "holdingDevice": {
          "deviceId": "d8YpQ7jgPBJEkBrnvp58",
          "name": "TstDev2"
        },
        "date": "2020-12-23T11:17:23.731Z"
      }
    ],
    "hasMore": false
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/assets/aBy2ovnucyWaSB6Tro9x/issuance?startDate=2020-12-01T00%3A00%3A00.000Z", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_asset_issuance_history(
            "aBy2ovnucyWaSB6Tro9x",
            Some("2020-12-01T00:00:00Z".into()),
            None,
            None,
            None,
        ).await.unwrap();

        assert_eq!(result, RetrieveAssetIssuanceHistoryResult {
            issuance_events: vec![
                AssetIssuanceEventEntry {
                    amount: 1500.0,
                    holding_device: DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: Some(String::from("TstDev1")),
                        prod_unique_id: Some(String::from("ABC123")),
                    },
                    date: "2020-12-23T10:51:45.935Z".into(),
                },
                AssetIssuanceEventEntry {
                    amount: 112.5,
                    holding_device: DeviceInfo {
                        device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                        name: Some(String::from("TstDev2")),
                        prod_unique_id: None,
                    },
                    date: "2020-12-23T11:17:23.731Z".into(),
                },
            ],
            has_more: false,
        });
    }

    #[tokio::test]
    async fn it_list_asset_holders() {
        // Simulate successful 'List Asset Holders' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "assetHolders": [
      {
        "holder": {
          "deviceId": "d8YpQ7jgPBJEkBrnvp58",
          "name": "TstDev2"
        },
        "balance": {
          "total": 166.75,
          "unconfirmed": 0
        }
      },
      {
        "holder": {
          "deviceId": "drc3XdxNtzoucpw9xiRp",
          "name": "TstDev1",
          "prodUniqueId": "ABC123"
        },
        "balance": {
          "total": 1445.75,
          "unconfirmed": 0
        }
      },
      {
        "migrated": true,
        "balance": {
          "total": 387.5,
          "unconfirmed": 0
        }
      }
    ],
    "hasMore": false
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/assets/aBy2ovnucyWaSB6Tro9x/holders", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.list_asset_holders(
            "aBy2ovnucyWaSB6Tro9x",
            None,
            None,
        ).await.unwrap();

        assert_eq!(result, ListAssetHoldersResult {
            asset_holders: vec![
                AssetHolderEntry {
                    holder: Some(DeviceInfo {
                        device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                        name: Some(String::from("TstDev2")),
                        prod_unique_id: None,
                    }),
                    migrated: None,
                    balance: AssetBalance {
                        total: 166.75,
                        unconfirmed: 0.0,
                    },
                },
                AssetHolderEntry {
                    holder: Some(DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: Some(String::from("TstDev1")),
                        prod_unique_id: Some(String::from("ABC123")),
                    }),
                    migrated: None,
                    balance: AssetBalance {
                        total: 1445.75,
                        unconfirmed: 0.0,
                    },
                },
                AssetHolderEntry {
                    holder: None,
                    migrated: Some(true),
                    balance: AssetBalance {
                        total: 387.5,
                        unconfirmed: 0.0,
                    },
                },
            ],
            has_more: false,
        });
    }

    #[tokio::test]
    async fn it_export_asset_estimate_only() {
        // Simulate successful 'Export Asset' API method response for estimate only

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "estimatedPrice": "0.05850782"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/assets/aCSy24HLjKMbpnvJ8GTx/export/ethereum", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"token":{"name":"Catenis test token #11","symbol":"CTK11"},"options":{"consumptionProfile":"fastest","estimateOnly":true}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.export_asset(
            "aCSy24HLjKMbpnvJ8GTx",
            ForeignBlockchain::Ethereum,
            NewForeignTokenInfo {
                name: String::from("Catenis test token #11"),
                symbol: String::from("CTK11"),
            },
            Some(ExportAssetOptions {
                consumption_profile: Some(ForeignConsumptionProfile::Fastest),
                estimate_only: Some(true),
            }),
        ).await.unwrap();

        assert_eq!(result, ExportAssetResult {
            foreign_transaction: None,
            token: None,
            status: None,
            date: None,
            estimated_price: Some(String::from("0.05850782")),
        });
    }

    #[tokio::test]
    async fn it_export_asset_regular() {
        // Simulate successful 'Export Asset' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "foreignTransaction": {
      "txid": "0x6299c35ccfa803ab0cb043e8d8ae4be8d7f3432d85f288ebb81e4d624e566b0a",
      "isPending": true
    },
    "token": {
      "name": "Catenis test token #11",
      "symbol": "CTK11"
    },
    "status": "pending",
    "date": "2021-08-10T12:57:12.583Z"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/assets/aCSy24HLjKMbpnvJ8GTx/export/ethereum", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"token":{"name":"Catenis test token #11","symbol":"CTK11"},"options":{"consumptionProfile":"fastest"}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.export_asset(
            "aCSy24HLjKMbpnvJ8GTx",
            ForeignBlockchain::Ethereum,
            NewForeignTokenInfo {
                name: String::from("Catenis test token #11"),
                symbol: String::from("CTK11"),
            },
            Some(ExportAssetOptions {
                consumption_profile: Some(ForeignConsumptionProfile::Fastest),
                estimate_only: None,
            }),
        ).await.unwrap();

        assert_eq!(result, ExportAssetResult {
            foreign_transaction: Some(ForeignTransactionInfo {
                txid: String::from("0x6299c35ccfa803ab0cb043e8d8ae4be8d7f3432d85f288ebb81e4d624e566b0a"),
                is_pending: true,
                success: None,
                error: None,
            }),
            token: Some(ForeignTokenInfo {
                name: String::from("Catenis test token #11"),
                symbol: String::from("CTK11"),
                id: None,
            }),
            status: Some(AssetExportStatus::Pending),
            date: Some("2021-08-10T12:57:12.583Z".into()),
            estimated_price: None,
        });
    }

    #[tokio::test]
    async fn it_migrate_asset_estimate_only() {
        // Simulate successful 'Migrate Asset' API method response for estimate only

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "estimatedPrice": "0.001723913"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/assets/aCSy24HLjKMbpnvJ8GTx/migrate/ethereum", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"migration":{"direction":"outward","amount":25.3,"destAddress":"0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943"},"options":{"consumptionProfile":"slow","estimateOnly":true}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.migrate_asset(
            "aCSy24HLjKMbpnvJ8GTx",
            ForeignBlockchain::Ethereum,
            AssetMigration::Info(AssetMigrationInfo {
                direction: AssetMigrationDirection::Outward,
                amount: 25.3,
                dest_address: Some(String::from("0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943")),
            }),
            Some(MigrateAssetOptions {
                consumption_profile: Some(ForeignConsumptionProfile::Slow),
                estimate_only: Some(true),
            }),
        ).await.unwrap();

        assert_eq!(result, MigrateAssetResult {
            migration_id: None,
            catenis_service: None,
            foreign_transaction: None,
            status: None,
            date: None,
            estimated_price: Some(String::from("0.001723913")),
        });
    }

    #[tokio::test]
    async fn it_migrate_asset_regular() {
        // Simulate successful 'Migrate Asset' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "migrationId": "gTQ8Qf5W6kdmdYdEEoD9",
    "catenisService": {
      "status": "fulfilled",
      "txid": "7d6a20ee009ad2bcbf5c799ee4eac594e4447bdb5007250f8ba038de97f63777"
    },
    "foreignTransaction": {
      "txid": "0x92fb47432e50b623441bb3b55dd65bf879183f87ea4913a16e75503c98792df9",
      "isPending": true
    },
    "status": "pending",
    "date": "2021-08-10T12:59:41.492Z"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/assets/aCSy24HLjKMbpnvJ8GTx/migrate/ethereum", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"migration":{"direction":"outward","amount":25.3,"destAddress":"0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943"},"options":{"consumptionProfile":"slow"}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.migrate_asset(
            "aCSy24HLjKMbpnvJ8GTx",
            ForeignBlockchain::Ethereum,
            AssetMigration::Info(AssetMigrationInfo {
                direction: AssetMigrationDirection::Outward,
                amount: 25.3,
                dest_address: Some(String::from("0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943")),
            }),
            Some(MigrateAssetOptions {
                consumption_profile: Some(ForeignConsumptionProfile::Slow),
                estimate_only: None,
            }),
        ).await.unwrap();

        assert_eq!(result, MigrateAssetResult {
            migration_id: Some(String::from("gTQ8Qf5W6kdmdYdEEoD9")),
            catenis_service: Some(CatenisServiceInfo {
                status: CatenisServiceStatus::Fulfilled,
                txid: Some(String::from("7d6a20ee009ad2bcbf5c799ee4eac594e4447bdb5007250f8ba038de97f63777")),
                error: None,
            }),
            foreign_transaction: Some(ForeignTransactionInfo {
                txid: String::from("0x92fb47432e50b623441bb3b55dd65bf879183f87ea4913a16e75503c98792df9"),
                is_pending: true,
                success: None,
                error: None,
            }),
            status: Some(AssetMigrationStatus::Pending),
            date: Some("2021-08-10T12:59:41.492Z".into()),
            estimated_price: None,
        });
    }

    #[tokio::test]
    async fn it_migrate_asset_retry() {
        // Simulate successful 'Migrate Asset' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "migrationId": "gSLb9FTdGxgSLufuNzhR",
    "catenisService": {
      "status": "awaiting"
    },
    "foreignTransaction": {
      "txid": "0x883a4d9e02713b177fdd26b33e871dc765db3c964f2b1ef8e6f97eca24d718ee",
      "isPending": true
    },
    "status": "pending",
    "date": "2021-08-03T19:11:10.350Z"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/assets/aCSy24HLjKMbpnvJ8GTx/migrate/ethereum", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"migration":"gSLb9FTdGxgSLufuNzhR"}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.migrate_asset(
            "aCSy24HLjKMbpnvJ8GTx",
            ForeignBlockchain::Ethereum,
            AssetMigration::ID(String::from("gSLb9FTdGxgSLufuNzhR")),
            None,
        ).await.unwrap();

        assert_eq!(result, MigrateAssetResult {
            migration_id: Some(String::from("gSLb9FTdGxgSLufuNzhR")),
            catenis_service: Some(CatenisServiceInfo {
                status: CatenisServiceStatus::Awaiting,
                txid: None,
                error: None,
            }),
            foreign_transaction: Some(ForeignTransactionInfo {
                txid: String::from("0x883a4d9e02713b177fdd26b33e871dc765db3c964f2b1ef8e6f97eca24d718ee"),
                is_pending: true,
                success: None,
                error: None,
            }),
            status: Some(AssetMigrationStatus::Pending),
            date: Some("2021-08-03T19:11:10.350Z".into()),
            estimated_price: None,
        });
    }

    #[tokio::test]
    async fn it_asset_export_outcome() {
        // Simulate successful 'Asset Export Outcome' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "foreignTransaction": {
      "txid": "0x6299c35ccfa803ab0cb043e8d8ae4be8d7f3432d85f288ebb81e4d624e566b0a",
      "isPending": false,
      "success": true
    },
    "token": {
      "name": "Catenis test token #11",
      "symbol": "CTK11",
      "id": "0x5cE78E7204DD8f7d86142fAaA694d5354b997600"
    },
    "status": "success",
    "date": "2021-08-10T12:57:24.217Z"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/assets/aCSy24HLjKMbpnvJ8GTx/export/ethereum", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.asset_export_outcome(
            "aCSy24HLjKMbpnvJ8GTx",
            ForeignBlockchain::Ethereum,
        ).await.unwrap();

        assert_eq!(result, AssetExportOutcomeResult {
            foreign_transaction: ForeignTransactionInfo {
                txid: String::from("0x6299c35ccfa803ab0cb043e8d8ae4be8d7f3432d85f288ebb81e4d624e566b0a"),
                is_pending: false,
                success: Some(true),
                error: None,
            },
            token: ForeignTokenInfo {
                name: String::from("Catenis test token #11"),
                symbol: String::from("CTK11"),
                id: Some(String::from("0x5cE78E7204DD8f7d86142fAaA694d5354b997600")),
            },
            status: AssetExportStatus::Success,
            date: "2021-08-10T12:57:24.217Z".into(),
        });
    }

    #[tokio::test]
    async fn it_asset_migration_outcome() {
        // Simulate successful 'Asset Migration Outcome' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "direction": "outward",
    "assetId": "aCSy24HLjKMbpnvJ8GTx",
    "foreignBlockchain": "ethereum",
    "amount": 5.0,
    "catenisService": {
      "status": "fulfilled",
      "txid": "7d6a20ee009ad2bcbf5c799ee4eac594e4447bdb5007250f8ba038de97f63777"
    },
    "foreignTransaction": {
      "txid": "0x92fb47432e50b623441bb3b55dd65bf879183f87ea4913a16e75503c98792df9",
      "isPending": false,
      "success": true
    },
    "status": "success",
    "date": "2021-08-10T13:00:08.656Z"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/assets/migrations/gTQ8Qf5W6kdmdYdEEoD9", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.asset_migration_outcome(
            "gTQ8Qf5W6kdmdYdEEoD9",
        ).await.unwrap();

        assert_eq!(result, AssetMigrationOutcomeResult {
            asset_id: String::from("aCSy24HLjKMbpnvJ8GTx"),
            foreign_blockchain: ForeignBlockchain::Ethereum,
            direction: AssetMigrationDirection::Outward,
            amount: 5.0,
            catenis_service: CatenisServiceInfo {
                status: CatenisServiceStatus::Fulfilled,
                txid: Some(String::from("7d6a20ee009ad2bcbf5c799ee4eac594e4447bdb5007250f8ba038de97f63777")),
                error: None,
            },
            foreign_transaction: ForeignTransactionInfo {
                txid: String::from("0x92fb47432e50b623441bb3b55dd65bf879183f87ea4913a16e75503c98792df9"),
                is_pending: false,
                success: Some(true),
                error: None,
            },
            status: AssetMigrationStatus::Success,
            date: "2021-08-10T13:00:08.656Z".into(),
        });
    }

    #[tokio::test]
    async fn it_list_exported_assets() {
        // Simulate successful 'List Exported Assets' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "exportedAssets": [
      {
        "assetId": "aH2AkrrL55GcThhPNa3J",
        "foreignBlockchain": "ethereum",
        "foreignTransaction": {
          "txid": "0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a",
          "isPending": false,
          "success": true
        },
        "token": {
          "name": "Catenis test token #10",
          "symbol": "CTK10",
          "id": "0x537580164Ba9DB2e8C254a38E254ce15d07fDef9"
        },
        "status": "success",
        "date": "2021-08-03T18:41:27.679Z"
      },
      {
        "assetId": "aCSy24HLjKMbpnvJ8GTx",
        "foreignBlockchain": "ethereum",
        "foreignTransaction": {
          "txid": "0x6299c35ccfa803ab0cb043e8d8ae4be8d7f3432d85f288ebb81e4d624e566b0a",
          "isPending": false,
          "success": true
        },
        "token": {
          "name": "Catenis test token #11",
          "symbol": "CTK11",
          "id": "0x5cE78E7204DD8f7d86142fAaA694d5354b997600"
        },
        "status": "success",
        "date": "2021-08-10T12:57:24.217Z"
      }
    ],
    "hasMore": true
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/assets/exported?foreignBlockchain=ethereum&status=error&negateStatus=true&startDate=2021-08-01T00%3A00%3A00.000Z&endDate=2021-08-24T00%3A00%3A00.000Z&limit=2&skip=0", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.list_exported_assets(
            Some(ListExportedAssetsOptions {
                asset_id: None,
                foreign_blockchain: Some(ForeignBlockchain::Ethereum),
                token_symbol: None,
                status: Some(vec![AssetExportStatus::Error]),
                negate_status: Some(true),
                start_date: Some("2021-08-01T00:00:00Z".into()),
                end_date: Some("2021-08-24T00:00:00Z".into()),
                limit: Some(2),
                skip: Some(0),
            }),
        ).await.unwrap();

        assert_eq!(result, ListExportedAssetsResult {
            exported_assets: vec![
                AssetExportEntry {
                    asset_id: String::from("aH2AkrrL55GcThhPNa3J"),
                    foreign_blockchain: ForeignBlockchain::Ethereum,
                    foreign_transaction: ForeignTransactionInfo {
                        txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
                        is_pending: false,
                        success: Some(true),
                        error: None,
                    },
                    token: ForeignTokenInfo {
                        name: String::from("Catenis test token #10"),
                        symbol: String::from("CTK10"),
                        id: Some(String::from("0x537580164Ba9DB2e8C254a38E254ce15d07fDef9")),
                    },
                    status: AssetExportStatus::Success,
                    date: "2021-08-03T18:41:27.679Z".into(),
                },
                AssetExportEntry {
                    asset_id: String::from("aCSy24HLjKMbpnvJ8GTx"),
                    foreign_blockchain: ForeignBlockchain::Ethereum,
                    foreign_transaction: ForeignTransactionInfo {
                        txid: String::from("0x6299c35ccfa803ab0cb043e8d8ae4be8d7f3432d85f288ebb81e4d624e566b0a"),
                        is_pending: false,
                        success: Some(true),
                        error: None,
                    },
                    token: ForeignTokenInfo {
                        name: String::from("Catenis test token #11"),
                        symbol: String::from("CTK11"),
                        id: Some(String::from("0x5cE78E7204DD8f7d86142fAaA694d5354b997600")),
                    },
                    status: AssetExportStatus::Success,
                    date: "2021-08-10T12:57:24.217Z".into(),
                },
            ],
            has_more: true,
        });
    }

    #[tokio::test]
    async fn it_list_asset_migrations() {
        // Simulate successful 'List Asset Migrations' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "assetMigrations": [
      {
        "migrationId": "gq8x3efLpEXTkGQchHTb",
        "assetId": "aH2AkrrL55GcThhPNa3J",
        "foreignBlockchain": "ethereum",
        "direction": "outward",
        "amount": 10,
        "catenisService": {
          "status": "fulfilled",
          "txid": "61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf"
        },
        "foreignTransaction": {
          "txid": "0x212ab54f136a6fc1deae9ec217ef2d0417615178777131e8bb6958447fd20fe7",
          "isPending": false,
          "success": true
        },
        "status": "success",
        "date": "2021-08-03T18:51:55.591Z"
      },
      {
        "migrationId": "gTQ8Qf5W6kdmdYdEEoD9",
        "assetId": "aCSy24HLjKMbpnvJ8GTx",
        "foreignBlockchain": "ethereum",
        "direction": "outward",
        "amount": 5,
        "catenisService": {
          "status": "fulfilled",
          "txid": "7d6a20ee009ad2bcbf5c799ee4eac594e4447bdb5007250f8ba038de97f63777"
        },
        "foreignTransaction": {
          "txid": "0x92fb47432e50b623441bb3b55dd65bf879183f87ea4913a16e75503c98792df9",
          "isPending": false,
          "success": true
        },
        "status": "success",
        "date": "2021-08-10T13:00:08.656Z"
      }
    ],
    "hasMore": true
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/assets/migrations?foreignBlockchain=ethereum&direction=outward&status=interrupted%2Cerror&negateStatus=true&startDate=2021-08-01T00%3A00%3A00.000Z&endDate=2021-08-24T00%3A00%3A00.000Z&limit=2&skip=0", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.list_asset_migrations(
            Some(ListAssetMigrationsOptions {
                asset_id: None,
                foreign_blockchain: Some(ForeignBlockchain::Ethereum),
                direction: Some(AssetMigrationDirection::Outward),
                status: Some(vec![
                    AssetMigrationStatus::Interrupted,
                    AssetMigrationStatus::Error,
                ]),
                negate_status: Some(true),
                start_date: Some("2021-08-01T00:00:00Z".into()),
                end_date: Some("2021-08-24T00:00:00Z".into()),
                limit: Some(2),
                skip: Some(0),
            }),
        ).await.unwrap();

        assert_eq!(result, ListAssetMigrationsResult {
            asset_migrations: vec![
                AssetMigrationEntry {
                    migration_id: String::from("gq8x3efLpEXTkGQchHTb"),
                    asset_id: String::from("aH2AkrrL55GcThhPNa3J"),
                    foreign_blockchain: ForeignBlockchain::Ethereum,
                    direction: AssetMigrationDirection::Outward,
                    amount: 10.0,
                    catenis_service: CatenisServiceInfo {
                        status: CatenisServiceStatus::Fulfilled,
                        txid: Some(String::from("61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf")),
                        error: None,
                    },
                    foreign_transaction: ForeignTransactionInfo {
                        txid: String::from("0x212ab54f136a6fc1deae9ec217ef2d0417615178777131e8bb6958447fd20fe7"),
                        is_pending: false,
                        success: Some(true),
                        error: None,
                    },
                    status: AssetMigrationStatus::Success,
                    date: "2021-08-03T18:51:55.591Z".into(),
                },
                AssetMigrationEntry {
                    migration_id: String::from("gTQ8Qf5W6kdmdYdEEoD9"),
                    asset_id: String::from("aCSy24HLjKMbpnvJ8GTx"),
                    foreign_blockchain: ForeignBlockchain::Ethereum,
                    direction: AssetMigrationDirection::Outward,
                    amount: 5.0,
                    catenis_service: CatenisServiceInfo {
                        status: CatenisServiceStatus::Fulfilled,
                        txid: Some(String::from("7d6a20ee009ad2bcbf5c799ee4eac594e4447bdb5007250f8ba038de97f63777")),
                        error: None,
                    },
                    foreign_transaction: ForeignTransactionInfo {
                        txid: String::from("0x92fb47432e50b623441bb3b55dd65bf879183f87ea4913a16e75503c98792df9"),
                        is_pending: false,
                        success: Some(true),
                        error: None,
                    },
                    status: AssetMigrationStatus::Success,
                    date: "2021-08-10T13:00:08.656Z".into(),
                },
            ],
            has_more: true,
        });
    }

    #[tokio::test]
    async fn it_list_permission_events() {
        // Simulate successful 'List Permission Events' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "receive-notify-new-msg": "Receive notification of new message from a device",
    "receive-notify-msg-read": "Receive notification of message read by a device",
    "receive-notify-asset-of": "Receive notification of asset received for assets issued by a device",
    "receive-notify-asset-from": "Receive notification of asset received from a device",
    "receive-notify-confirm-asset-of": "Receive notification of confirmation of pending asset issued by a device",
    "receive-notify-confirm-asset-from": "Receive notification of confirmation of pending asset transferred by a device",
    "send-read-msg-confirm": "Send read message confirmation to a device",
    "receive-msg": "Receive message from a device",
    "disclose-main-props": "Disclose device's main properties (name, product unique ID) to a device",
    "disclose-identity-info": "Disclose device's basic identification information to a device",
    "receive-asset-of": "Receive an amount of an asset issued by a device",
    "receive-asset-from": "Receive an amount of an asset from a device"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/permission/events", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.list_permission_events().await.unwrap();

        assert_eq!(
            result,
            vec![
                PermissionEvent::ReceiveNotifyNewMsg,
                PermissionEvent::ReceiveNotifyMsgRead,
                PermissionEvent::ReceiveNotifyAssetOf,
                PermissionEvent::ReceiveNotifyAssetFrom,
                PermissionEvent::ReceiveNotifyConfirmAssetOf,
                PermissionEvent::ReceiveNotifyConfirmAssetFrom,
                PermissionEvent::SendReadMsgConfirm,
                PermissionEvent::ReceiveMsg,
                PermissionEvent::DiscloseMainProps,
                PermissionEvent::DiscloseIdentityInfo,
                PermissionEvent::ReceiveAssetOf,
                PermissionEvent::ReceiveAssetFrom,
            ].into_iter().zip(vec![
                String::from("Receive notification of new message from a device"),
                String::from("Receive notification of message read by a device"),
                String::from("Receive notification of asset received for assets issued by a device"),
                String::from("Receive notification of asset received from a device"),
                String::from("Receive notification of confirmation of pending asset issued by a device"),
                String::from("Receive notification of confirmation of pending asset transferred by a device"),
                String::from("Send read message confirmation to a device"),
                String::from("Receive message from a device"),
                String::from("Disclose device's main properties (name, product unique ID) to a device"),
                String::from("Disclose device's basic identification information to a device"),
                String::from("Receive an amount of an asset issued by a device"),
                String::from("Receive an amount of an asset from a device"),
            ].into_iter()).collect()
        );
    }

    #[tokio::test]
    async fn it_retrieve_permission_rights() {
        // Simulate successful 'Retrieve Permission Rights' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "system": "deny",
    "catenisNode": {
      "allow": [
        "0"
      ]
    },
    "client": {
      "allow": [
        "cEXd845DSMw9g6tM5dhy"
      ],
      "deny": [
        "c3gBoX45xk3yAmenyDRD"
      ]
    },
    "device": {
      "deny": [
        {
          "deviceId": "drc3XdxNtzoucpw9xiRp",
          "name": "TstDev1",
          "prodUniqueId": "ABC123"
        }
      ],
      "allow": [
        {
          "deviceId": "dhQMtAZxKHgfceZM5k9B"
        },
        {
          "deviceId": "daqRTyQK6hwo8dLtMnT7"
        }
      ]
    }
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/permission/events/receive-msg/rights", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_permission_rights(
            PermissionEvent::ReceiveMsg,
        ).await.unwrap();

        assert_eq!(result, RetrievePermissionRightsResult {
            system: PermissionRight::Deny,
            catenis_node: Some(PermissionRightsSetting {
                allow: Some(vec![
                    String::from("0"),
                ]),
                deny: None,
            }),
            client: Some(PermissionRightsSetting {
                allow: Some(vec![
                    String::from("cEXd845DSMw9g6tM5dhy"),
                ]),
                deny: Some(vec![
                    String::from("c3gBoX45xk3yAmenyDRD"),
                ]),
            }),
            device: Some(DevicePermissionRightsSetting {
                allow: Some(vec![
                    DeviceInfo {
                        device_id: String::from("dhQMtAZxKHgfceZM5k9B"),
                        name: None,
                        prod_unique_id: None,
                    },
                    DeviceInfo {
                        device_id: String::from("daqRTyQK6hwo8dLtMnT7"),
                        name: None,
                        prod_unique_id: None,
                    },
                ]),
                deny: Some(vec![
                    DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: Some(String::from("TstDev1")),
                        prod_unique_id: Some(String::from("ABC123")),
                    },
                ]),
            }),
        });
    }

    #[tokio::test]
    async fn it_set_permission_rights() {
        // Simulate successful 'Set Permission Rights' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "success": true
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("POST")),
            path: Some(format!("/api/{}/permission/events/receive-msg/rights", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                    String::from("content-type"),
                ].into_iter().zip(vec![
                    None,
                    None,
                    Some(HttpHeader {
                        field: String::from("content-type"),
                        value: String::from("application/json; charset=utf-8"),
                    }),
                ].into_iter()).collect()
            ),
            body: Some(String::from(r#"{"system":"deny","catenisNode":{"allow":["0"],"none":["*"]},"client":{"allow":["self"],"deny":["c3gBoX45xk3yAmenyDRD"],"none":["*"]},"device":{"allow":[{"id":"CL4_DEV001","isProdUniqueId":true},{"id":"dhQMtAZxKHgfceZM5k9B"}],"deny":[{"id":"self"}],"none":[{"id":"*"}]}}"#)),
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.set_permission_rights(
            PermissionEvent::ReceiveMsg,
            AllPermissionRightsUpdate {
                system: Some(PermissionRight::Deny),
                catenis_node: Some(PermissionRightsUpdate {
                    allow: Some(vec![
                        String::from("0"),
                    ]),
                    deny: None,
                    none: Some(vec![
                        String::from("*"),
                    ]),
                }),
                client: Some(PermissionRightsUpdate {
                    allow: Some(vec![
                        String::from("self"),
                    ]),
                    deny: Some(vec![
                        String::from("c3gBoX45xk3yAmenyDRD"),
                    ]),
                    none: Some(vec![
                        String::from("*"),
                    ]),
                }),
                device: Some(DevicePermissionRightsUpdate {
                    allow: Some(vec![
                        DeviceId {
                            id: String::from("CL4_DEV001"),
                            is_prod_unique_id: Some(true),
                        },
                        DeviceId {
                            id: String::from("dhQMtAZxKHgfceZM5k9B"),
                            is_prod_unique_id: None,
                        }
                    ]),
                    deny: Some(vec![
                        DeviceId {
                            id: String::from("self"),
                            is_prod_unique_id: None,
                        },
                    ]),
                    none: Some(vec![
                        DeviceId {
                            id: String::from("*"),
                            is_prod_unique_id: None,
                        }
                    ]),
                }),
            },
        ).await.unwrap();

        assert_eq!(result, SetPermissionRightsResult {
            success: true,
        });
    }

    #[tokio::test]
    async fn it_check_effective_permission_right() {
        // Simulate successful 'Check Effective Permission Right' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "d8YpQ7jgPBJEkBrnvp58": "allow"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/permission/events/receive-msg/rights/d8YpQ7jgPBJEkBrnvp58", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.check_effective_permission_right(
            PermissionEvent::ReceiveMsg,
            DeviceId {
                id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                is_prod_unique_id: None,
            },
        ).await.unwrap();

        assert_eq!(
            result,
            vec![
                String::from("d8YpQ7jgPBJEkBrnvp58"),
            ].into_iter().zip(vec![
                PermissionRight::Allow
            ].into_iter()).collect(),
        );
    }

    #[tokio::test]
    async fn it_retrieve_device_identification_info() {
        // Simulate successful 'Retrieve Device Identification Info' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "catenisNode": {
      "ctnNodeIndex": 0,
      "name": "Catenis Hub",
      "description": "Central Catenis node used to house clients that access the system through the Internet"
    },
    "client": {
      "clientId": "cEXd845DSMw9g6tM5dhy",
      "name": "Test Client 1"
    },
    "device": {
      "deviceId": "drc3XdxNtzoucpw9xiRp",
      "name": "TstDev1",
      "prodUniqueId": "ABC123"
    }
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/devices/self", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.retrieve_device_identification_info(
            DeviceId {
                id: String::from("self"),
                is_prod_unique_id: None,
            },
        ).await.unwrap();

        assert_eq!(result, RetrieveDeviceIdentificationInfoResult {
            catenis_node: CatenisNodeInfo {
                ctn_node_index: 0,
                name: Some(String::from("Catenis Hub")),
                description: Some(String::from("Central Catenis node used to house clients that access the system through the Internet")),
            },
            client: ClientInfo {
                client_id: String::from("cEXd845DSMw9g6tM5dhy"),
                name: Some(String::from("Test Client 1")),
            },
            device: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: Some(String::from("TstDev1")),
                prod_unique_id: Some(String::from("ABC123")),
            },
        });
    }

    #[tokio::test]
    async fn it_list_notification_events() {
        // Simulate successful 'List Notification Events' API method response

        // Start HTTP server in success simulation node
        let res_body = r#"{
  "status": "success",
  "data": {
    "new-msg-received": "A new message has been received",
    "sent-msg-read": "Previously sent message has been read by intended receiver (target device)",
    "asset-received": "An amount of an asset has been received",
    "asset-confirmed": "An amount of an asset that was pending due to an asset transfer has been confirmed",
    "final-msg-progress": "Progress of asynchronous message processing has come to an end",
    "asset-export-outcome": "Asset export has been finalized",
    "asset-migration-outcome": "Asset migration has been finalized"
  }
}"#;
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(res_body).unwrap(),
            ),
            "localhost"
        ).with_expected_request(PartialHttpRequest {
            method: Some(String::from("GET")),
            path: Some(format!("/api/{}/notification/events", DEFAULT_API_VERSION.to_string())),
            headers: Some(
                vec![
                    String::from("x-bcot-timestamp"),
                    String::from("authorization"),
                ].into_iter().zip(vec![
                    None,
                    None,
                ].into_iter()).collect()
            ),
            body: None,
        });
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        let result = ctn_client.list_notification_events().await.unwrap();

        assert_eq!(
            result,
            vec![
                NotificationEvent::NewMsgReceived,
                NotificationEvent::SentMsgRead,
                NotificationEvent::AssetReceived,
                NotificationEvent::AssetConfirmed,
                NotificationEvent::FinalMsgProgress,
                NotificationEvent::AssetExportOutcome,
                NotificationEvent::AssetMigrationOutcome,
            ].into_iter().zip(vec![
                String::from("A new message has been received"),
                String::from("Previously sent message has been read by intended receiver (target device)"),
                String::from("An amount of an asset has been received"),
                String::from("An amount of an asset that was pending due to an asset transfer has been confirmed"),
                String::from("Progress of asynchronous message processing has come to an end"),
                String::from("Asset export has been finalized"),
                String::from("Asset migration has been finalized"),
            ].into_iter()).collect()
        );
    }

    #[tokio::test]
    async fn it_send_request_success() {
        // Simulate successful 'Read Message' API method response

        // Start HTTP server in success simulation node
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(r#"{"status":"success","data":{"msgInfo":{"action":"send","from":{"deviceId":"d8YpQ7jgPBJEkBrnvp58","name":"TstDev2"}},"msgData":"Test message #1 (2020-11-30)"}}"#).unwrap(),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        // Get request used for calling 'Read Message' API method
        let req = ctn_client.get_request(
            "messages/:message_id",
            Some(&[
                ("message_id", "oNXszDazhuq4utktSMMi"),
            ]),
            Some(&[
                ("encoding", "utf8"),
            ]),
        ).unwrap();

        let result = ctn_client.send_request(req).await;

        assert!(!result.is_err(), "Returned error from sending request");

        let res = result.unwrap();

        assert!(res.status().is_success(), "Unexpected HTTP response: not success");
        assert_eq!(res.text().await.unwrap(), r#"{"status":"success","data":{"msgInfo":{"action":"send","from":{"deviceId":"d8YpQ7jgPBJEkBrnvp58","name":"TstDev2"}},"msgData":"Test message #1 (2020-11-30)"}}"#);
    }

    #[tokio::test]
    async fn it_send_request_error() {
        // Simulate error 'Read Message' API method response

        // Start HTTP server in error simulation node
        let http_server = HttpServer::new(
            HttpServerMode::Error(
                400,
                Some(HttpBody::from_json(r#"{"status":"error","message":"Invalid message ID"}"#).unwrap()),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        // Get request used for calling 'Read Message' API method
        let req = ctn_client.get_request(
            "messages/:message_id",
            Some(&[
                ("message_id", "oNXszDazhuq4utktSMMi"),
            ]),
            Some(&[
                ("encoding", "utf8"),
            ]),
        ).unwrap();

        let result = ctn_client.send_request(req).await;

        assert!(result.is_err(), "Returned success from sending request");
        assert_eq!(result.err().unwrap().to_string(), "Catenis API error: [400] - Invalid message ID");
    }

    #[tokio::test]
    async fn it_sign_and_send_request_success() {
        // Simulate successful 'Read Message' API method response

        // Start HTTP server in success simulation node
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(r#"{"status":"success","data":{"msgInfo":{"action":"send","from":{"deviceId":"d8YpQ7jgPBJEkBrnvp58","name":"TstDev2"}},"msgData":"Test message #1 (2020-11-30)"}}"#).unwrap(),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        // Get request used for calling 'Read Message' API method
        let req = ctn_client.get_request(
            "messages/:message_id",
            Some(&[
                ("message_id", "oNXszDazhuq4utktSMMi"),
            ]),
            Some(&[
                ("encoding", "utf8"),
            ]),
        ).unwrap();

        let result = ctn_client.sign_and_send_request(req).await;

        assert!(!result.is_err(), "Returned error from sending request");

        let res = result.unwrap();

        assert!(res.status().is_success(), "Unexpected HTTP response: not success");
        assert_eq!(res.text().await.unwrap(), r#"{"status":"success","data":{"msgInfo":{"action":"send","from":{"deviceId":"d8YpQ7jgPBJEkBrnvp58","name":"TstDev2"}},"msgData":"Test message #1 (2020-11-30)"}}"#);
    }

    #[tokio::test]
    async fn it_sign_and_send_request_error() {
        // Simulate error 'Read Message' API method response

        // Start HTTP server in error simulation node
        let http_server = HttpServer::new(
            HttpServerMode::Error(
                400,
                Some(HttpBody::from_json(r#"{"status":"error","message":"Invalid message ID"}"#).unwrap()),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host(&format!("localhost:{}", server_port)),
                ClientOptions::Secure(false),
            ],
        ).unwrap();

        // Get request used for calling 'Read Message' API method
        let req = ctn_client.get_request(
            "messages/:message_id",
            Some(&[
                ("message_id", "oNXszDazhuq4utktSMMi"),
            ]),
            Some(&[
                ("encoding", "utf8"),
            ]),
        ).unwrap();

        let result = ctn_client.sign_and_send_request(req).await;

        assert!(result.is_err(), "Returned success from sending request");
        assert_eq!(result.err().unwrap().to_string(), "Catenis API error: [400] - Invalid message ID");
    }

    #[test]
    fn it_assemble_get_request() {
        // Instantiate Catenis API client
        let ctn_client = CatenisClient::new(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
        ).unwrap();

        // Get request used for calling 'Read Message' API method
        let req = ctn_client.get_request(
            "messages/:message_id",
            Some(&[
                ("message_id", "oNXszDazhuq4utktSMMi"),
            ]),
            Some(&[
                ("encoding", "utf8"),
            ]),
        ).unwrap();

        assert_eq!(req.method().to_string(), "GET");
        assert_eq!(req.url().to_string(), format!("https://catenis.io/api/{}/messages/oNXszDazhuq4utktSMMi?encoding=utf8", DEFAULT_API_VERSION.to_string()));
    }

    #[tokio::test]
    async fn it_assemble_post_request() {
        // Instantiate Catenis API client
        let ctn_client = CatenisClient::new(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
        ).unwrap();

        // Get request used for calling 'Log Message' API method
        let req = ctn_client.post_request(
            "messages/log",
            String::from(r#"{"message":"Test message","options":{"encoding":"utf8"}}"#),
            None::<KVList>,
            None::<KVList>,
        ).await.unwrap();

        assert_eq!(req.method().to_string(), "POST");
        assert_eq!(req.url().to_string(), format!("https://catenis.io/api/{}/messages/log", DEFAULT_API_VERSION.to_string()));
        assert_eq!(req.headers().get(CONTENT_TYPE).unwrap(), "application/json; charset=utf-8");
        assert!(!req.headers().contains_key(CONTENT_ENCODING), "Request contains unexpected Content-Encoding HTTP header");
        assert_eq!(req.body().unwrap().as_bytes().unwrap(), b"{\"message\":\"Test message\",\"options\":{\"encoding\":\"utf8\"}}");
    }

    #[tokio::test]
    async fn it_assemble_post_request_with_compression_not_done() {
        // Instantiate Catenis API client
        let ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::UseCompression(true),
                ClientOptions::CompressThreshold(57),
            ]
        ).unwrap();

        // Get request used for calling 'Log Message' API method
        let req = ctn_client.post_request(
            "messages/log",
            String::from(r#"{"message":"Test message","options":{"encoding":"utf8"}}"#),
            None::<KVList>,
            None::<KVList>,
        ).await.unwrap();

        assert_eq!(req.method().to_string(), "POST");
        assert_eq!(req.url().to_string(), format!("https://catenis.io/api/{}/messages/log", DEFAULT_API_VERSION.to_string()));
        assert_eq!(req.headers().get(CONTENT_TYPE).unwrap(), "application/json; charset=utf-8");
        assert!(!req.headers().contains_key(CONTENT_ENCODING), "Request contains unexpected Content-Encoding HTTP header");
        assert_eq!(req.body().unwrap().as_bytes().unwrap(), b"{\"message\":\"Test message\",\"options\":{\"encoding\":\"utf8\"}}");
    }

    #[tokio::test]
    async fn it_assemble_post_request_with_compression() {
        // Instantiate Catenis API client
        let ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::UseCompression(true),
                ClientOptions::CompressThreshold(56),
            ]
        ).unwrap();

        // Get request used for calling 'Log Message' API method
        let req = ctn_client.post_request(
            "messages/log",
            String::from(r#"{"message":"Test message","options":{"encoding":"utf8"}}"#),
            None::<KVList>,
            None::<KVList>,
        ).await.unwrap();

        assert_eq!(req.method().to_string(), "POST");
        assert_eq!(req.url().to_string(), format!("https://catenis.io/api/{}/messages/log", DEFAULT_API_VERSION.to_string()));
        assert_eq!(req.headers().get(CONTENT_TYPE).unwrap(), "application/json; charset=utf-8");
        assert_eq!(req.headers().get(CONTENT_ENCODING).unwrap(), "deflate");
        assert_eq!(req.body().unwrap().as_bytes().unwrap(), b"\x78\x9c\x01\x38\x00\xc7\xff\x7b\x22\x6d\x65\x73\x73\x61\x67\x65\x22\x3a\x22\x54\x65\x73\x74\x20\x6d\x65\x73\x73\x61\x67\x65\x22\x2c\x22\x6f\x70\x74\x69\x6f\x6e\x73\x22\x3a\x7b\x22\x65\x6e\x63\x6f\x64\x69\x6e\x67\x22\x3a\x22\x75\x74\x66\x38\x22\x7d\x7d\x2e\x4c\x13\x83");
    }

    #[test]
    fn it_assemble_get_ws_request() {
        // Instantiate Catenis API client
        let ctn_client = CatenisClient::new(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
        ).unwrap();

        // Get request used to connect to WebSocket notification channel
        let req = ctn_client.get_ws_request(
            "notify/ws/:event_name",
            Some(&[
                ("event_name", "new-msg-received"),
            ]),
        ).unwrap();

        assert_eq!(req.method().to_string(), "GET");
        assert_eq!(req.url().to_string(), format!("wss://catenis.io/api/{}/notify/ws/new-msg-received", DEFAULT_API_VERSION.to_string()));
    }

    #[tokio::test]
    async fn it_sign_request() {
        // Set custom "system" time
        let _custom_time = test_helper::time::CustomTime::set(&time::date!(2020-12-01).with_time(time::time!(06:00:00)).assume_utc());

        // Instantiate Catenis API client
        let mut ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host("localhost:3000"),
                ClientOptions::Secure(false),
                ClientOptions::Version(ApiVersion(0, 10)),
            ],
        ).unwrap();

        // Get request used for calling 'Log Message' API method
        let mut req = ctn_client.post_request(
            "messages/log",
            String::from(r#"{"message":"Test message","options":{"encoding":"utf8"}}"#),
            None::<KVList>,
            None::<KVList>,
        ).await.unwrap();

        // Sign request
        ctn_client.sign_request(&mut req).unwrap();

        assert!(req.headers().contains_key(X_BCOT_TIMESTAMP), "Missing X-Bcot-Timestamp header");
        assert_eq!(req.headers().get(X_BCOT_TIMESTAMP).unwrap(), "20201201T060000Z");
        assert!(req.headers().contains_key(AUTHORIZATION), "Missing Authorization header");
        assert_eq!(req.headers().get(AUTHORIZATION).unwrap(), "CTN1-HMAC-SHA256 Credential=drc3XdxNtzoucpw9xiRp/20201201/ctn1_request,Signature=af2b41b1786b812809cc01291fd324880f48017b96332192566006d2fd7eefb4");
    }

    #[tokio::test]
    async fn it_parse_response() {
        // Simulate successful 'Read Message' API method response

        // Start HTTP server in success simulation node
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from_json(r#"{"status":"success","data":{"msgInfo":{"action":"send","from":{"deviceId":"d8YpQ7jgPBJEkBrnvp58","name":"TstDev2"}},"msgData":"Test message #1 (2020-11-30)"}}"#).unwrap(),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Send HTTP request and get response
        let res = reqwest::get(&format!("http://localhost:{}/messages/oNXszDazhuq4utktSMMi", server_port)).await.unwrap();

        // Parse response
        let result: Result<ReadMessageResponse> = CatenisClient::parse_response(res).await;

        assert!(!result.is_err(), "Error parsing Catenis API response");
        assert_eq!(result.unwrap(), ReadMessageResponse {
            status: String::from("success"),
            data: ReadMessageResult {
                msg_info: Some(MessageInfo {
                    action: RecordMessageAction::Send,
                    from: Some(DeviceInfo {
                        device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                        name: Some(String::from("TstDev2")),
                        prod_unique_id: None,
                    }),
                }),
                msg_data: Some(String::from("Test message #1 (2020-11-30)")),
                continuation_token: None,
                cached_message_id: None,
            },
        });
    }

    #[tokio::test]
    async fn it_parse_invalid_response() {
        // Simulate invalid 'Read Message' API method response

        // Start HTTP server in success simulation node
        let http_server = HttpServer::new(
            HttpServerMode::Success(
                HttpBody::from(r#"{}"#),
            ),
            "localhost"
        );
        http_server.start();

        let server_port = http_server.get_port();

        // Send HTTP request and get response
        let res = reqwest::get(&format!("http://localhost:{}/messages/oNXszDazhuq4utktSMMi", server_port)).await.unwrap();

        // Parse response
        let result: Result<ReadMessageResponse> = CatenisClient::parse_response(res).await;

        assert!(result.is_err(), "No error reported while parsing invalid Catenis API response");
        assert_eq!(result.err().unwrap().to_string(), String::from("Catenis client error: Inconsistent Catenis API response: missing field `status` at line 1 column 2"))
    }

    #[tokio::test]
    async fn it_get_new_http_client_with_no_compression() {
        // Start HTTP server in echo mode to retrieve HTTP request effectively sent
        let http_server = HttpServer::new(HttpServerMode::Echo, "localhost");
        http_server.start();

        let server_port = http_server.get_port();

        // Get HTTP client and generate an HTTP request
        let http_client = CatenisClient::new_http_client(false).unwrap();
        let req = http_client.get(&format!("http://localhost:{}/messages", server_port)).build().unwrap();

        // Send HTTP request and get the response
        let res = http_client.execute(req).await.unwrap();

        // Parse returned HTTP request from response body
        let res_body = res.text().await.unwrap();
        let http_request = HttpRequest::from_json(&res_body).unwrap();

        assert!(!http_request.headers.contains_key("accept-encoding"), "Request having unexpected Accept-Encoding HTTP header");
    }

    #[tokio::test]
    async fn it_get_new_http_client_with_compression() {
        // Start HTTP server in echo mode to retrieve HTTP request effectively sent
        let http_server = HttpServer::new(HttpServerMode::Echo, "localhost");
        http_server.start();

        let server_port = http_server.get_port();

        // Get HTTP client and generate an HTTP request
        let http_client = CatenisClient::new_http_client(true).unwrap();
        let req = http_client.get(&format!("http://localhost:{}/messages", server_port)).build().unwrap();

        // Send HTTP request and get the response
        let res = http_client.execute(req).await.unwrap();

        // Parse returned HTTP request from response body
        let res_body = res.text().await.unwrap();
        let http_request = HttpRequest::from_json(&res_body).unwrap();

        assert!(http_request.headers.contains_key("accept-encoding"), "Request missing expected Accept-Encoding HTTP header");
        assert_eq!(http_request.headers.get("accept-encoding").unwrap().value, "gzip");
    }

    #[tokio::test]
    async fn it_automatically_decompress_response() {
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

        // Get HTTP client and generate an HTTP request
        let http_client = CatenisClient::new_http_client(true).unwrap();
        let req = http_client.get(&format!("http://localhost:{}/messages", server_port)).build().unwrap();

        // Send HTTP request and get the response
        let res = http_client.execute(req).await.unwrap();

        // Read response body
        let res_body = res.bytes().await.unwrap();

        assert_eq!(res_body.as_ref(), body.as_bytes());
    }

    #[tokio::test]
    async fn it_compress_body() {
        let body = String::from("This is only a test");

        let compressed_body = CatenisClient::compress_body(body).await.unwrap();

        assert_eq!(compressed_body.as_slice(), b"\x78\x9c\x0b\xc9\xc8\x2c\x56\x00\xa2\xfc\xbc\x9c\x4a\x85\x44\x85\x92\xd4\xe2\x12\x00\x43\x81\x06\xd8");
    }
}