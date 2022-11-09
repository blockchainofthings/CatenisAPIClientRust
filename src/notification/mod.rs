use serde::{
    Deserialize,
};

use crate::{
    date_time::UtcDateTime,
    api::{
        DeviceInfo, MessageProcessError, MessageAction, MessageProcessSuccess,
        ForeignBlockchain, ForeignTokenInfo, AssetMigrationDirection, CatenisServiceInfo,
        NFAssetIssuanceProcessError, NFTokenDataManipulationProgress, NFTokenRetrievalProcessError,
    }
};

mod ws;

pub use ws::*;
use crate::api::NFAssetIssuanceResult;

/// Final status for asynchronous message processing.
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

/// Final status for asynchronous processing of non-fungible asset issuance.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFAssetIssuanceProcessProgressDone {
    /// The percentage of the total processing that has been already completed.
    pub percent_processed: u8,
    /// Indicates that the processing has finished.
    ///
    /// > **Note**: this should always to `true`.
    pub done: bool,
    /// Indicates whether the asset issuance has been successfully completed.
    pub success: bool,
    /// Processing error.
    ///
    /// > **Note**: only returned in case of error.
    pub error: Option<NFAssetIssuanceProcessError>,
    /// Date and time when processing was finalized.
    pub finish_date: UtcDateTime,
}

/// Final status for asynchronous processing of non-fungible token retrieval.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFTokenRetrievalProcessProgressDone {
    /// Number of bytes of non-fungible token data that have been retrieved.
    pub bytes_retrieved: usize,
    /// Indicates that the data retrieval has been finalized.
    ///
    /// > **Note**: this should always to `true`.
    pub done: bool,
    /// Indicates whether all the non-fungible token data has been successfully retrieved.
    pub success: bool,
    /// Processing error.
    ///
    /// > **Note**: only returned in case of error.
    pub error: Option<NFTokenRetrievalProcessError>,
    /// Date and time when the data retrieval has been finalized.
    pub finish_date: UtcDateTime,
}

/// Final status for asynchronous processing of non-fungible token transfer.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFTokenTransferProcessProgressDone {
    /// Progress of the non-fungible token data manipulation: reading and rewriting it after
    /// re-encryption (if required).
    pub data_manipulation: NFTokenDataManipulationProgress,
    /// Indicates that the non-fungible token transfer has been finalized.
    ///
    /// > **Note**: this should always to `true`.
    pub done: bool,
    ///  Indicates whether the non-fungible token has been successfully transferred.
    pub success: bool,
    /// Processing error.
    ///
    /// > **Note**: only returned in case of error.
    pub error: Option<NFTokenRetrievalProcessError>,
    /// Date and time when the non-fungible token transfer has been finalized.
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

/// *Final Non-Fungible Asset Issuance Outcome* notification data.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FinalNFAssetIssuanceOutcomeNotify {
    /// The ID of the non-fungible asset issuance.
    pub asset_issuance_id: String,
    /// The ID of the non-fungible asset for which more non-fungible tokens are being issued.
    ///
    /// > **Note**: only returned in case of re-issuance.
    pub asset_id: Option<String>,
    /// Final processing status.
    pub progress: NFAssetIssuanceProcessProgressDone,
    /// The result of the asset issuance.
    ///
    /// > **Note**: only returned if processing finished successfully.
    pub result: Option<NFAssetIssuanceResult>,
}

/// *Non-Fungible Token Received* notification data.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NonFungibleTokenReceivedNotify {
    /// List of the IDs of the non-fungible tokens that have been received.
    pub nf_token_ids: Vec<String>,
    /// Identifies the virtual device that issued the non-fungible tokens — the *issuing device*.
    pub issuer: DeviceInfo,
    /// Identifies the virtual device that sent or assigned the non-fungible tokens — the *sending
    /// device*.
    pub from: DeviceInfo,
    /// Date and time when the non-fungible tokens have been received.
    pub received_date: UtcDateTime,
}

