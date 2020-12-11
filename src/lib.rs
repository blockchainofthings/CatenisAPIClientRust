use std::{
    fmt::{
        self,
        Display, Formatter,
    },
};

mod base_client;
mod client;
mod error;
mod date_time;
pub mod api;
pub mod notification;
#[cfg(feature = "async")]
pub mod async_impl;

use error::GenericError;

pub use client::*;
pub use error::{
    Error, Result,
};
pub use date_time::UtcDateTime;
pub use notification::WsNotifyChannel;

pub(crate) const X_BCOT_TIMESTAMP: &str = "x-bcot-timestamp";
const DEFAULT_BASE_URL: &str = "https://catenis.io/";
const API_BASE_URL_PATH: &str = "api/:version/";
const DEFAULT_API_VERSION: ApiVersion = ApiVersion(0, 10);

type KVList<'a> = &'a [(&'a str, &'a str)];

#[derive(Debug, Copy, Clone)]
pub enum Environment {
    Prod,
    Sandbox,
}

#[derive(Debug, Copy, Clone)]
pub struct ApiVersion(u16, u16);

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