mod client;
mod error;
pub mod notification;

pub use client::CatenisClient;
#[doc(no_inline)]
pub use notification::WsNotifyChannel;