/// *Non-Fungible Token Confirmed* notification data.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NonFungibleTokenConfirmedNotify {
    /// List of the IDs of the non-fungible tokens that have been confirmed.
    pub nf_token_ids: Vec<String>,
    /// Identifies the virtual device that issued the non-fungible tokens — the *issuing device*.
    pub issuer: DeviceInfo,
    /// Identifies the virtual device that sent or assigned the non-fungible tokens — the *sending
    /// device*.
    pub from: DeviceInfo,
    /// Date and time when the non-fungible tokens have been confirmed.
    pub confirmed_date: UtcDateTime,
}

/// *Final Non-Fungible Token Retrieval Outcome* notification data.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FinalNFTokenRetrievalOutcomeNotify {
    /// The ID of the non-fungible token being retrieved.
    pub nf_token_id: String,
    /// The ID of the non-fungible token retrieval.
    pub token_retrieval_id: String,
    /// Final processing status.
    pub progress: NFTokenRetrievalProcessProgressDone,
    /// The token that should be used to complete the retrieval of the non-fungible token.
    ///
    /// > **Note**: only returned if the processing finished successfully.
    pub continuation_token: Option<String>,
}

/// *Final Non-Fungible Token Transfer Outcome* notification data.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FinalNFTokenTransferOutcomeNotify {
    /// The ID of the non-fungible token being transferred.
    pub nf_token_id: String,
    /// The ID of the non-fungible token transfer.
    pub token_transfer_id: String,
    /// Final processing status.
    pub progress: NFTokenTransferProcessProgressDone,
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
    /// *Final Non-Fungible Asset Issuance Outcome* notification message.
    FinalNFAssetIssuanceOutcome(FinalNFAssetIssuanceOutcomeNotify),
    /// *Non-Fungible Token Received* notification message.
    NonFungibleTokenReceived(NonFungibleTokenReceivedNotify),
    /// *Non-Fungible Token Confirmed* notification message.
    NonFungibleTokenConfirmed(NonFungibleTokenConfirmedNotify),
    /// *Final Non-Fungible Token Retrieval Outcome* notification message.
    FinalNFTokenRetrievalOutcome(FinalNFTokenRetrievalOutcomeNotify),
    /// *Final Non-Fungible Token Transfer Outcome* notification message.
    FinalNFTokenTransferOutcome(FinalNFTokenTransferOutcomeNotify),
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

    mod nf_assets_tests {
        use crate::api::NFAssetIssuanceResult;
        use super::*;

        #[test]
        fn it_deserialize_nf_asset_issuance_process_progress_done_error() {
            let json = r#"{"percentProcessed":25,"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2022-11-01T16:55:21.000"}"#;

            let nf_asset_issuance_process_progress_done: NFAssetIssuanceProcessProgressDone = serde_json::from_str(json).unwrap();

            assert_eq!(nf_asset_issuance_process_progress_done, NFAssetIssuanceProcessProgressDone {
                percent_processed: 25,
                done: true,
                success: false,
                error: Some(NFAssetIssuanceProcessError {
                    code: 500,
                    message: String::from("Internal server error"),
                }),
                finish_date: "2022-11-01T16:55:21.000".into(),
            });
        }

        #[test]
        fn it_deserialize_nf_asset_issuance_process_progress_done_success() {
            let json = r#"{"percentProcessed":100,"done":true,"success":true,"finishDate":"2022-11-01T16:57:46.123"}"#;

            let nf_asset_issuance_process_progress_done: NFAssetIssuanceProcessProgressDone = serde_json::from_str(json).unwrap();

            assert_eq!(nf_asset_issuance_process_progress_done, NFAssetIssuanceProcessProgressDone {
                percent_processed: 100,
                done: true,
                success: true,
                error: None,
                finish_date: "2022-11-01T16:57:46.123".into(),
            });
        }

        #[test]
        fn it_deserialize_final_nf_asset_issuance_outcome_notify_no_opts() {
            let json = r#"{"assetIssuanceId":"iWWKqTx6svmErabyCZKM","progress":{"percentProcessed":25,"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2022-11-01T16:55:21.000"}}"#;

            let final_nf_asset_issuance_outcome_notify: FinalNFAssetIssuanceOutcomeNotify = serde_json::from_str(json).unwrap();

            assert_eq!(final_nf_asset_issuance_outcome_notify, FinalNFAssetIssuanceOutcomeNotify {
                asset_issuance_id: String::from("iWWKqTx6svmErabyCZKM"),
                asset_id: None,
                progress: NFAssetIssuanceProcessProgressDone {
                    percent_processed: 25,
                    done: true,
                    success: false,
                    error: Some(NFAssetIssuanceProcessError {
                        code: 500,
                        message: String::from("Internal server error"),
                    }),
                    finish_date: "2022-11-01T16:55:21.000".into(),
                },
                result: None,
            });
        }

        #[test]
        fn it_deserialize_final_nf_asset_issuance_outcome_notify_all_opts() {
            let json = r#"{"assetIssuanceId":"iWWKqTx6svmErabyCZKM","assetId":"ahfTzqgWAXnMR6Z57mcp","progress":{"percentProcessed":100,"done":true,"success":true,"finishDate":"2022-11-01T16:57:46.123"},"result":{"nfTokenIds":["tDGQpGy627J6uAw4grYq"]}}"#;

            let final_nf_asset_issuance_outcome_notify: FinalNFAssetIssuanceOutcomeNotify = serde_json::from_str(json).unwrap();

            assert_eq!(final_nf_asset_issuance_outcome_notify, FinalNFAssetIssuanceOutcomeNotify {
                asset_issuance_id: String::from("iWWKqTx6svmErabyCZKM"),
                asset_id: Some(String::from("ahfTzqgWAXnMR6Z57mcp")),
                progress: NFAssetIssuanceProcessProgressDone {
                    percent_processed: 100,
                    done: true,
                    success: true,
                    error: None,
                    finish_date: "2022-11-01T16:57:46.123".into(),
                },
                result: Some(NFAssetIssuanceResult {
                    asset_id: None,
                    nf_token_ids: vec![
                        String::from("tDGQpGy627J6uAw4grYq"),
                    ],
                }),
            });
        }

        #[test]
        fn it_deserialize_final_nf_asset_issuance_outcome_notification_message() {
            let json = r#"{"assetIssuanceId":"iWWKqTx6svmErabyCZKM","assetId":"ahfTzqgWAXnMR6Z57mcp","progress":{"percentProcessed":100,"done":true,"success":true,"finishDate":"2022-11-01T16:57:46.123"},"result":{"nfTokenIds":["tDGQpGy627J6uAw4grYq"]}}"#;

            let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

            assert_eq!(
                notification_message,
                NotificationMessage::FinalNFAssetIssuanceOutcome(
                    FinalNFAssetIssuanceOutcomeNotify {
                        asset_issuance_id: String::from("iWWKqTx6svmErabyCZKM"),
                        asset_id: Some(String::from("ahfTzqgWAXnMR6Z57mcp")),
                        progress: NFAssetIssuanceProcessProgressDone {
                            percent_processed: 100,
                            done: true,
                            success: true,
                            error: None,
                            finish_date: "2022-11-01T16:57:46.123".into(),
                        },
                        result: Some(NFAssetIssuanceResult {
                            asset_id: None,
                            nf_token_ids: vec![
                                String::from("tDGQpGy627J6uAw4grYq"),
                            ],
                        }),
                    }
                )
            );
        }

        #[test]
        fn it_deserialize_non_fungible_token_received_notify() {
            let json = r#"{"nfTokenIds":["tQyJrga3ke65RR23iyr2","tf2rbknDoo9wPsKBkskj"],"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"from":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"},"receivedDate":"2022-11-09T12:11:34.443"}"#;

            let non_fungible_token_received_notify: NonFungibleTokenReceivedNotify = serde_json::from_str(json).unwrap();

            assert_eq!(non_fungible_token_received_notify, NonFungibleTokenReceivedNotify {
                nf_token_ids: vec![
                    String::from("tQyJrga3ke65RR23iyr2"),
                    String::from("tf2rbknDoo9wPsKBkskj"),
                ],
                issuer: DeviceInfo {
                    device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                    name: None,
                    prod_unique_id: None,
                },
                from: DeviceInfo {
                    device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                    name: None,
                    prod_unique_id: None,
                },
                received_date: "2022-11-09T12:11:34.443".into()
            });
        }

        #[test]
        fn it_deserialize_non_fungible_token_received_notification_message() {
            let json = r#"{"nfTokenIds":["tQyJrga3ke65RR23iyr2","tf2rbknDoo9wPsKBkskj"],"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"from":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"},"receivedDate":"2022-11-09T12:11:34.443"}"#;

            let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

            assert_eq!(
                notification_message,
                NotificationMessage::NonFungibleTokenReceived(
                    NonFungibleTokenReceivedNotify {
                        nf_token_ids: vec![
                            String::from("tQyJrga3ke65RR23iyr2"),
                            String::from("tf2rbknDoo9wPsKBkskj"),
                        ],
                        issuer: DeviceInfo {
                            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                            name: None,
                            prod_unique_id: None,
                        },
                        from: DeviceInfo {
                            device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                            name: None,
                            prod_unique_id: None,
                        },
                        received_date: "2022-11-09T12:11:34.443".into()
                    }
                )
            );
        }

        #[test]
        fn it_deserialize_non_fungible_token_confirmed_notify() {
            let json = r#"{"nfTokenIds":["tQyJrga3ke65RR23iyr2","tf2rbknDoo9wPsKBkskj"],"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"from":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"},"confirmedDate":"2022-11-09T12:20:45.120"}"#;

            let non_fungible_token_confirmed_notify: NonFungibleTokenConfirmedNotify = serde_json::from_str(json).unwrap();

            assert_eq!(non_fungible_token_confirmed_notify, NonFungibleTokenConfirmedNotify {
                nf_token_ids: vec![
                    String::from("tQyJrga3ke65RR23iyr2"),
                    String::from("tf2rbknDoo9wPsKBkskj"),
                ],
                issuer: DeviceInfo {
                    device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                    name: None,
                    prod_unique_id: None,
                },
                from: DeviceInfo {
                    device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                    name: None,
                    prod_unique_id: None,
                },
                confirmed_date: "2022-11-09T12:20:45.120".into()
            });
        }

        #[test]
        fn it_deserialize_non_fungible_token_confirmed_notification_message() {
            let json = r#"{"nfTokenIds":["tQyJrga3ke65RR23iyr2","tf2rbknDoo9wPsKBkskj"],"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"from":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"},"confirmedDate":"2022-11-09T12:20:45.120"}"#;

            let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

            assert_eq!(
                notification_message,
                NotificationMessage::NonFungibleTokenConfirmed(
                    NonFungibleTokenConfirmedNotify {
                        nf_token_ids: vec![
                            String::from("tQyJrga3ke65RR23iyr2"),
                            String::from("tf2rbknDoo9wPsKBkskj"),
                        ],
                        issuer: DeviceInfo {
                            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                            name: None,
                            prod_unique_id: None,
                        },
                        from: DeviceInfo {
                            device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                            name: None,
                            prod_unique_id: None,
                        },
                        confirmed_date: "2022-11-09T12:20:45.120".into()
                    }
                )
            );
        }

        #[test]
        fn it_deserialize_nf_token_retrieval_process_progress_done_error() {
            let json = r#"{"bytesRetrieved":512,"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2022-11-05T12:01:47.483"}"#;

            let nf_token_retrieval_process_progress_done: NFTokenRetrievalProcessProgressDone = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_retrieval_process_progress_done, NFTokenRetrievalProcessProgressDone {
                bytes_retrieved: 512,
                done: true,
                success: false,
                error: Some(NFTokenRetrievalProcessError {
                    code: 500,
                    message: String::from("Internal server error"),
                }),
                finish_date: "2022-11-05T12:01:47.483".into(),
            });
        }

        #[test]
        fn it_deserialize_nf_token_retrieval_process_progress_done_success() {
            let json = r#"{"bytesRetrieved":1024,"done":true,"success":true,"finishDate":"2022-11-05T12:06:32.405"}"#;

            let nf_token_retrieval_process_progress_done: NFTokenRetrievalProcessProgressDone = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_retrieval_process_progress_done, NFTokenRetrievalProcessProgressDone {
                bytes_retrieved: 1024,
                done: true,
                success: true,
                error: None,
                finish_date: "2022-11-05T12:06:32.405".into(),
            });
        }

        #[test]
        fn it_deserialize_final_nf_token_retrieval_outcome_notify_no_opts() {
            let json = r#"{"nfTokenId":"tDGQpGy627J6uAw4grYq","tokenRetrievalId":"rGEcL2HhoarCupvbkrv9","progress":{"bytesRetrieved":512,"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2022-11-05T12:01:47.483"}}"#;

            let final_nf_token_retrieval_outcome_notify: FinalNFTokenRetrievalOutcomeNotify = serde_json::from_str(json).unwrap();

            assert_eq!(final_nf_token_retrieval_outcome_notify, FinalNFTokenRetrievalOutcomeNotify {
                nf_token_id: String::from("tDGQpGy627J6uAw4grYq"),
                token_retrieval_id: String::from("rGEcL2HhoarCupvbkrv9"),
                progress: NFTokenRetrievalProcessProgressDone {
                    bytes_retrieved: 512,
                    done: true,
                    success: false,
                    error: Some(NFTokenRetrievalProcessError {
                        code: 500,
                        message: String::from("Internal server error"),
                    }),
                    finish_date: "2022-11-05T12:01:47.483".into(),
                },
                continuation_token: None,
            });
        }

        #[test]
        fn it_deserialize_final_nf_token_retrieval_outcome_notify_all_opts() {
            let json = r#"{"nfTokenId":"tDGQpGy627J6uAw4grYq","tokenRetrievalId":"rGEcL2HhoarCupvbkrv9","progress":{"bytesRetrieved":1024,"done":true,"success":true,"finishDate":"2022-11-05T12:06:32.405"},"continuationToken":"eXxdwcoXm3dJBF7Ej759"}"#;

            let final_nf_token_retrieval_outcome_notify: FinalNFTokenRetrievalOutcomeNotify = serde_json::from_str(json).unwrap();

            assert_eq!(final_nf_token_retrieval_outcome_notify, FinalNFTokenRetrievalOutcomeNotify {
                nf_token_id: String::from("tDGQpGy627J6uAw4grYq"),
                token_retrieval_id: String::from("rGEcL2HhoarCupvbkrv9"),
                progress: NFTokenRetrievalProcessProgressDone {
                    bytes_retrieved: 1024,
                    done: true,
                    success: true,
                    error: None,
                    finish_date: "2022-11-05T12:06:32.405".into(),
                },
                continuation_token: Some(String::from("eXxdwcoXm3dJBF7Ej759")),
            });
        }

        #[test]
        fn it_deserialize_final_nf_token_retrieval_outcome_notification_message() {
            let json = r#"{"nfTokenId":"tDGQpGy627J6uAw4grYq","tokenRetrievalId":"rGEcL2HhoarCupvbkrv9","progress":{"bytesRetrieved":1024,"done":true,"success":true,"finishDate":"2022-11-05T12:06:32.405"},"continuationToken":"eXxdwcoXm3dJBF7Ej759"}"#;

            let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

            assert_eq!(
                notification_message,
                NotificationMessage::FinalNFTokenRetrievalOutcome(
                    FinalNFTokenRetrievalOutcomeNotify {
                        nf_token_id: String::from("tDGQpGy627J6uAw4grYq"),
                        token_retrieval_id: String::from("rGEcL2HhoarCupvbkrv9"),
                        progress: NFTokenRetrievalProcessProgressDone {
                            bytes_retrieved: 1024,
                            done: true,
                            success: true,
                            error: None,
                            finish_date: "2022-11-05T12:06:32.405".into(),
                        },
                        continuation_token: Some(String::from("eXxdwcoXm3dJBF7Ej759")),
                    }
                )
            );
        }

        #[test]
        fn it_deserialize_nf_token_transfer_process_progress_done_error() {
            let json = r#"{"dataManipulation":{"bytesRead":512},"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2022-11-07T09:05:52.972"}"#;

            let nf_token_transfer_process_progress_done: NFTokenTransferProcessProgressDone = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_transfer_process_progress_done, NFTokenTransferProcessProgressDone {
                data_manipulation: NFTokenDataManipulationProgress {
                    bytes_read: 512,
                    bytes_written: None
                },
                done: true,
                success: false,
                error: Some(NFTokenRetrievalProcessError {
                    code: 500,
                    message: String::from("Internal server error"),
                }),
                finish_date: "2022-11-07T09:05:52.972".into(),
            });
        }

        #[test]
        fn it_deserialize_nf_token_transfer_process_progress_done_success() {
            let json = r#"{"dataManipulation":{"bytesRead":1234,"bytesWritten":1024},"done":true,"success":true,"finishDate":"2022-11-07T10:09:57.384"}"#;

            let nf_token_transfer_process_progress_done: NFTokenTransferProcessProgressDone = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_transfer_process_progress_done, NFTokenTransferProcessProgressDone {
                data_manipulation: NFTokenDataManipulationProgress {
                    bytes_read: 1234,
                    bytes_written: Some(1024),
                },
                done: true,
                success: true,
                error: None,
                finish_date: "2022-11-07T10:09:57.384".into(),
            });
        }

        #[test]
        fn it_deserialize_final_nf_token_transfer_outcome_notify() {
            let json = r#"{"nfTokenId":"tDGQpGy627J6uAw4grYq","tokenTransferId":"xuYnPMKQSBXi28wRaZpN","progress":{"dataManipulation":{"bytesRead":1234,"bytesWritten":1024},"done":true,"success":true,"finishDate":"2022-11-07T10:09:57.384"}}"#;

            let final_nf_token_transfer_outcome_notify: FinalNFTokenTransferOutcomeNotify = serde_json::from_str(json).unwrap();

            assert_eq!(final_nf_token_transfer_outcome_notify, FinalNFTokenTransferOutcomeNotify {
                nf_token_id: String::from("tDGQpGy627J6uAw4grYq"),
                token_transfer_id: String::from("xuYnPMKQSBXi28wRaZpN"),
                progress: NFTokenTransferProcessProgressDone {
                    data_manipulation: NFTokenDataManipulationProgress {
                        bytes_read: 1234,
                        bytes_written: Some(1024),
                    },
                    done: true,
                    success: true,
                    error: None,
                    finish_date: "2022-11-07T10:09:57.384".into(),
                },
            });
        }

        #[test]
        fn it_deserialize_final_nf_token_transfer_outcome_notification_message() {
            let json = r#"{"nfTokenId":"tDGQpGy627J6uAw4grYq","tokenTransferId":"xuYnPMKQSBXi28wRaZpN","progress":{"dataManipulation":{"bytesRead":1234,"bytesWritten":1024},"done":true,"success":true,"finishDate":"2022-11-07T10:09:57.384"}}"#;

            let notification_message: NotificationMessage = serde_json::from_str(json).unwrap();

            assert_eq!(
                notification_message,
                NotificationMessage::FinalNFTokenTransferOutcome(
                    FinalNFTokenTransferOutcomeNotify {
                        nf_token_id: String::from("tDGQpGy627J6uAw4grYq"),
                        token_transfer_id: String::from("xuYnPMKQSBXi28wRaZpN"),
                        progress: NFTokenTransferProcessProgressDone {
                            data_manipulation: NFTokenDataManipulationProgress {
                                bytes_read: 1234,
                                bytes_written: Some(1024),
                            },
                            done: true,
                            success: true,
                            error: None,
                            finish_date: "2022-11-07T10:09:57.384".into(),
                        },
                    }
                )
            );
        }
    }
}