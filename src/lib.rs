//! # Catenis API Client
//!
//! This library is used to make it easier to access the Catenis API services.
//!
//! > **Note**: this release of the library targets **version 0.11** of the Catenis API.
//!
//! ## Usage
//!
//! To use it, one needs to instantiate a new [`CatenisClient`] object. Then, to make a call to an
//! API method, just call the corresponding method on the client object.
//!
//! ### Example
//!
//! ```no_run
//! use catenis_api_client::{
//!     CatenisClient, ClientOptions, Environment, Result,
//! };
//!
//! # fn main() -> Result<()> {
//! // Instantiate Catenis API client object
//! let mut ctn_client = CatenisClient::new_with_options(
//!     Some((
//!         "drc3XdxNtzoucpw9xiRp",
//!         concat!(
//!             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
//!             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
//!         ),
//!     ).into()),
//!     &[
//!         ClientOptions::Environment(Environment::Sandbox),
//!     ],
//! )?;
//!
//! // Call Read Message API method
//! let result = ctn_client.read_message("o3muoTnnD6cXYyarYY38", None)?;
//!
//! println!("Read message result: {:?}", result);
//! # Ok(())
//! # }
//! ```
//!
//! ## Notification
//!
//! The library also makes it easy for receiving notifications from the Catenis system through its
//! [`WsNotifyChannel`] data structure, which embeds a WebSocket client.
//!
//! ## Asynchronous processing
//!
//! The library allows for asynchronous processing using the [Tokio](https://crates.io/crates/tokio/0.2.24)
//! runtime.
//!
//! > **Note**: only Tokio version 0.2 is currently supported.
//!
//! To activate asynchronous processing, the **`async`** feature must be enabled.
//!
//! ```toml
//! catenis_api_client = { version = "2.0", features = ["async"] }
//! ```
//!
//! The asynchronous version of the client can then be accessed from the [`async_impl`] module.
//!
//! ### Example
//!
//! ```no_run
//! use catenis_api_client::{
//!     async_impl,
//!     ClientOptions, Environment, Result,
//! };
//!
//! # #[tokio::main]
//! # async fn main() -> Result<()> {
//! // Instantiate asynchronous Catenis API client object
//! let mut ctn_client = async_impl::CatenisClient::new_with_options(
//!     Some((
//!         "drc3XdxNtzoucpw9xiRp",
//!         concat!(
//!             "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
//!             "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
//!         ),
//!     ).into()),
//!     &[
//!         ClientOptions::Environment(Environment::Sandbox),
//!     ],
//! )?;
//! # Ok(())
//! # }
//! ```

use std::{
    fmt::{
        self,
        Display, Formatter,
    },
};

#[macro_use]
mod macro_impl;

mod base_client;
mod client;
mod date_time;

#[cfg(test)]
mod test_helper;

pub mod api;
pub mod notification;
pub mod error;
#[cfg(feature = "async")]
pub mod async_impl;

use error::GenericError;

pub use client::*;
#[doc(no_inline)]
pub use error::{
    Error, Result,
};
pub use date_time::UtcDateTime;
#[doc(no_inline)]
pub use notification::WsNotifyChannel;

pub(crate) const X_BCOT_TIMESTAMP: &str = "x-bcot-timestamp";
const DEFAULT_BASE_URL: &str = "https://catenis.io/";
const API_BASE_URL_PATH: &str = "api/:version/";
const DEFAULT_API_VERSION: ApiVersion = ApiVersion(0, 11);

type KVList<'a> = &'a [(&'a str, &'a str)];

/// Credentials for a Catenis virtual device to access the Catenis API services.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeviceCredentials {
    /// The device ID.
    pub device_id: String,
    /// Device's API access secret.
    pub api_access_secret: String,
}

impl From<(&str, &str)> for DeviceCredentials {
    fn from(pair: (&str, &str)) -> DeviceCredentials {
        DeviceCredentials {
            device_id: String::from(pair.0),
            api_access_secret: String::from(pair.1),
        }
    }
}

/// Available Catenis API server environments.
#[derive(Debug, Copy, Clone)]
pub enum Environment {
    /// Production environment.
    Prod,
    /// Sandbox environment used for testing.
    Sandbox,
}

/// Specifies a version of the Catenis API.
///
/// # Example
///
/// ```
/// use catenis_api_client::ApiVersion;
///
/// # fn main() {
/// let api_version = ApiVersion(10, 0);
///
/// assert_eq!(api_version.to_string(), "10.0");
/// # }
///```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ApiVersion(pub u16, pub u16);

impl Display for ApiVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

/// Option settings that can be used when instantiating the Catenis API client.
#[derive(Debug, Copy, Clone)]
pub enum ClientOptions<'a> {
    /// Host name (with optional port) of target Catenis API server. Default value: **`"catenis.io"`**.
    Host(&'a str),
    /// Environment of target Catenis API server. Default value: **`Environment::Prod`**.
    Environment(Environment),
    /// Indicates whether a secure connection (HTTPS) should be used. Default value: **`true`**.
    Secure(bool),
    /// Version of Catenis API to target. Default value: **`ApiVersion(11, 0)`**.
    Version(ApiVersion),
    /// Indicates whether request body should be compressed. Default value: **`true`**.
    UseCompression(bool),
    /// Minimum size, in bytes, of request body for it to be compressed. Default value: **`1024`**.
    CompressThreshold(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_convert_device_credentials_from_pair() {
        let device_credentials = DeviceCredentials::from((
            "drc3XdxNtzoucpw9xiRp",
            "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
        ));

        assert_eq!(device_credentials, DeviceCredentials {
            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
            api_access_secret: String::from("4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"),
        });
    }

    #[test]
    fn it_convert_pair_into_device_credentials() {
        let device_credentials: DeviceCredentials = (
            "drc3XdxNtzoucpw9xiRp",
            "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
        ).into();

        assert_eq!(device_credentials, DeviceCredentials {
            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
            api_access_secret: String::from("4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"),
        });
    }

    #[test]
    fn it_format_api_version() {
        let api_version = ApiVersion(10, 0);

        assert_eq!(api_version.to_string(), "10.0");
    }
}