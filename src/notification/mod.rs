use serde::{
    Deserialize,
};

use crate::{
    date_time::UtcDateTime,
    api::{
        DeviceInfo, MessageProcessError, MessageAction, MessageProcessSuccess,
        ForeignBlockchain, ForeignTokenInfo, AssetMigrationDirection, CatenisServiceInfo,
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

/// Information about an already executed foreign blockchain transaction.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedForeignTransactionInfo {
    /// The ID (or hash) of the foreign blockchain transaction.
    pub txid: String,
    /// The value `false`, indicating that the foreign blockchain transaction has already been
    ///  executed.
    pub is_pending: bool,
    /// Indicates whether the foreign blockchain transaction has been successfully executed or not.
    pub success: bool,
    /// An error message describing what went wrong when executing the transaction.
    ///
    /// > **Note**: only returned if the foreign blockchain transaction's execution has failed.
    pub error: Option<String>,
}

/// Terminal asset export status.
#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TerminalAssetExportStatus {
    /// The asset export has been successfully finalized.
    Success,
    /// The asset export has failed.
    Error,
}

/// Terminal asset migration status.
#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TerminalAssetMigrationStatus {
    /// Migration started (first step completed successfully) but failed during its second step.
    ///
    /// >**Note**: this represents an inconsistent state, and migration should be retried.
    Interrupted,
    /// The asset migration has been successfully finalized.
    Success,
    /// The asset migration has failed.
    Error,
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

/// *Final Asset Export Outcome* notification data.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FinalAssetExportOutcomeNotify {
    /// The ID of the exported asset.
    pub asset_id: String,
    /// The foreign blockchain to where the asset has been exported.
    pub foreign_blockchain: ForeignBlockchain,
    /// Information about the transaction issued on the foreign blockchain to create the resulting
    ///  foreign token.
    pub foreign_transaction: ExecutedForeignTransactionInfo,
    /// Information about the resulting foreign token.
    pub token: ForeignTokenInfo,
    /// The final state of the asset export.
    pub status: TerminalAssetExportStatus,
    /// Date and time when the asset has been exported.
    pub date: UtcDateTime,
}

