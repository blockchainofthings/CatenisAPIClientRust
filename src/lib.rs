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
const DEFAULT_API_VERSION: ApiVersion = ApiVersion(0, 10);

type KVList<'a> = &'a [(&'a str, &'a str)];

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeviceCredentials {
    pub device_id: String,
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

#[derive(Debug, Copy, Clone)]
pub enum Environment {
    Prod,
    Sandbox,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ApiVersion(pub u16, pub u16);

impl Display for ApiVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ClientOptions<'a> {
    Host(&'a str),
    Environment(Environment),
    Secure(bool),
    Version(ApiVersion),
    UseCompression(bool),
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