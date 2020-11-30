use serde::{
    Deserialize,
};

use crate::{
    date_time::UtcDateTime,
    api::{
        DeviceInfo, MessageProcessError, MessageAction, MessageProcessSuccess,
    }
};

mod ws;

pub use ws::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessProgressDone {
    pub bytes_processed: usize,
    pub done: bool,
    pub success: bool,
    pub error: Option<MessageProcessError>,
    pub finish_date: UtcDateTime,
}

// Notification messages

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum NotificationMessage {
    NewMessageReceived(NewMessageReceivedNotify),
    SentMessageRead(SentMessageReadNotify),
    AssetReceived(AssetReceivedNotify),
    AssetConfirmed(AssetConfirmedNotify),
    FinalMessageProgress(FinalMessageProgressNotify),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMessageReceivedNotify {
    pub message_id: String,
    pub from: DeviceInfo,
    pub received_date: UtcDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentMessageReadNotify {
    pub message_id: String,
    pub to: DeviceInfo,
    pub read_date: UtcDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetReceivedNotify {
    pub asset_id: String,
    pub amount: f64,
    pub issuer: DeviceInfo,
    pub from: DeviceInfo,
    pub received_date: UtcDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetConfirmedNotify {
    pub asset_id: String,
    pub amount: f64,
    pub issuer: DeviceInfo,
    pub from: DeviceInfo,
    pub confirmed_date: UtcDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalMessageProgressNotify {
    pub ephemeral_message_id: String,
    pub action: MessageAction,
    pub progress: MessageProcessProgressDone,
    pub result: Option<MessageProcessSuccess>,
}

#[cfg(test)]
mod tests {
}