/// *Final Asset Migration Outcome* notification data.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FinalAssetMigrationOutcomeNotify {
    /// The ID of the asset migration.
    pub migration_id: String,
    /// The ID of the asset the amount of which has been migrated.
    pub asset_id: String,
    /// The foreign blockchain to/from where the asset amount has been migrated.
    pub foreign_blockchain: ForeignBlockchain,
    /// The direction of the migration.
    pub direction: AssetMigrationDirection,
    /// The migrated asset amount.
    pub amount: f64,
    /// Information about the execution of the migrate asset Catenis service.
    pub catenis_service: CatenisServiceInfo,
    /// Information about the transaction issued on the foreign blockchain to mint/burn the amount
    ///  of the foreign token.
    pub foreign_transaction: ExecutedForeignTransactionInfo,
    /// The final state of the asset migration.
    pub status: TerminalAssetMigrationStatus,
    /// Date and time when the asset amount has been migrated.
    pub date: UtcDateTime,
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
    /// *Final Asset Export Outcome* notification message.
    FinalAssetExportOutcome(FinalAssetExportOutcomeNotify),
    /// *Final Asset Migration Outcome* notification message.
    FinalAssetMigrationOutcome(FinalAssetMigrationOutcomeNotify),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::CatenisServiceStatus;

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
    fn it_deserialize_executed_foreign_transaction_info_no_opts() {
        let json = r#"{"txid":"0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a","isPending":false,"success":true}"#;

        let executed_foreign_transaction_info: ExecutedForeignTransactionInfo = serde_json::from_str(json).unwrap();

        assert_eq!(executed_foreign_transaction_info, ExecutedForeignTransactionInfo {
            txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
            is_pending: false,
            success: true,
            error: None
        });
    }

    #[test]
    fn it_deserialize_executed_foreign_transaction_info_all_opts() {
        let json = r#"{"txid":"0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a","isPending":false,"success":false,"error":"Simulated error"}"#;

        let executed_foreign_transaction_info: ExecutedForeignTransactionInfo = serde_json::from_str(json).unwrap();

        assert_eq!(executed_foreign_transaction_info, ExecutedForeignTransactionInfo {
            txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
            is_pending: false,
            success: false,
            error: Some(String::from("Simulated error")),
        });
    }

    #[test]
    fn it_deserialize_terminal_asset_export_status() {
        let json = r#""success""#;

        let terminal_asset_export_status: TerminalAssetExportStatus = serde_json::from_str(json).unwrap();

        assert_eq!(terminal_asset_export_status, TerminalAssetExportStatus::Success);
    }

    #[test]
    fn it_deserialize_terminal_asset_migration_status() {
        let json = r#""interrupted""#;

        let terminal_asset_migration_status: TerminalAssetMigrationStatus = serde_json::from_str(json).unwrap();

        assert_eq!(terminal_asset_migration_status, TerminalAssetMigrationStatus::Interrupted);
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
    fn it_deserialize_final_asset_export_outcome_notify() {
        let json = r#"{"assetId":"amNiawM69NjYM8QghoPD","foreignBlockchain":"ethereum","foreignTransaction":{"isPending":false,"success":true,"txid":"0x1738722379007192cac3b8e7ee3babfd1c8304133ea1a7957b93f6134ed62e48"},"token":{"name":"Catenis test token #5","symbol":"CTK5","id":"0xbAE69964D40900c6933A2CF8dD53f97c97Ab9BE7"},"status":"success","date":"2021-07-02T21:26:33.841Z"}"#;

        let final_asset_export_outcome_notify: FinalAssetExportOutcomeNotify = serde_json::from_str(json).unwrap();

        assert_eq!(final_asset_export_outcome_notify, FinalAssetExportOutcomeNotify {
            asset_id: String::from("amNiawM69NjYM8QghoPD"),
            foreign_blockchain: ForeignBlockchain::Ethereum,
            foreign_transaction: ExecutedForeignTransactionInfo {
                txid: String::from("0x1738722379007192cac3b8e7ee3babfd1c8304133ea1a7957b93f6134ed62e48"),
                is_pending: false,
                success: true,
                error: None,
            },
            token: ForeignTokenInfo {
                name: String::from("Catenis test token #5"),
                symbol: String::from("CTK5"),
                id: Some(String::from("0xbAE69964D40900c6933A2CF8dD53f97c97Ab9BE7")),
            },
            status: TerminalAssetExportStatus::Success,
            date: "2021-07-02T21:26:33.841Z".into(),
        });
    }

    #[test]
    fn it_deserialize_final_asset_migration_outcome_notify() {
        let json = r#"{"migrationId":"gbRQqs5z3ReCGfygorai","assetId":"amNiawM69NjYM8QghoPD","foreignBlockchain":"ethereum","direction":"outward","amount":50,"catenisService":{"status":"fulfilled","txid":"823941a1e02eab77d5ceecc943d7745bc49068f35d4109f7d60f9ca6fc669838"},"foreignTransaction":{"isPending":false,"success":true,"txid":"0x098d5588d03db577edfc8fc5b0094a62e22825bc9c7fc8e38430563350c75bfd"},"status":"success","date":"2021-07-03T12:51:04.771Z"}"#;
        
        let final_asset_migration_outcome_notify: FinalAssetMigrationOutcomeNotify = serde_json::from_str(json).unwrap();
        
        assert_eq!(final_asset_migration_outcome_notify, FinalAssetMigrationOutcomeNotify {
            migration_id: String::from("gbRQqs5z3ReCGfygorai"),
            asset_id: String::from("amNiawM69NjYM8QghoPD"),
            foreign_blockchain: ForeignBlockchain::Ethereum,
            direction: AssetMigrationDirection::Outward,
            amount: 50.0,
            catenis_service: CatenisServiceInfo {
                status: CatenisServiceStatus::Fulfilled,
                txid: Some(String::from("823941a1e02eab77d5ceecc943d7745bc49068f35d4109f7d60f9ca6fc669838")),
                error: None,
            },
            foreign_transaction: ExecutedForeignTransactionInfo {
                txid: String::from("0x098d5588d03db577edfc8fc5b0094a62e22825bc9c7fc8e38430563350c75bfd"),
                is_pending: false,
                success: true,
                error: None,
            },
            status: TerminalAssetMigrationStatus::Success,
            date: "2021-07-03T12:51:04.771Z".into(),
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

    #[test]
    fn it_deserialize_final_asset_export_outcome_notification_message() {
        let json = r#"{"assetId":"amNiawM69NjYM8QghoPD","foreignBlockchain":"ethereum","foreignTransaction":{"isPending":false,"success":true,"txid":"0x1738722379007192cac3b8e7ee3babfd1c8304133ea1a7957b93f6134ed62e48"},"token":{"name":"Catenis test token #5","symbol":"CTK5","id":"0xbAE69964D40900c6933A2CF8dD53f97c97Ab9BE7"},"status":"success","date":"2021-07-02T21:26:33.841Z"}"#;

        let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

        assert_eq!(
            notification_message,
            NotificationMessage::FinalAssetExportOutcome(
                FinalAssetExportOutcomeNotify {
                    asset_id: String::from("amNiawM69NjYM8QghoPD"),
                    foreign_blockchain: ForeignBlockchain::Ethereum,
                    foreign_transaction: ExecutedForeignTransactionInfo {
                        txid: String::from("0x1738722379007192cac3b8e7ee3babfd1c8304133ea1a7957b93f6134ed62e48"),
                        is_pending: false,
                        success: true,
                        error: None,
                    },
                    token: ForeignTokenInfo {
                        name: String::from("Catenis test token #5"),
                        symbol: String::from("CTK5"),
                        id: Some(String::from("0xbAE69964D40900c6933A2CF8dD53f97c97Ab9BE7")),
                    },
                    status: TerminalAssetExportStatus::Success,
                    date: "2021-07-02T21:26:33.841Z".into(),
                }
            )
        );
    }

    #[test]
    fn it_deserialize_final_asset_migration_outcome_notification_message() {
        let json = r#"{"migrationId":"gbRQqs5z3ReCGfygorai","assetId":"amNiawM69NjYM8QghoPD","foreignBlockchain":"ethereum","direction":"outward","amount":50,"catenisService":{"status":"fulfilled","txid":"823941a1e02eab77d5ceecc943d7745bc49068f35d4109f7d60f9ca6fc669838"},"foreignTransaction":{"isPending":false,"success":true,"txid":"0x098d5588d03db577edfc8fc5b0094a62e22825bc9c7fc8e38430563350c75bfd"},"status":"success","date":"2021-07-03T12:51:04.771Z"}"#;

        let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

        assert_eq!(
            notification_message,
            NotificationMessage::FinalAssetMigrationOutcome(
                FinalAssetMigrationOutcomeNotify {
                    migration_id: String::from("gbRQqs5z3ReCGfygorai"),
                    asset_id: String::from("amNiawM69NjYM8QghoPD"),
                    foreign_blockchain: ForeignBlockchain::Ethereum,
                    direction: AssetMigrationDirection::Outward,
                    amount: 50.0,
                    catenis_service: CatenisServiceInfo {
                        status: CatenisServiceStatus::Fulfilled,
                        txid: Some(String::from("823941a1e02eab77d5ceecc943d7745bc49068f35d4109f7d60f9ca6fc669838")),
                        error: None,
                    },
                    foreign_transaction: ExecutedForeignTransactionInfo {
                        txid: String::from("0x098d5588d03db577edfc8fc5b0094a62e22825bc9c7fc8e38430563350c75bfd"),
                        is_pending: false,
                        success: true,
                        error: None,
                    },
                    status: TerminalAssetMigrationStatus::Success,
                    date: "2021-07-03T12:51:04.771Z".into(),
                }
            )
        );
    }
}