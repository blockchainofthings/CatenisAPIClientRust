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

/// Final status for asynchronous message processing
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessProgressDone {
    /// Total number of bytes of message that had been processed.
    pub bytes_processed: usize,
    /// Indicates that processing has finished. This should always to `true`.
    pub done: bool,
    /// Indicates whether message has been successfully processed.
    pub success: bool,
    /// Processing error. Only returned if processing finished with error.
    pub error: Option<MessageProcessError>,
    /// Date and time when processing was finalized.
    pub finish_date: UtcDateTime,
}

// Notification messages

/// *New Message Received* notification data.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewMessageReceivedNotify {
    /// The ID of the received message.
    pub message_id: String,
    /// Identifies the virtual device that sent the received message — the *origin device*.
    pub from: DeviceInfo,
    /// Date and time when the message has been received.
    pub received_date: UtcDateTime,
}

/// *Sent Message Read* notification data.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SentMessageReadNotify {
    /// The ID of the read message.
    pub message_id: String,
    /// Identifies the virtual device to which the read message had been sent — the *target device*.
    pub to: DeviceInfo,
    /// Date and time when the message has been read.
    pub read_date: UtcDateTime,
}

/// *Asset Received* notification data.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetReceivedNotify {
    /// The ID of the received asset.
    pub asset_id: String,
    /// The amount of the asset that has been received.
    pub amount: f64,
    /// Identifies the virtual device that issued the asset — the *issuing device*.
    pub issuer: DeviceInfo,
    /// Identifies the virtual device that sent or assigned the asset amount — the *sending device*.
    pub from: DeviceInfo,
    /// Date and time when the asset amount has been received.
    pub received_date: UtcDateTime,
}

/// *Asset Confirmed* notification data.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetConfirmedNotify {
    /// The ID of the confirmed asset.
    pub asset_id: String,
    /// The amount of the asset that has been confirmed.
    pub amount: f64,
    /// Identifies the virtual device that issued the asset — the *issuing device*.
    pub issuer: DeviceInfo,
    /// Identifies the virtual device that originally sent or assigned the asset amount — the
    /// *sending device*.
    pub from: DeviceInfo,
    /// Date and time when the asset amount has been confirmed.
    pub confirmed_date: UtcDateTime,
}

/// *Final Message Progress* notification data.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FinalMessageProgressNotify {
    /// The ID of the ephemeral message — either a provisional or a cached message — to which this notification refers.
    pub ephemeral_message_id: String,
    /// The action that was to be performed on the message.
    pub action: MessageAction,
    /// Final processing status.
    pub progress: MessageProcessProgressDone,
    /// Result of processing. Only returned if processing finished successfully.
    pub result: Option<MessageProcessSuccess>,
}

/// A message received through the WebSocket notification channel representing a given Catenis
/// notification event.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum NotificationMessage {
    /// *New Message Received* notification message.
    NewMessageReceived(NewMessageReceivedNotify),
    /// *Sent Message Read* notification message.
    SentMessageRead(SentMessageReadNotify),
    /// *Asset Received* notification message.
    AssetReceived(AssetReceivedNotify),
    /// *Asset Confirmed* notification message.
    AssetConfirmed(AssetConfirmedNotify),
    /// *Final Message Progress* notification message.
    FinalMessageProgress(FinalMessageProgressNotify),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_deserialize_message_process_progress_done_no_opts() {
        let json = r#"{"bytesProcessed":0,"done":true,"success":true,"finishDate":"2020-12-09T12:05:23.012Z"}"#;

        let message_process_progress_done: MessageProcessProgressDone = serde_json::from_str(json).unwrap();

        assert_eq!(message_process_progress_done, MessageProcessProgressDone {
            bytes_processed: 0,
            done: true,
            success: true,
            error: None,
            finish_date: "2020-12-09T12:05:23.012Z".into(),
        });
    }

    #[test]
    fn it_deserialize_message_process_progress_done_all_opts() {
        let json = r#"{"bytesProcessed":1024,"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2020-12-09T12:05:23.012Z"}"#;

        let message_process_progress: MessageProcessProgressDone = serde_json::from_str(json).unwrap();

        assert_eq!(message_process_progress, MessageProcessProgressDone {
            bytes_processed: 1024,
            done: true,
            success: false,
            error: Some(MessageProcessError {
                code: 500,
                message: String::from("Internal server error")
            }),
            finish_date: "2020-12-09T12:05:23.012Z".into(),
        });
    }

    #[test]
    fn it_deserialize_new_message_received_notify() {
        let json = r#"{"messageId":"mt7ZYbBYpM3zcgAf3H8X","from":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"receivedDate":"2020-12-09T12:05:23.012Z"}"#;

        let new_message_received_notify: NewMessageReceivedNotify = serde_json::from_str(json).unwrap();

        assert_eq!(new_message_received_notify, NewMessageReceivedNotify {
            message_id: String::from("mt7ZYbBYpM3zcgAf3H8X"),
            from: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
            received_date: "2020-12-09T12:05:23.012Z".into(),
        });
    }

    #[test]
    fn it_deserialize_sent_message_read_notify() {
        let json = r#"{"messageId":"mt7ZYbBYpM3zcgAf3H8X","to":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"readDate":"2020-12-09T12:05:23.012Z"}"#;

        let sent_message_read_notify: SentMessageReadNotify = serde_json::from_str(json).unwrap();

        assert_eq!(sent_message_read_notify, SentMessageReadNotify {
            message_id: String::from("mt7ZYbBYpM3zcgAf3H8X"),
            to: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
            read_date: "2020-12-09T12:05:23.012Z".into(),
        });
    }

    #[test]
    fn it_deserialize_asset_received_notify() {
        let json = r#"{"assetId":"aQjlzShmrnEZeeYBZihc","amount":54,"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"from":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"receivedDate":"2020-12-09T12:05:23.012Z"}"#;

        let asset_received_notify: AssetReceivedNotify = serde_json::from_str(json).unwrap();

        assert_eq!(asset_received_notify, AssetReceivedNotify {
            asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
            amount: 54.0,
            issuer: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
            from: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
            received_date: "2020-12-09T12:05:23.012Z".into(),
        });
    }

    #[test]
    fn it_deserialize_asset_confirmed_notify() {
        let json = r#"{"assetId":"aQjlzShmrnEZeeYBZihc","amount":54,"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"from":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"confirmedDate":"2020-12-09T12:05:23.012Z"}"#;

        let asset_confirmed_notify: AssetConfirmedNotify = serde_json::from_str(json).unwrap();

        assert_eq!(asset_confirmed_notify, AssetConfirmedNotify {
            asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
            amount: 54.0,
            issuer: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
            from: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
            confirmed_date: "2020-12-09T12:05:23.012Z".into(),
        });
    }

    #[test]
    fn it_deserialize_final_message_progress_notify_no_opts() {
        let json = r#"{"ephemeralMessageId":"pJiMtfdB94YkvRvXp7dA","action":"log","progress":{"bytesProcessed":0,"done":true,"success":true,"finishDate":"2020-12-09T12:05:23.012Z"}}"#;

        let final_message_progress_notify: FinalMessageProgressNotify = serde_json::from_str(json).unwrap();

        assert_eq!(final_message_progress_notify, FinalMessageProgressNotify {
            ephemeral_message_id: String::from("pJiMtfdB94YkvRvXp7dA"),
            action: MessageAction::Log,
            progress: MessageProcessProgressDone {
                bytes_processed: 0,
                done: true,
                success: true,
                error: None,
                finish_date: "2020-12-09T12:05:23.012Z".into(),
            },
            result: None,
        });
    }

    #[test]
    fn it_deserialize_final_message_progress_notify_all_opts() {
        let json = r#"{"ephemeralMessageId":"pJiMtfdB94YkvRvXp7dA","action":"log","progress":{"bytesProcessed":0,"done":true,"success":true,"finishDate":"2020-12-09T12:05:23.012Z"},"result":{"messageId":"mt7ZYbBYpM3zcgAf3H8X"}}"#;

        let final_message_progress_notify: FinalMessageProgressNotify = serde_json::from_str(json).unwrap();

        assert_eq!(final_message_progress_notify, FinalMessageProgressNotify {
            ephemeral_message_id: String::from("pJiMtfdB94YkvRvXp7dA"),
            action: MessageAction::Log,
            progress: MessageProcessProgressDone {
                bytes_processed: 0,
                done: true,
                success: true,
                error: None,
                finish_date: "2020-12-09T12:05:23.012Z".into(),
            },
            result: Some(MessageProcessSuccess {
                message_id: String::from("mt7ZYbBYpM3zcgAf3H8X"),
                continuation_token: None,
            }),
        });
    }

    #[test]
    fn it_deserialize_new_message_received_notification_message() {
        let json = r#"{"messageId":"mt7ZYbBYpM3zcgAf3H8X","from":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"receivedDate":"2020-12-09T12:05:23.012Z"}"#;

        let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

        assert_eq!(
            notification_message,
            NotificationMessage::NewMessageReceived(
                NewMessageReceivedNotify {
                    message_id: String::from("mt7ZYbBYpM3zcgAf3H8X"),
                    from: DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: None,
                        prod_unique_id: None,
                    },
                    received_date: "2020-12-09T12:05:23.012Z".into(),
                }
            )
        );
    }

    #[test]
    fn it_deserialize_sent_message_read_notification_message() {
        let json = r#"{"messageId":"mt7ZYbBYpM3zcgAf3H8X","to":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"readDate":"2020-12-09T12:05:23.012Z"}"#;

        let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

        assert_eq!(
            notification_message,
            NotificationMessage::SentMessageRead(
                SentMessageReadNotify {
                    message_id: String::from("mt7ZYbBYpM3zcgAf3H8X"),
                    to: DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: None,
                        prod_unique_id: None,
                    },
                    read_date: "2020-12-09T12:05:23.012Z".into(),
                }
            )
        );
    }

    #[test]
    fn it_deserialize_asset_received_notification_message() {
        let json = r#"{"assetId":"aQjlzShmrnEZeeYBZihc","amount":54,"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"from":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"receivedDate":"2020-12-09T12:05:23.012Z"}"#;

        let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

        assert_eq!(
            notification_message,
            NotificationMessage::AssetReceived(
                AssetReceivedNotify {
                    asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
                    amount: 54.0,
                    issuer: DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: None,
                        prod_unique_id: None,
                    },
                    from: DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: None,
                        prod_unique_id: None,
                    },
                    received_date: "2020-12-09T12:05:23.012Z".into(),
                }
            )
        );
    }

    #[test]
    fn it_deserialize_asset_confirmed_notification_message() {
        let json = r#"{"assetId":"aQjlzShmrnEZeeYBZihc","amount":54,"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"from":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"confirmedDate":"2020-12-09T12:05:23.012Z"}"#;

        let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

        assert_eq!(
            notification_message,
            NotificationMessage::AssetConfirmed(
                AssetConfirmedNotify {
                    asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
                    amount: 54.0,
                    issuer: DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: None,
                        prod_unique_id: None,
                    },
                    from: DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: None,
                        prod_unique_id: None,
                    },
                    confirmed_date: "2020-12-09T12:05:23.012Z".into(),
                }
            )
        );
    }

    #[test]
    fn it_deserialize_final_message_progress_notification_message() {
        let json = r#"{"ephemeralMessageId":"pJiMtfdB94YkvRvXp7dA","action":"log","progress":{"bytesProcessed":0,"done":true,"success":true,"finishDate":"2020-12-09T12:05:23.012Z"}}"#;

        let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

        assert_eq!(
            notification_message,
            NotificationMessage::FinalMessageProgress(
                FinalMessageProgressNotify {
                    ephemeral_message_id: String::from("pJiMtfdB94YkvRvXp7dA"),
                    action: MessageAction::Log,
                    progress: MessageProcessProgressDone {
                        bytes_processed: 0,
                        done: true,
                        success: true,
                        error: None,
                        finish_date: "2020-12-09T12:05:23.012Z".into(),
                    },
                    result: None,
                }
            )
        );
    }
}