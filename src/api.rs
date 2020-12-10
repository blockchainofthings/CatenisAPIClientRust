use std::{
    fmt,
    collections::hash_map::HashMap,
    cmp::{
        PartialEq,
        Eq,
    },
    hash::Hash
};
use serde::{
    Deserialize, Serialize, Deserializer, de::{self, Visitor},
};

use crate::date_time::UtcDateTime;

pub type ListPermissionEventsResult = HashMap<PermissionEvent, String>;
pub type CheckEffectivePermissionRightResult = HashMap<String, PermissionRight>;
pub type ListNotificationEventsResult = HashMap<NotificationEvent, String>;

#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceId {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_prod_unique_id: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub device_id: String,
    pub name: Option<String>,
    pub prod_unique_id: Option<String>,
}

#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChunkedMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_final: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}

#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Encoding {
    UTF8,
    Base64,
    Hex,
}

impl ToString for Encoding {
    fn to_string(&self) -> String {
        String::from(match self {
            Encoding::UTF8 => "utf8",
            Encoding::Base64 => "base64",
            Encoding::Hex => "hex",
        })
    }
}

#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Storage {
    Auto,
    Embedded,
    External,
}

#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogMessageOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub off_chain: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<Storage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "async")]
    pub async_: Option<bool>,
}

#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub off_chain: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<Storage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_confirmation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "async")]
    pub async_: Option<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadMessageOptions {
    pub encoding: Option<Encoding>,
    pub continuation_token: Option<String>,
    pub data_chunk_size: Option<usize>,
    pub async_: Option<bool>,
}

#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecordMessageAction {
    Log,
    Send,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    pub action: RecordMessageAction,
    pub from: Option<DeviceInfo>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OffChainContainer {
    pub cid: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainContainer {
    pub txid: String,
    pub confirmed: bool,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IpfsStorage {
    pub ipfs: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceOwner {
    pub company: Option<String>,
    pub contact: Option<String>,
    pub name: Option<String>,
    pub domains: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OriginDeviceInfo {
    pub address: String,
    pub device_id: String,
    pub name: Option<String>,
    pub prod_unique_id: Option<String>,
    pub owned_by: DeviceOwner,
}

#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum TransactionType {
    #[serde(rename = "Send Message")]
    SendMessage,
    #[serde(rename = "Log Message")]
    LogMessage,
    #[serde(rename = "Settle Off-Chain Message")]
    SettleOffChainMessage,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BatchDocRef {
    pub cid: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainTransaction {
    pub txid: String,
    #[serde(rename = "type")]
    pub type_: TransactionType,
    pub batch_doc: Option<BatchDocRef>,
    pub origin_device: Option<OriginDeviceInfo>,
}

#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum OffChainMessageType {
    #[serde(rename = "Send Message")]
    SendMessage,
    #[serde(rename = "Log Message")]
    LogMessage,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OffChainMsgEnvelope {
    pub cid: String,
    #[serde(rename = "type")]
    pub type_: OffChainMessageType,
    pub origin_device: Option<OffChainOriginDeviceInfo>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProofInfo {
    pub message: String,
    pub signature: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OffChainOriginDeviceInfo {
    pub pub_key_hash: String,
    pub device_id: String,
    pub name: Option<String>,
    pub prod_unique_id: Option<String>,
    pub owned_by: DeviceOwner,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessError {
    pub code: u16,
    pub message: String
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessProgress {
    pub bytes_processed: usize,
    pub done: bool,
    pub success: Option<bool>,
    pub error: Option<MessageProcessError>,
    pub finish_date: Option<UtcDateTime>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessSuccess {
    pub message_id: String,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageAction {
    Log,
    Send,
    Read,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MessageActionOption {
    Log,
    Send,
    Any,
}

impl ToString for MessageActionOption {
    fn to_string(&self) -> String {
        String::from(match self {
            MessageActionOption::Log => "log",
            MessageActionOption::Send => "send",
            MessageActionOption::Any => "any",
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MessageDirectionOption {
    Inbound,
    Outbound,
    Any,
}

impl ToString for MessageDirectionOption {
    fn to_string(&self) -> String {
        String::from(match self {
            MessageDirectionOption::Inbound => "inbound",
            MessageDirectionOption::Outbound => "outbound",
            MessageDirectionOption::Any => "any",
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MessageReadStateOption {
    Read,
    Unread,
    Any,
}

impl ToString for MessageReadStateOption {
    fn to_string(&self) -> String {
        String::from(match self {
            MessageReadStateOption::Read => "read",
            MessageReadStateOption::Unread => "unread",
            MessageReadStateOption::Any => "any",
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ListMessagesOptions {
    pub action: Option<MessageActionOption>,
    pub direction: Option<MessageDirectionOption>,
    pub from_devices: Option<Vec<DeviceId>>,
    pub to_devices: Option<Vec<DeviceId>>,
    pub read_state: Option<MessageReadStateOption>,
    pub start_date: Option<UtcDateTime>,
    pub end_date: Option<UtcDateTime>,
    pub limit: Option<u16>,
    pub skip: Option<usize>,
}

#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageEntry {
    pub message_id: String,
    pub action: RecordMessageAction,
    pub direction: Option<MessageDirection>,
    pub from: Option<DeviceInfo>,
    pub to: Option<DeviceInfo>,
    pub read_confirmation_enabled: Option<bool>,
    pub read: Option<bool>,
    pub date: UtcDateTime,
}

#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewAssetInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub can_reissue: bool,
    pub decimal_places: u8
}

#[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetBalance {
    pub total: f64,
    pub unconfirmed: f64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OwnedAssetEntry {
    pub asset_id: String,
    pub balance: AssetBalance,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IssuedAssetEntry {
    pub asset_id: String,
    pub total_existent_balance: f64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetIssuanceEventEntry {
    pub amount: f64,
    pub holding_device: DeviceInfo,

}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetHolderEntry {
    pub holder: DeviceInfo,
    pub balance: AssetBalance,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum PermissionEvent {
    ReceiveNotifyNewMsg,
    ReceiveNotifyMsgRead,
    ReceiveNotifyAssetOf,
    ReceiveNotifyAssetFrom,
    ReceiveNotifyConfirmAssetOf,
    ReceiveNotifyConfirmAssetFrom,
    SendReadMsgConfirm,
    ReceiveMsg,
    DiscloseMainProps,
    DiscloseIdentityInfo,
    ReceiveAssetOf,
    ReceiveAssetFrom,
    UnknownEvent(String),
}

impl<'de> Deserialize<'de> for PermissionEvent {
    fn deserialize<D>(deserializer: D) -> Result<PermissionEvent, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct PermissionEventVisitor;

        impl<'de> Visitor<'de> for PermissionEventVisitor {
            type Value = PermissionEvent;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<PermissionEvent, E>
                where
                    E: de::Error,
            {
                let event: PermissionEvent = value.into();
                Ok(event)
            }
        }

        deserializer.deserialize_string(PermissionEventVisitor)
    }
}

impl ToString for PermissionEvent {
    fn to_string(&self) -> String {
        String::from(match self {
            PermissionEvent::ReceiveNotifyNewMsg => "receive-notify-new-msg",
            PermissionEvent::ReceiveNotifyMsgRead => "receive-notify-msg-read",
            PermissionEvent::ReceiveNotifyAssetOf => "receive-notify-asset-of",
            PermissionEvent::ReceiveNotifyAssetFrom => "receive-notify-asset-from",
            PermissionEvent::ReceiveNotifyConfirmAssetOf => "receive-notify-confirm-asset-of",
            PermissionEvent::ReceiveNotifyConfirmAssetFrom => "receive-notify-confirm-asset-from",
            PermissionEvent::SendReadMsgConfirm => "send-read-msg-confirm",
            PermissionEvent::ReceiveMsg => "receive-msg",
            PermissionEvent::DiscloseMainProps => "disclose-main-props",
            PermissionEvent::DiscloseIdentityInfo => "disclose-identity-info",
            PermissionEvent::ReceiveAssetOf => "receive-asset-of",
            PermissionEvent::ReceiveAssetFrom => "receive-asset-from",
            PermissionEvent::UnknownEvent(s) => s.as_str(),
        })
    }
}

impl Into<PermissionEvent> for &str {
    fn into(self) -> PermissionEvent {
        match self {
            "receive-notify-new-msg" => PermissionEvent::ReceiveNotifyNewMsg,
            "receive-notify-msg-read" => PermissionEvent::ReceiveNotifyMsgRead,
            "receive-notify-asset-of" => PermissionEvent::ReceiveNotifyAssetOf,
            "receive-notify-asset-from" => PermissionEvent::ReceiveNotifyAssetFrom,
            "receive-notify-confirm-asset-of" => PermissionEvent::ReceiveNotifyConfirmAssetOf,
            "receive-notify-confirm-asset-from" => PermissionEvent::ReceiveNotifyConfirmAssetFrom,
            "send-read-msg-confirm" => PermissionEvent::SendReadMsgConfirm,
            "receive-msg" => PermissionEvent::ReceiveMsg,
            "disclose-main-props" => PermissionEvent::DiscloseMainProps,
            "disclose-identity-info" => PermissionEvent::DiscloseIdentityInfo,
            "receive-asset-of" => PermissionEvent::ReceiveAssetOf,
            "receive-asset-from" => PermissionEvent::ReceiveAssetFrom,
            s @ _ => PermissionEvent::UnknownEvent(String::from(s)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionRight {
    Allow,
    Deny,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRightsSetting {
    pub allow: Option<Vec<String>>,
    pub deny: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DevicePermissionRightsSetting {
    pub allow: Option<Vec<DeviceInfo>>,
    pub deny: Option<Vec<DeviceInfo>>,
}

#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRightsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub none: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DevicePermissionRightsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<DeviceId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<DeviceId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub none: Option<Vec<DeviceId>>,
}

#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AllPermissionRightsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<PermissionRight>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catenis_node: Option<PermissionRightsUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<PermissionRightsUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<DevicePermissionRightsUpdate>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatenisNodeInfo {
    pub ctn_node_idx: u32,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub client_id: String,
    pub name: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum NotificationEvent {
    NewMsgReceived,
    SentMsgRead,
    AssetReceived,
    AssetConfirmed,
    FinalMsgProgress,
    UnknownEvent(String),
}

impl<'de> Deserialize<'de> for NotificationEvent {
    fn deserialize<D>(deserializer: D) -> Result<NotificationEvent, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct NotificationEventVisitor;

        impl<'de> Visitor<'de> for NotificationEventVisitor {
            type Value = NotificationEvent;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<NotificationEvent, E>
                where
                    E: de::Error,
            {
                let event: NotificationEvent = value.into();
                Ok(event)
            }
        }

        deserializer.deserialize_string(NotificationEventVisitor)
    }
}

impl ToString for NotificationEvent {
    fn to_string(&self) -> String {
        String::from(match self {
            NotificationEvent::NewMsgReceived => "new-msg-received",
            NotificationEvent::SentMsgRead => "sent-msg-read",
            NotificationEvent::AssetReceived => "asset-received",
            NotificationEvent::AssetConfirmed => "asset-confirmed",
            NotificationEvent::FinalMsgProgress => "final-msg-progress",
            NotificationEvent::UnknownEvent(s) => s.as_str(),
        })
    }
}

impl Into<NotificationEvent> for &str {
    fn into(self) -> NotificationEvent {
        match self {
            "new-msg-received" => NotificationEvent::NewMsgReceived,
            "sent-msg-read" => NotificationEvent::SentMsgRead,
            "asset-received" => NotificationEvent::AssetReceived,
            "asset-confirmed" => NotificationEvent::AssetConfirmed,
            "final-msg-progress" => NotificationEvent::FinalMsgProgress,
            s @ _ => NotificationEvent::UnknownEvent(String::from(s)),
        }
    }
}

// Result (used in response) data structures

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogMessageResult {
    pub continuation_token: Option<String>,
    pub message_id: Option<String>,
    pub provisional_message_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResult {
    pub continuation_token: Option<String>,
    pub message_id: Option<String>,
    pub provisional_message_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReadMessageResult {
    pub msg_info: Option<MessageInfo>,
    pub msg_data: Option<String>,
    pub continuation_token: Option<String>,
    pub cached_message_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageContainerResult {
    pub off_chain: Option<OffChainContainer>,
    pub blockchain: Option<BlockchainContainer>,
    pub external_storage: Option<IpfsStorage>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageOriginResult {
    pub tx: Option<BlockchainTransaction>,
    pub off_chain_msg_envelope: Option<OffChainMsgEnvelope>,
    pub proof: ProofInfo,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageProgressResult {
    pub action: MessageAction,
    pub progress: MessageProcessProgress,
    pub result: Option<MessageProcessSuccess>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListMessagesResult {
    pub messages: Vec<MessageEntry>,
    pub msg_count: u16,
    pub has_more: bool,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IssueAssetResult {
    pub asset_id: String,
}

#[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReissueAssetResult {
    pub total_existent_balance: f64,
}

#[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransferAssetResult {
    pub remaining_balance: f64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveAssetInfoResult {
    pub asset_id: String,
    pub name: String,
    pub description: String,
    pub can_reissue: bool,
    pub decimal_places: u8,
    pub issuer: DeviceInfo,
    pub total_existent_balance: f64,
}

#[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetBalanceResult {
    pub total: f64,
    pub unconfirmed: f64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListOwnedAssetsResult {
    pub owned_assets: Vec<OwnedAssetEntry>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListIssuedAssetsResult {
    pub issued_assets: Vec<IssuedAssetEntry>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveAssetIssuanceHistoryResult {
    pub issuance_events: Vec<AssetIssuanceEventEntry>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListAssetHoldersResult {
    pub asset_holders: Vec<AssetHolderEntry>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievePermissionRightsResult {
    pub system: PermissionRight,
    pub catenis_node: Option<PermissionRightsSetting>,
    pub client: Option<PermissionRightsSetting>,
    pub device: Option<DevicePermissionRightsSetting>,
}

#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetPermissionRightsResult {
    pub success: bool,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveDeviceIdentificationInfoResult {
    pub catenis_node: CatenisNodeInfo,
    pub client: ClientInfo,
    pub device: DeviceInfo,
}

// Request data structures

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogMessageRequest {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<LogMessageOptions>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogChunkedMessageRequest {
    pub message: ChunkedMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<LogMessageOptions>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SendMessageRequest {
    pub message: String,
    pub target_device: DeviceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<SendMessageOptions>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SendChunkedMessageRequest {
    pub message: ChunkedMessage,
    pub target_device: DeviceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<SendMessageOptions>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssueAssetRequest {
    pub asset_info: NewAssetInfo,
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holding_device: Option<DeviceId>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReissueAssetRequest {
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holding_device: Option<DeviceId>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferAssetRequest {
    pub amount: f64,
    pub receiving_device: DeviceId,
}

// Response data structures

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogMessageResponse {
    pub status: String,
    pub data: LogMessageResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SendMessageResponse {
    pub status: String,
    pub data: SendMessageResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReadMessageResponse {
    pub status: String,
    pub data: ReadMessageResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RetrieveMessageContainerResponse {
    pub status: String,
    pub data: RetrieveMessageContainerResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RetrieveMessageOriginResponse {
    pub status: String,
    pub data: RetrieveMessageOriginResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RetrieveMessageProgressResponse {
    pub status: String,
    pub data: RetrieveMessageProgressResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListMessagesResponse {
    pub status: String,
    pub data: ListMessagesResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssueAssetResponse {
    pub status: String,
    pub data: IssueAssetResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReissueAssetResponse {
    pub status: String,
    pub data: ReissueAssetResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferAssetResponse {
    pub status: String,
    pub data: TransferAssetResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RetrieveAssetInfoResponse {
    pub status: String,
    pub data: RetrieveAssetInfoResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetAssetBalanceResponse {
    pub status: String,
    pub data: GetAssetBalanceResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListOwnedAssetsResponse {
    pub status: String,
    pub data: ListOwnedAssetsResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListIssuedAssetsResponse {
    pub status: String,
    pub data: ListIssuedAssetsResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RetrieveAssetIssuanceHistoryResponse {
    pub status: String,
    pub data: RetrieveAssetIssuanceHistoryResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListAssetHoldersResponse {
    pub status: String,
    pub data: ListAssetHoldersResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListPermissionEventsResponse {
    pub status: String,
    pub data: ListPermissionEventsResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RetrievePermissionRightsResponse {
    pub status: String,
    pub data: RetrievePermissionRightsResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetPermissionRightsResponse {
    pub status: String,
    pub data: SetPermissionRightsResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CheckEffectivePermissionRightResponse {
    pub status: String,
    pub data: CheckEffectivePermissionRightResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RetrieveDeviceIdentificationInfoResponse {
    pub status: String,
    pub data: RetrieveDeviceIdentificationInfoResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListNotificationEventsResponse {
    pub status: String,
    pub data: ListNotificationEventsResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serialize_device_id_no_opts() {
        let device_id = DeviceId {
            id: String::from("drc3XdxNtzoucpw9xiRp"),
            is_prod_unique_id: None,
        };

        let json = serde_json::to_string(&device_id).unwrap();

        assert_eq!(json, r#"{"id":"drc3XdxNtzoucpw9xiRp"}"#);
    }

    #[test]
    fn it_serialize_device_id_all_opts() {
        let device_id = DeviceId {
            id: String::from("drc3XdxNtzoucpw9xiRp"),
            is_prod_unique_id: Some(false),
        };

        let json = serde_json::to_string(&device_id).unwrap();

        assert_eq!(json, r#"{"id":"drc3XdxNtzoucpw9xiRp","isProdUniqueId":false}"#);
    }

    #[test]
    fn it_deserialize_device_info_no_opts() {
        let json = r#"{"deviceId":"drc3XdxNtzoucpw9xiRp"}"#;

        let device_info: DeviceInfo = serde_json::from_str(json).unwrap();

        assert_eq!(device_info, DeviceInfo{
            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
            name: None,
            prod_unique_id: None,
        });
    }

    #[test]
    fn it_deserialize_device_info_all_opts() {
        let json = r#"{"deviceId":"drc3XdxNtzoucpw9xiRp","name":"Test device","prodUniqueId":"XYZ-001"}"#;

        let device_info: DeviceInfo = serde_json::from_str(json).unwrap();

        assert_eq!(device_info, DeviceInfo{
            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
            name: Some(String::from("Test device")),
            prod_unique_id: Some(String::from("XYZ-001")),
        });
    }

    #[test]
    fn it_serialize_chunked_message_no_opts() {
        let chunked_message = ChunkedMessage {
            data: None,
            is_final: None,
            continuation_token: None,
        };

        let json = serde_json::to_string(&chunked_message).unwrap();

        assert_eq!(json, r#"{}"#);
    }

    #[test]
    fn it_serialize_chunked_message_all_opts() {
        let chunked_message = ChunkedMessage {
            data: Some(String::from("Test message")),
            is_final: Some(false),
            continuation_token: Some(String::from("kjXP2CZaSdkTKCi2jDi2")),
        };

        let json = serde_json::to_string(&chunked_message).unwrap();

        assert_eq!(json, r#"{"data":"Test message","isFinal":false,"continuationToken":"kjXP2CZaSdkTKCi2jDi2"}"#);
    }

    #[test]
    fn it_serialize_encoding() {
        let encoding = Encoding::UTF8;

        let json = serde_json::to_string(&encoding).unwrap();

        assert_eq!(json, r#""utf8""#)
    }

    #[test]
    fn it_convert_encoding() {
        let encoding_str = Encoding::Base64.to_string();

        assert_eq!(encoding_str, "base64");
    }

    #[test]
    fn it_serialize_storage() {
        let encoding = Storage::Auto;

        let json = serde_json::to_string(&encoding).unwrap();

        assert_eq!(json, r#""auto""#)
    }

    #[test]
    fn it_serialize_log_message_options_no_opts() {
        let log_message_options = LogMessageOptions {
            encoding: None,
            encrypt: None,
            off_chain: None,
            storage: None,
            async_: None,
        };

        let json = serde_json::to_string(&log_message_options).unwrap();

        assert_eq!(json, r#"{}"#)
    }

    #[test]
    fn it_serialize_log_message_options_all_opts() {
        let log_message_options = LogMessageOptions {
            encoding: Some(Encoding::Hex),
            encrypt: Some(true),
            off_chain: Some(false),
            storage: Some(Storage::External),
            async_: Some(false),
        };

        let json = serde_json::to_string(&log_message_options).unwrap();

        assert_eq!(json, r#"{"encoding":"hex","encrypt":true,"offChain":false,"storage":"external","async":false}"#)
    }

    #[test]
    fn it_serialize_send_message_options_no_opts() {
        let send_message_options = SendMessageOptions {
            encoding: None,
            encrypt: None,
            off_chain: None,
            storage: None,
            read_confirmation: None,
            async_: None,
        };

        let json = serde_json::to_string(&send_message_options).unwrap();

        assert_eq!(json, r#"{}"#)
    }

    #[test]
    fn it_serialize_send_message_options_all_opts() {
        let send_message_options = SendMessageOptions {
            encoding: Some(Encoding::Hex),
            encrypt: Some(true),
            off_chain: Some(false),
            storage: Some(Storage::External),
            read_confirmation: Some(true),
            async_: Some(false),
        };

        let json = serde_json::to_string(&send_message_options).unwrap();

        assert_eq!(json, r#"{"encoding":"hex","encrypt":true,"offChain":false,"storage":"external","readConfirmation":true,"async":false}"#)
    }

    #[test]
    fn it_deserialize_record_message_action() {
        let json = r#""log""#;

        let action: RecordMessageAction = serde_json::from_str(json).unwrap();

        assert_eq!(action, RecordMessageAction::Log);
    }

    #[test]
    fn it_deserialize_message_info_no_opts() {
        let json = r#"{"action":"send"}"#;

        let message_info: MessageInfo = serde_json::from_str(json).unwrap();

        assert_eq!(message_info, MessageInfo {
            action: RecordMessageAction::Send,
            from: None,
        });
    }

    #[test]
    fn it_deserialize_message_info_all_opts() {
        let json = r#"{"action":"send","from":{"deviceId":"drc3XdxNtzoucpw9xiRp"}}"#;

        let message_info: MessageInfo = serde_json::from_str(json).unwrap();

        assert_eq!(message_info, MessageInfo {
            action: RecordMessageAction::Send,
            from: Some(DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            }),
        });
    }

    #[test]
    fn it_deserialize_off_chain_container() {
        let json = r#"{"cid":"QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"}"#;

        let off_chain_container: OffChainContainer = serde_json::from_str(json).unwrap();

        assert_eq!(off_chain_container, OffChainContainer {
            cid: String::from("QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"),
        });
    }

    #[test]
    fn it_deserialize_blockchain_container() {
        let json = r#"{"txid":"505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7","confirmed":true}"#;

        let blockchain_container: BlockchainContainer = serde_json::from_str(json).unwrap();

        assert_eq!(blockchain_container, BlockchainContainer {
            txid: String::from("505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7"),
            confirmed: true,
        });
    }

    #[test]
    fn it_deserialize_ipfs_storage() {
        let json = r#"{"ipfs":"Qmd2FBqC4dGTbiNe7HyYEJR2grpLWPzDNfBgajkDNRhK1X"}"#;

        let ipfs_storage: IpfsStorage = serde_json::from_str(json).unwrap();

        assert_eq!(ipfs_storage, IpfsStorage {
            ipfs: String::from("Qmd2FBqC4dGTbiNe7HyYEJR2grpLWPzDNfBgajkDNRhK1X"),
        });
    }

    #[test]
    fn it_deserialize_device_owner_no_opts() {
        let json = r#"{}"#;

        let device_owner: DeviceOwner = serde_json::from_str(json).unwrap();

        assert_eq!(device_owner, DeviceOwner {
            company: None,
            contact: None,
            name: None,
            domains: None,
        });
    }

    #[test]
    fn it_deserialize_device_owner_all_opts() {
        let json = r#"{"company":"Blockchain of Things","contact":"Contact One","name":"Person One","domains":["blockchainofthings.com","catenis.io"]}"#;

        let device_owner: DeviceOwner = serde_json::from_str(json).unwrap();

        assert_eq!(device_owner, DeviceOwner {
            company: Some(String::from("Blockchain of Things")),
            contact: Some(String::from("Contact One")),
            name: Some(String::from("Person One")),
            domains: Some(vec![
                String::from("blockchainofthings.com"),
                String::from("catenis.io"),
            ]),
        });
    }

    #[test]
    fn it_deserialize_origin_device_info_no_opts() {
        let json = r#"{"address":"bcrt1qww35dh64wvd2mwdm3m22lwphns6h7rp4s79p82","deviceId":"drc3XdxNtzoucpw9xiRp","ownedBy":{"name":"Person Two"}}"#;

        let origin_device_info: OriginDeviceInfo = serde_json::from_str(json).unwrap();

        assert_eq!(origin_device_info, OriginDeviceInfo {
            address: String::from("bcrt1qww35dh64wvd2mwdm3m22lwphns6h7rp4s79p82"),
            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
            name: None,
            prod_unique_id: None,
            owned_by: DeviceOwner {
                company: None,
                contact: None,
                name: Some(String::from("Person Two")),
                domains: None,
            },
        });
    }

    #[test]
    fn it_deserialize_origin_device_info_all_opts() {
        let json = r#"{"address":"bcrt1qww35dh64wvd2mwdm3m22lwphns6h7rp4s79p82","deviceId":"drc3XdxNtzoucpw9xiRp","name":"Test device","prodUniqueId":"XYZ-001","ownedBy":{"name":"Person Two"}}"#;

        let origin_device_info: OriginDeviceInfo = serde_json::from_str(json).unwrap();

        assert_eq!(origin_device_info, OriginDeviceInfo {
            address: String::from("bcrt1qww35dh64wvd2mwdm3m22lwphns6h7rp4s79p82"),
            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
            name: Some(String::from("Test device")),
            prod_unique_id: Some(String::from("XYZ-001")),
            owned_by: DeviceOwner {
                company: None,
                contact: None,
                name: Some(String::from("Person Two")),
                domains: None,
            },
        });
    }

    #[test]
    fn it_deserialize_transaction_type() {
        let json = r#""Log Message""#;

        let transaction_type: TransactionType = serde_json::from_str(json).unwrap();

        assert_eq!(transaction_type, TransactionType::LogMessage);
    }

    #[test]
    fn it_deserialize_batch_doc_ref() {
        let json = r#"{"cid":"QmPX8wJnRBmq6LLrDccnm4NeVR1TCiqAGncLschq4yvfCX"}"#;

        let batch_doc_ref: BatchDocRef = serde_json::from_str(json).unwrap();

        assert_eq!(batch_doc_ref, BatchDocRef {
            cid: String::from("QmPX8wJnRBmq6LLrDccnm4NeVR1TCiqAGncLschq4yvfCX"),
        });
    }
    
    #[test]
    fn it_deserialize_blockchain_transaction_no_opts() {
        let json = r#"{"txid":"da00834123604aa1cb6bf911b95f352e2129ee9514478a6d3ac06b04b76cf8cf","type":"Send Message"}"#;

        let blockchain_transaction: BlockchainTransaction = serde_json::from_str(json).unwrap();

        assert_eq!(blockchain_transaction, BlockchainTransaction {
            txid: String::from("da00834123604aa1cb6bf911b95f352e2129ee9514478a6d3ac06b04b76cf8cf"),
            type_: TransactionType::SendMessage,
            batch_doc: None,
            origin_device: None,
        });
    }

    #[test]
    fn it_deserialize_blockchain_transaction_all_opts() {
        let json = r#"{"txid":"da00834123604aa1cb6bf911b95f352e2129ee9514478a6d3ac06b04b76cf8cf","type":"Send Message","batchDoc":{"cid":"QmPX8wJnRBmq6LLrDccnm4NeVR1TCiqAGncLschq4yvfCX"},"originDevice":{"address":"bcrt1qww35dh64wvd2mwdm3m22lwphns6h7rp4s79p82","deviceId":"drc3XdxNtzoucpw9xiRp","name":"Test device","prodUniqueId":"XYZ-001","ownedBy":{"name":"Person Two"}}}"#;

        let blockchain_transaction: BlockchainTransaction = serde_json::from_str(json).unwrap();

        assert_eq!(blockchain_transaction, BlockchainTransaction {
            txid: String::from("da00834123604aa1cb6bf911b95f352e2129ee9514478a6d3ac06b04b76cf8cf"),
            type_: TransactionType::SendMessage,
            batch_doc: Some(BatchDocRef {
                cid: String::from("QmPX8wJnRBmq6LLrDccnm4NeVR1TCiqAGncLschq4yvfCX"),
            }),
            origin_device: Some(OriginDeviceInfo {
                address: String::from("bcrt1qww35dh64wvd2mwdm3m22lwphns6h7rp4s79p82"),
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: Some(String::from("Test device")),
                prod_unique_id: Some(String::from("XYZ-001")),
                owned_by: DeviceOwner {
                    company: None,
                    contact: None,
                    name: Some(String::from("Person Two")),
                    domains: None,
                }
            })
        });
    }

    #[test]
    fn it_deserialize_off_chain_message_type() {
        let json = r#""Log Message""#;

        let off_chain_message_type: OffChainMessageType = serde_json::from_str(json).unwrap();

        assert_eq!(off_chain_message_type, OffChainMessageType::LogMessage);
    }

    #[test]
    fn it_deserialize_off_chain_message_envelope_no_opts() {
        let json = r#"{"cid":"QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX","type":"Send Message"}"#;

        let off_chain_message_envelope: OffChainMsgEnvelope = serde_json::from_str(json).unwrap();

        assert_eq!(off_chain_message_envelope, OffChainMsgEnvelope {
            cid: String::from("QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"),
            type_: OffChainMessageType::SendMessage,
            origin_device: None,
        });
    }

    #[test]
    fn it_deserialize_off_chain_message_envelope_all_opts() {
        let json = r#"{"cid":"QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX","type":"Send Message","originDevice":{"pubKeyHash":"5e2d062321a6a3ac95f87c0c3d4a686a5456e7cd","deviceId":"drc3XdxNtzoucpw9xiRp","ownedBy":{"name":"Person Two"}}}"#;

        let off_chain_message_envelope: OffChainMsgEnvelope = serde_json::from_str(json).unwrap();

        assert_eq!(off_chain_message_envelope, OffChainMsgEnvelope {
            cid: String::from("QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"),
            type_: OffChainMessageType::SendMessage,
            origin_device: Some(OffChainOriginDeviceInfo {
                pub_key_hash: String::from("5e2d062321a6a3ac95f87c0c3d4a686a5456e7cd"),
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
                owned_by: DeviceOwner {
                    company: None,
                    contact: None,
                    name: Some(String::from("Person Two")),
                    domains: None
                }
            }),
        });
    }
    
    #[test]
    fn it_deserialize_proof_info() {
        //ProofInfo
        let json = r#"{"message":"This is only a test","signature":"IEn9KpwdIitSilCuERUz5Dg+siFe+tW7Bh4ezHDer5NBCQD1jfWJgYL2SnKnzBnGbay/WXL7eykuK8N4o4gRLNo="}"#;

        let proof_info: ProofInfo = serde_json::from_str(json).unwrap();

        assert_eq!(proof_info, ProofInfo {
            message: String::from("This is only a test"),
            signature: String::from("IEn9KpwdIitSilCuERUz5Dg+siFe+tW7Bh4ezHDer5NBCQD1jfWJgYL2SnKnzBnGbay/WXL7eykuK8N4o4gRLNo=")
        });
    }

    #[test]
    fn it_deserialize_off_chain_origin_device_info_no_opts() {
        let json = r#"{"pubKeyHash":"5e2d062321a6a3ac95f87c0c3d4a686a5456e7cd","deviceId":"drc3XdxNtzoucpw9xiRp","ownedBy":{"name":"Person Two"}}"#;

        let off_chain_origin_device_info: OffChainOriginDeviceInfo = serde_json::from_str(json).unwrap();

        assert_eq!(off_chain_origin_device_info, OffChainOriginDeviceInfo {
            pub_key_hash: String::from("5e2d062321a6a3ac95f87c0c3d4a686a5456e7cd"),
            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
            name: None,
            prod_unique_id: None,
            owned_by: DeviceOwner {
                company: None,
                contact: None,
                name: Some(String::from("Person Two")),
                domains: None,
            },
        });
    }

    #[test]
    fn it_deserialize_off_chain_origin_device_info_all_opts() {
        let json = r#"{"pubKeyHash":"5e2d062321a6a3ac95f87c0c3d4a686a5456e7cd","deviceId":"drc3XdxNtzoucpw9xiRp","name":"Test device","prodUniqueId":"XYZ-001","ownedBy":{"name":"Person Two"}}"#;

        let off_chain_origin_device_info: OffChainOriginDeviceInfo = serde_json::from_str(json).unwrap();

        assert_eq!(off_chain_origin_device_info, OffChainOriginDeviceInfo {
            pub_key_hash: String::from("5e2d062321a6a3ac95f87c0c3d4a686a5456e7cd"),
            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
            name: Some(String::from("Test device")),
            prod_unique_id: Some(String::from("XYZ-001")),
            owned_by: DeviceOwner {
                company: None,
                contact: None,
                name: Some(String::from("Person Two")),
                domains: None,
            },
        });
    }

    #[test]
    fn it_deserialize_message_process_error() {
        let json = r#"{"code":500,"message":"Internal server error"}"#;

        let message_process_error: MessageProcessError = serde_json::from_str(json).unwrap();

        assert_eq!(message_process_error, MessageProcessError {
            code: 500,
            message: String::from("Internal server error"),
        });
    }

    #[test]
    fn it_deserialize_message_process_progress_no_opts() {
        let json = r#"{"bytesProcessed":0,"done":false}"#;

        let message_process_progress: MessageProcessProgress = serde_json::from_str(json).unwrap();

        assert_eq!(message_process_progress, MessageProcessProgress {
            bytes_processed: 0,
            done: false,
            success: None,
            error: None,
            finish_date: None,
        });
    }

    #[test]
    fn it_deserialize_message_process_progress_all_opts() {
        let json = r#"{"bytesProcessed":1024,"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2020-12-09T12:05:23.012Z"}"#;

        let message_process_progress: MessageProcessProgress = serde_json::from_str(json).unwrap();

        assert_eq!(message_process_progress, MessageProcessProgress {
            bytes_processed: 1024,
            done: true,
            success: Some(false),
            error: Some(MessageProcessError {
                code: 500,
                message: String::from("Internal server error")
            }),
            finish_date: Some("2020-12-09T12:05:23.012Z".into()),
        });
    }

    #[test]
    fn it_deserialize_message_process_success_no_opts() {
        let json = r#"{"messageId":"mt7ZYbBYpM3zcgAf3H8X"}"#;

        let message_process_success: MessageProcessSuccess = serde_json::from_str(json).unwrap();

        assert_eq!(message_process_success, MessageProcessSuccess {
            message_id: String::from("mt7ZYbBYpM3zcgAf3H8X"),
            continuation_token: None,
        });
    }

    #[test]
    fn it_deserialize_message_process_success_all_opts() {
        let json = r#"{"messageId":"mt7ZYbBYpM3zcgAf3H8X","continuationToken":"kjXP2CZaSdkTKCi2jDi2"}"#;

        let message_process_success: MessageProcessSuccess = serde_json::from_str(json).unwrap();

        assert_eq!(message_process_success, MessageProcessSuccess {
            message_id: String::from("mt7ZYbBYpM3zcgAf3H8X"),
            continuation_token: Some(String::from("kjXP2CZaSdkTKCi2jDi2")),
        });
    }

    #[test]
    fn it_deserialize_message_action() {
        let json = r#""read""#;

        let message_action: MessageAction = serde_json::from_str(json).unwrap();

        assert_eq!(message_action, MessageAction::Read);
    }

    #[test]
    fn it_convert_message_action_option() {
        let message_action_option_str = MessageActionOption::Send.to_string();

        assert_eq!(message_action_option_str, "send");
    }

    #[test]
    fn it_convert_message_direction_option() {
        let message_direction_option_str = MessageDirectionOption::Inbound.to_string();

        assert_eq!(message_direction_option_str, "inbound");
    }

    #[test]
    fn it_convert_message_read_state_option() {
        let message_read_state_option_str = MessageReadStateOption::Unread.to_string();

        assert_eq!(message_read_state_option_str, "unread");
    }

    #[test]
    fn it_deserialize_message_direction() {
        let json = r#""outbound""#;

        let message_direction: MessageDirection = serde_json::from_str(json).unwrap();

        assert_eq!(message_direction, MessageDirection::Outbound);
    }
    
    #[test]
    fn it_deserialize_message_entry_no_opts() {
        let json = r#"{"messageId":"moiyHzHxFH8DsyzGjPsh","action":"log","date":"2020-10-23T10:43:21.611Z"}"#;

        let message_entry: MessageEntry = serde_json::from_str(json).unwrap();

        assert_eq!(message_entry, MessageEntry {
            message_id: String::from("moiyHzHxFH8DsyzGjPsh"),
            action: RecordMessageAction::Log,
            direction: None,
            from: None,
            to: None,
            read_confirmation_enabled: None,
            read: None,
            date: "2020-10-23T10:43:21.611Z".into(),
        });
    }

    #[test]
    fn it_deserialize_message_entry_all_opts() {
        let json = r#"{"messageId":"mWg4PS76thiFdTbA56Tg","action":"send","direction":"outbound","from":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"to":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"},"readConfirmationEnabled":true,"read":false,"date":"2020-10-22T14:11:29.692Z"}"#;

        let message_entry: MessageEntry = serde_json::from_str(json).unwrap();

        assert_eq!(message_entry, MessageEntry {
            message_id: String::from("mWg4PS76thiFdTbA56Tg"),
            action: RecordMessageAction::Send,
            direction: Some(MessageDirection::Outbound),
            from: Some(DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None
            }),
            to: Some(DeviceInfo {
                device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                name: None,
                prod_unique_id: None
            }),
            read_confirmation_enabled: Some(true),
            read: Some(false),
            date: "2020-10-22T14:11:29.692Z".into(),
        });
    }

    #[test]
    fn it_serialize_new_asset_info_no_opts() {
        let new_asset_info = NewAssetInfo {
            name: String::from("TestAsset_1"),
            description: None,
            can_reissue: false,
            decimal_places: 2,
        };

        let json = serde_json::to_string(&new_asset_info).unwrap();

        assert_eq!(json, r#"{"name":"TestAsset_1","canReissue":false,"decimalPlaces":2}"#)
    }

    #[test]
    fn it_serialize_new_asset_info_all_opts() {
        let new_asset_info = NewAssetInfo {
            name: String::from("TestAsset_1"),
            description: Some(String::from("First asset issued for test")),
            can_reissue: false,
            decimal_places: 2,
        };

        let json = serde_json::to_string(&new_asset_info).unwrap();

        assert_eq!(json, r#"{"name":"TestAsset_1","description":"First asset issued for test","canReissue":false,"decimalPlaces":2}"#)
    }

    #[test]
    fn it_deserialize_asset_balance() {
        let json = r#"{"total":123.25,"unconfirmed":0}"#;

        let asset_balance: AssetBalance = serde_json::from_str(json).unwrap();

        assert_eq!(asset_balance, AssetBalance {
            total: 123.25,
            unconfirmed: 0.0,
        });
    }

    #[test]
    fn it_deserialize_owned_asset_entry() {
        let json = r#"{"assetId":"aQjlzShmrnEZeeYBZihc","balance":{"total":123.25,"unconfirmed":0}}"#;

        let owned_asset_entry: OwnedAssetEntry = serde_json::from_str(json).unwrap();

        assert_eq!(owned_asset_entry, OwnedAssetEntry {
            asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
            balance: AssetBalance {
                total: 123.25,
                unconfirmed: 0.0,
            },
        });
    }

    #[test]
    fn it_deserialize_issued_asset_entry() {
        let json = r#"{"assetId":"aQjlzShmrnEZeeYBZihc","totalExistentBalance":123.25}"#;

        let issued_asset_entry: IssuedAssetEntry = serde_json::from_str(json).unwrap();

        assert_eq!(issued_asset_entry, IssuedAssetEntry {
            asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
            total_existent_balance: 123.25,
        });
    }

    #[test]
    fn it_deserialize_asset_issuance_event_entry() {
        let json = r#"{"amount":123,"holdingDevice":{"deviceId":"drc3XdxNtzoucpw9xiRp"}}"#;

        let asset_issuance_event_entry: AssetIssuanceEventEntry = serde_json::from_str(json).unwrap();

        assert_eq!(asset_issuance_event_entry, AssetIssuanceEventEntry {
            amount: 123.0,
            holding_device: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
        });
    }

    #[test]
    fn it_deserialize_asset_holder_entry() {
        let json = r#"{"holder":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"balance":{"total":123.25,"unconfirmed":0}}"#;

        let asset_holder_entry: AssetHolderEntry = serde_json::from_str(json).unwrap();

        assert_eq!(asset_holder_entry, AssetHolderEntry {
            holder: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
            balance: AssetBalance {
                total: 123.25,
                unconfirmed: 0.0,
            },
        });
    }

    #[test]
    fn it_deserialize_permission_event() {
        let json = r#""receive-msg""#;

        let event: PermissionEvent = serde_json::from_str(json).unwrap();

        assert_eq!(event, PermissionEvent::ReceiveMsg);
    }

    #[test]
    fn it_deserialize_unknown_permission_event() {
        let json = r#""anything""#;

        let event: PermissionEvent = serde_json::from_str(json).unwrap();

        assert_eq!(event, PermissionEvent::UnknownEvent(String::from("anything")));
    }

    #[test]
    fn it_convert_into_permission_event() {
        let permission_event: PermissionEvent = "receive-notify-new-msg".into();

        assert_eq!(permission_event, PermissionEvent::ReceiveNotifyNewMsg);
    }

    #[test]
    fn it_convert_into_unknown_permission_event() {
        let permission_event: PermissionEvent = "anything".into();

        assert_eq!(permission_event, PermissionEvent::UnknownEvent(String::from("anything")));
    }

    #[test]
    fn it_convert_from_permission_event() {
        let permission_event_str = PermissionEvent::ReceiveNotifyNewMsg.to_string();

        assert_eq!(permission_event_str, "receive-notify-new-msg");
    }

    #[test]
    fn it_convert_from_unknown_permission_event() {
        let permission_event_str = PermissionEvent::UnknownEvent(String::from("anything")).to_string();

        assert_eq!(permission_event_str, "anything");
    }

    #[test]
    fn it_serialize_permission_right() {
        let permission_right = PermissionRight::Allow;

        let json = serde_json::to_string(&permission_right).unwrap();

        assert_eq!(json, r#""allow""#);
    }

    #[test]
    fn it_deserialize_permission_right() {
        let json = r#""deny""#;

        let permission_right: PermissionRight = serde_json::from_str(json).unwrap();

        assert_eq!(permission_right, PermissionRight::Deny);
    }

    #[test]
    fn it_deserialize_permission_rights_setting_no_opts() {
        let json = r#"{}"#;

        let permission_rights_setting: PermissionRightsSetting = serde_json::from_str(json).unwrap();

        assert_eq!(permission_rights_setting, PermissionRightsSetting {
            allow: None,
            deny: None,
        });
    }

    #[test]
    fn it_deserialize_permission_rights_setting_all_opts() {
        let json = r#"{"allow":["self","cEXd845DSMw9g6tM5dhy"],"deny":["c3gBoX45xk3yAmenyDRD"]}"#;

        let permission_rights_setting: PermissionRightsSetting = serde_json::from_str(json).unwrap();

        assert_eq!(permission_rights_setting, PermissionRightsSetting {
            allow: Some(vec![
                String::from("self"),
                String::from("cEXd845DSMw9g6tM5dhy"),
            ]),
            deny: Some(vec![
                String::from("c3gBoX45xk3yAmenyDRD"),
            ]),
        });
    }

    #[test]
    fn it_deserialize_device_permission_rights_setting_no_opts() {
        let json = r#"{}"#;

        let device_permission_rights_setting: DevicePermissionRightsSetting = serde_json::from_str(json).unwrap();

        assert_eq!(device_permission_rights_setting, DevicePermissionRightsSetting {
            allow: None,
            deny: None,
        });
    }

    #[test]
    fn it_deserialize_device_permission_rights_setting_all_opts() {
        let json = r#"{"allow":[{"deviceId":"self"},{"deviceId":"drc3XdxNtzoucpw9xiRp"}],"deny":[{"deviceId":"d8YpQ7jgPBJEkBrnvp58"}]}"#;

        let device_permission_rights_setting: DevicePermissionRightsSetting = serde_json::from_str(json).unwrap();

        assert_eq!(device_permission_rights_setting, DevicePermissionRightsSetting {
            allow: Some(vec![
                DeviceInfo {
                    device_id: String::from("self"),
                    name: None,
                    prod_unique_id: None,
                },
                DeviceInfo {
                    device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                    name: None,
                    prod_unique_id: None,
                },
            ]),
            deny: Some(vec![
                DeviceInfo {
                    device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                    name: None,
                    prod_unique_id: None,
                },
            ]),
        });
    }

    #[test]
    fn it_serialize_permission_rights_update_no_opts() {
        let permission_rights_update = PermissionRightsUpdate {
            allow: None,
            deny: None,
            none: None,
        };

        let json = serde_json::to_string(&permission_rights_update).unwrap();

        assert_eq!(json, r#"{}"#);
    }

    #[test]
    fn it_serialize_permission_rights_update_all_opts() {
        let permission_rights_update = PermissionRightsUpdate {
            allow: Some(vec![
                String::from("self"),
                String::from("cEXd845DSMw9g6tM5dhy"),
            ]),
            deny: Some(vec![
                String::from("c3gBoX45xk3yAmenyDRD"),
            ]),
            none: Some(vec![
                String::from("*"),
            ]),
        };

        let json = serde_json::to_string(&permission_rights_update).unwrap();

        assert_eq!(json, r#"{"allow":["self","cEXd845DSMw9g6tM5dhy"],"deny":["c3gBoX45xk3yAmenyDRD"],"none":["*"]}"#);
    }

    #[test]
    fn it_serialize_device_permission_rights_update_no_opts() {
        let device_permission_rights_update = DevicePermissionRightsUpdate {
            allow: None,
            deny: None,
            none: None,
        };

        let json = serde_json::to_string(&device_permission_rights_update).unwrap();

        assert_eq!(json, r#"{}"#);
    }

    #[test]
    fn it_serialize_device_permission_rights_update_all_opts() {
        let device_permission_rights_update = DevicePermissionRightsUpdate {
            allow: Some(vec![
                DeviceId {
                    id: String::from("self"),
                    is_prod_unique_id: None,
                },
                DeviceId {
                    id: String::from("drc3XdxNtzoucpw9xiRp"),
                    is_prod_unique_id: None,
                },
            ]),
            deny: Some(vec![
                DeviceId {
                    id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                    is_prod_unique_id: None,
                },
            ]),
            none: Some(vec![
                DeviceId {
                    id: String::from("*"),
                    is_prod_unique_id: None,
                },
            ]),
        };

        let json = serde_json::to_string(&device_permission_rights_update).unwrap();

        assert_eq!(json, r#"{"allow":[{"id":"self"},{"id":"drc3XdxNtzoucpw9xiRp"}],"deny":[{"id":"d8YpQ7jgPBJEkBrnvp58"}],"none":[{"id":"*"}]}"#);
    }

    #[test]
    fn it_serialize_all_permission_rights_update_no_opts() {
        let all_permission_rights_update = AllPermissionRightsUpdate {
            system: None,
            catenis_node: None,
            client: None,
            device: None,
        };

        let json = serde_json::to_string(&all_permission_rights_update).unwrap();

        assert_eq!(json, r#"{}"#);
    }

    #[test]
    fn it_serialize_all_permission_rights_update_all_opts() {
        let all_permission_rights_update = AllPermissionRightsUpdate {
            system: Some(PermissionRight::Allow),
            catenis_node: Some(PermissionRightsUpdate {
                allow: Some(vec![
                    String::from("0"),
                ]),
                deny: None,
                none: None
            }),
            client: Some(PermissionRightsUpdate {
                allow: Some(vec![
                    String::from("cEXd845DSMw9g6tM5dhy"),
                ]),
                deny: None,
                none: None,
            }),
            device: Some(DevicePermissionRightsUpdate {
                allow: None,
                deny: Some(vec![
                    DeviceId {
                        id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                        is_prod_unique_id: None,
                    }
                ]),
                none: None,
            }),
        };

        let json = serde_json::to_string(&all_permission_rights_update).unwrap();

        assert_eq!(json, r#"{"system":"allow","catenisNode":{"allow":["0"]},"client":{"allow":["cEXd845DSMw9g6tM5dhy"]},"device":{"deny":[{"id":"d8YpQ7jgPBJEkBrnvp58"}]}}"#);
    }

    #[test]
    fn it_deserialize_catenis_node_info_no_opts() {
        let json = r#"{"ctnNodeIdx":0}"#;

        let catenis_node_info: CatenisNodeInfo = serde_json::from_str(json).unwrap();

        assert_eq!(catenis_node_info, CatenisNodeInfo {
            ctn_node_idx: 0,
            name: None,
            description: None,
        });
    }

    #[test]
    fn it_deserialize_catenis_node_info_all_opts() {
        let json = r#"{"ctnNodeIdx":0,"name":"Catenis Hub","description":"Central Catenis node used to house clients that access the system through the Internet"}"#;

        let catenis_node_info: CatenisNodeInfo = serde_json::from_str(json).unwrap();

        assert_eq!(catenis_node_info, CatenisNodeInfo {
            ctn_node_idx: 0,
            name: Some(String::from("Catenis Hub")),
            description: Some(String::from("Central Catenis node used to house clients that access the system through the Internet")),
        });
    }

    #[test]
    fn it_deserialize_client_info_no_opts() {
        let json = r#"{"clientId":"cEXd845DSMw9g6tM5dhy"}"#;

        let client_info: ClientInfo = serde_json::from_str(json).unwrap();

        assert_eq!(client_info, ClientInfo {
            client_id: String::from("cEXd845DSMw9g6tM5dhy"),
            name: None,
        });
    }

    #[test]
    fn it_deserialize_client_info_all_opts() {
        let json = r#"{"clientId":"cEXd845DSMw9g6tM5dhy","name":"Test Client 1"}"#;

        let client_info: ClientInfo = serde_json::from_str(json).unwrap();

        assert_eq!(client_info, ClientInfo {
            client_id: String::from("cEXd845DSMw9g6tM5dhy"),
            name: Some(String::from("Test Client 1")),
        });
    }

    #[test]
    fn it_deserialize_notification_event() {
        let json = r#""new-msg-received""#;

        let event: NotificationEvent = serde_json::from_str(json).unwrap();

        assert_eq!(event, NotificationEvent::NewMsgReceived);
    }

    #[test]
    fn it_deserialize_unknown_notification_event() {
        let json = r#""anything""#;

        let event: NotificationEvent = serde_json::from_str(json).unwrap();

        assert_eq!(event, NotificationEvent::UnknownEvent(String::from("anything")));
    }

    #[test]
    fn it_convert_into_notification_event() {
        let notification_event: NotificationEvent = "sent-msg-read".into();

        assert_eq!(notification_event, NotificationEvent::SentMsgRead);
    }

    #[test]
    fn it_convert_into_unknown_notification_event() {
        let notification_event: NotificationEvent = "anything".into();

        assert_eq!(notification_event, NotificationEvent::UnknownEvent(String::from("anything")));
    }

    #[test]
    fn it_convert_from_notification_event() {
        let notification_event_str = NotificationEvent::FinalMsgProgress.to_string();

        assert_eq!(notification_event_str, "final-msg-progress");
    }

    #[test]
    fn it_convert_from_unknown_notification_event() {
        let notification_event_str = NotificationEvent::UnknownEvent(String::from("anything")).to_string();

        assert_eq!(notification_event_str, "anything");
    }

    #[test]
    fn it_deserialize_log_message_result_no_opts() {
        let json = r#"{}"#;

        let log_message_result: LogMessageResult = serde_json::from_str(json).unwrap();

        assert_eq!(log_message_result, LogMessageResult {
            continuation_token: None,
            message_id: None,
            provisional_message_id: None,
        });
    }

    #[test]
    fn it_deserialize_log_message_result_all_opts() {
        let json = r#"{"continuationToken":"kjXP2CZaSdkTKCi2jDi2","messageId":"mWg4PS76thiFdTbA56Tg","provisionalMessageId":"pJiMtfdB94YkvRvXp7dA"}"#;

        let log_message_result: LogMessageResult = serde_json::from_str(json).unwrap();

        assert_eq!(log_message_result, LogMessageResult {
            continuation_token: Some(String::from("kjXP2CZaSdkTKCi2jDi2")),
            message_id: Some(String::from("mWg4PS76thiFdTbA56Tg")),
            provisional_message_id: Some(String::from("pJiMtfdB94YkvRvXp7dA")),
        });
    }

    #[test]
    fn it_deserialize_send_message_result_no_opts() {
        let json = r#"{}"#;

        let send_message_result: SendMessageResult = serde_json::from_str(json).unwrap();

        assert_eq!(send_message_result, SendMessageResult {
            continuation_token: None,
            message_id: None,
            provisional_message_id: None,
        });
    }

    #[test]
    fn it_deserialize_send_message_result_all_opts() {
        let json = r#"{"continuationToken":"kjXP2CZaSdkTKCi2jDi2","messageId":"mWg4PS76thiFdTbA56Tg","provisionalMessageId":"pJiMtfdB94YkvRvXp7dA"}"#;

        let send_message_result: SendMessageResult = serde_json::from_str(json).unwrap();

        assert_eq!(send_message_result, SendMessageResult {
            continuation_token: Some(String::from("kjXP2CZaSdkTKCi2jDi2")),
            message_id: Some(String::from("mWg4PS76thiFdTbA56Tg")),
            provisional_message_id: Some(String::from("pJiMtfdB94YkvRvXp7dA")),
        });
    }

    #[test]
    fn it_deserialize_read_message_result_no_opts() {
        let json = r#"{}"#;

        let read_message_result: ReadMessageResult = serde_json::from_str(json).unwrap();

        assert_eq!(read_message_result, ReadMessageResult {
            msg_info: None,
            msg_data: None,
            continuation_token: None,
            cached_message_id: None,
        });
    }

    #[test]
    fn it_deserialize_read_message_result_all_opts() {
        let json = r#"{"msgInfo":{"action":"log"},"msgData":"Test message","continuationToken":"kjXP2CZaSdkTKCi2jDi2","cachedMessageId":"hEXMdtTMzkhyJ4WssQmp"}"#;

        let read_message_result: ReadMessageResult = serde_json::from_str(json).unwrap();

        assert_eq!(read_message_result, ReadMessageResult {
            msg_info: Some(MessageInfo {
                action: RecordMessageAction::Log,
                from: None,
            }),
            msg_data: Some(String::from("Test message")),
            continuation_token: Some(String::from("kjXP2CZaSdkTKCi2jDi2")),
            cached_message_id: Some(String::from("hEXMdtTMzkhyJ4WssQmp")),
        });
    }

    #[test]
    fn it_deserialize_retrieve_message_container_result_no_opts() {
        let json = r#"{}"#;

        let retrieve_message_container_result: RetrieveMessageContainerResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_container_result, RetrieveMessageContainerResult {
            off_chain: None,
            blockchain: None,
            external_storage: None,
        });
    }

    #[test]
    fn it_deserialize_retrieve_message_container_result_all_opts() {
        let json = r#"{"offChain":{"cid":"QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"},"blockchain":{"txid":"505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7","confirmed":true},"externalStorage":{"ipfs":"Qmd2FBqC4dGTbiNe7HyYEJR2grpLWPzDNfBgajkDNRhK1X"}}"#;

        let retrieve_message_container_result: RetrieveMessageContainerResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_container_result, RetrieveMessageContainerResult {
            off_chain: Some(OffChainContainer {
                cid: String::from("QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"),
            }),
            blockchain: Some(BlockchainContainer {
                txid: String::from("505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7"),
                confirmed: true,
            }),
            external_storage: Some(IpfsStorage {
                ipfs: String::from("Qmd2FBqC4dGTbiNe7HyYEJR2grpLWPzDNfBgajkDNRhK1X"),
            }),
        });
    }

    #[test]
    fn it_deserialize_retrieve_message_origin_result_no_opts() {
        let json = r#"{"proof":{"message":"This is only a test","signature":"IEn9KpwdIitSilCuERUz5Dg+siFe+tW7Bh4ezHDer5NBCQD1jfWJgYL2SnKnzBnGbay/WXL7eykuK8N4o4gRLNo="}}"#;

        let retrieve_message_origin_result: RetrieveMessageOriginResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_origin_result, RetrieveMessageOriginResult {
            tx: None,
            off_chain_msg_envelope: None,
            proof: ProofInfo {
                message: String::from("This is only a test"),
                signature: String::from("IEn9KpwdIitSilCuERUz5Dg+siFe+tW7Bh4ezHDer5NBCQD1jfWJgYL2SnKnzBnGbay/WXL7eykuK8N4o4gRLNo=")
            },
        });
    }

    #[test]
    fn it_deserialize_retrieve_message_origin_result_all_opts() {
        let json = r#"{"tx":{"txid":"da00834123604aa1cb6bf911b95f352e2129ee9514478a6d3ac06b04b76cf8cf","type":"Send Message"},"offChainMsgEnvelope":{"cid":"QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX","type":"Send Message"},"proof":{"message":"This is only a test","signature":"IEn9KpwdIitSilCuERUz5Dg+siFe+tW7Bh4ezHDer5NBCQD1jfWJgYL2SnKnzBnGbay/WXL7eykuK8N4o4gRLNo="}}"#;

        let retrieve_message_origin_result: RetrieveMessageOriginResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_origin_result, RetrieveMessageOriginResult {
            tx: Some(BlockchainTransaction {
                txid: String::from("da00834123604aa1cb6bf911b95f352e2129ee9514478a6d3ac06b04b76cf8cf"),
                type_: TransactionType::SendMessage,
                batch_doc: None,
                origin_device: None,
            }),
            off_chain_msg_envelope: Some(OffChainMsgEnvelope {
                cid: String::from("QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"),
                type_: OffChainMessageType::SendMessage,
                origin_device: None,
            }),
            proof: ProofInfo {
                message: String::from("This is only a test"),
                signature: String::from("IEn9KpwdIitSilCuERUz5Dg+siFe+tW7Bh4ezHDer5NBCQD1jfWJgYL2SnKnzBnGbay/WXL7eykuK8N4o4gRLNo=")
            },
        });
    }

    #[test]
    fn it_deserialize_retrieve_message_progress_result_no_opts() {
        let json = r#"{"action":"log","progress":{"bytesProcessed":0,"done":false}}"#;

        let retrieve_message_progress_result: RetrieveMessageProgressResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_progress_result, RetrieveMessageProgressResult {
            action: MessageAction::Log,
            progress: MessageProcessProgress {
                bytes_processed: 0,
                done: false,
                success: None,
                error: None,
                finish_date: None,
            },
            result: None,
        });
    }

    #[test]
    fn it_deserialize_retrieve_message_progress_result_all_opts() {
        let json = r#"{"action":"log","progress":{"bytesProcessed":0,"done":false},"result":{"messageId":"mt7ZYbBYpM3zcgAf3H8X"}}"#;

        let retrieve_message_progress_result: RetrieveMessageProgressResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_progress_result, RetrieveMessageProgressResult {
            action: MessageAction::Log,
            progress: MessageProcessProgress {
                bytes_processed: 0,
                done: false,
                success: None,
                error: None,
                finish_date: None,
            },
            result: Some(MessageProcessSuccess {
                message_id: String::from("mt7ZYbBYpM3zcgAf3H8X"),
                continuation_token: None,
            }),
        });
    }

    #[test]
    fn it_deserialize_list_messages_result() {
        let json = r#"{"messages":[{"messageId":"moiyHzHxFH8DsyzGjPsh","action":"log","date":"2020-10-23T10:43:21.611Z"},{"messageId":"mWg4PS76thiFdTbA56Tg","action":"log","date":"2020-10-22T14:11:29.692Z"}],"msgCount":2,"hasMore":false}"#;

        let list_messages_result: ListMessagesResult = serde_json::from_str(json).unwrap();

        assert_eq!(list_messages_result, ListMessagesResult {
            messages: vec![
                MessageEntry {
                    message_id: String::from("moiyHzHxFH8DsyzGjPsh"),
                    action: RecordMessageAction::Log,
                    direction: None,
                    from: None,
                    to: None,
                    read_confirmation_enabled: None,
                    read: None,
                    date: "2020-10-23T10:43:21.611Z".into(),
                },
                MessageEntry {
                    message_id: String::from("mWg4PS76thiFdTbA56Tg"),
                    action: RecordMessageAction::Log,
                    direction: None,
                    from: None,
                    to: None,
                    read_confirmation_enabled: None,
                    read: None,
                    date: "2020-10-22T14:11:29.692Z".into(),
                },
            ],
            msg_count: 2,
            has_more: false
        });
    }

    #[test]
    fn it_deserialize_issue_asset_result() {
        let json = r#"{"assetId":"aQjlzShmrnEZeeYBZihc"}"#;

        let issue_asset_result: IssueAssetResult = serde_json::from_str(json).unwrap();

        assert_eq!(issue_asset_result, IssueAssetResult {
            asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
        });
    }

    #[test]
    fn it_deserialize_reissue_asset_result() {
        let json = r#"{"totalExistentBalance":123.25}"#;

        let reissue_asset_result: ReissueAssetResult = serde_json::from_str(json).unwrap();

        assert_eq!(reissue_asset_result, ReissueAssetResult {
            total_existent_balance: 123.25
        });
    }

    #[test]
    fn it_deserialize_transfer_asset_result() {
        let json = r#"{"remainingBalance":54}"#;

        let transfer_asset_result: TransferAssetResult = serde_json::from_str(json).unwrap();

        assert_eq!(transfer_asset_result, TransferAssetResult {
            remaining_balance: 54.0
        });
    }
    
    #[test]
    fn it_deserialize_retrieve_asset_info_result() {
        let json = r#"{"assetId":"aQjlzShmrnEZeeYBZihc","name":"TestAsset_1","description":"First asset issued for test","canReissue":false,"decimalPlaces":2,"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"totalExistentBalance":123.25}"#;

        let retrieve_asset_info_result: RetrieveAssetInfoResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_asset_info_result, RetrieveAssetInfoResult {
            asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
            name: String::from("TestAsset_1"),
            description: String::from("First asset issued for test"),
            can_reissue: false,
            decimal_places: 2,
            issuer: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
            total_existent_balance: 123.25,
        });
    }

    #[test]
    fn it_deserialize_get_asset_balance_result() {
        let json = r#"{"total":123.25,"unconfirmed":0}"#;

        let get_asset_balance_result: GetAssetBalanceResult = serde_json::from_str(json).unwrap();

        assert_eq!(get_asset_balance_result, GetAssetBalanceResult {
            total: 123.25,
            unconfirmed: 0.0,
        });
    }

    #[test]
    fn it_deserialize_list_owned_assets_result() {
        let json = r#"{"ownedAssets":[{"assetId":"aQjlzShmrnEZeeYBZihc","balance":{"total":123.25,"unconfirmed":0}},{"assetId":"a7Sq2A5NXdkoNfbJcdtA","balance":{"total":150,"unconfirmed":0}}],"hasMore":false}"#;

        let list_owned_assets_result: ListOwnedAssetsResult = serde_json::from_str(json).unwrap();

        assert_eq!(list_owned_assets_result, ListOwnedAssetsResult {
            owned_assets: vec![
                OwnedAssetEntry {
                    asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
                    balance: AssetBalance {
                        total: 123.25,
                        unconfirmed: 0.0,
                    },
                },
                OwnedAssetEntry {
                    asset_id: String::from("a7Sq2A5NXdkoNfbJcdtA"),
                    balance: AssetBalance {
                        total: 150.0,
                        unconfirmed: 0.0,
                    },
                },
            ],
            has_more: false,
        });
    }

    #[test]
    fn it_deserialize_list_issued_assets_result() {
        let json = r#"{"issuedAssets":[{"assetId":"aQjlzShmrnEZeeYBZihc","totalExistentBalance":123.25},{"assetId":"a7Sq2A5NXdkoNfbJcdtA","totalExistentBalance":150}],"hasMore":false}"#;

        let list_issued_assets_result: ListIssuedAssetsResult = serde_json::from_str(json).unwrap();

        assert_eq!(list_issued_assets_result, ListIssuedAssetsResult {
            issued_assets: vec![
                IssuedAssetEntry {
                    asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
                    total_existent_balance: 123.25,
                },
                IssuedAssetEntry {
                    asset_id: String::from("a7Sq2A5NXdkoNfbJcdtA"),
                    total_existent_balance: 150.0,
                },
            ],
            has_more: false,
        });
    }

    #[test]
    fn it_deserialize_retrieve_asset_issuance_history_result() {
        let json = r#"{"issuanceEvents":[{"amount":123,"holdingDevice":{"deviceId":"drc3XdxNtzoucpw9xiRp"}},{"amount":150,"holdingDevice":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"}}],"hasMore":false}"#;

        let retrieve_asset_issuance_history_result: RetrieveAssetIssuanceHistoryResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_asset_issuance_history_result, RetrieveAssetIssuanceHistoryResult {
            issuance_events: vec![
                AssetIssuanceEventEntry {
                    amount: 123.0,
                    holding_device: DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: None,
                        prod_unique_id: None,
                    },
                },
                AssetIssuanceEventEntry {
                    amount: 150.0,
                    holding_device: DeviceInfo {
                        device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                        name: None,
                        prod_unique_id: None,
                    },
                },
            ],
            has_more: false,
        });
    }

    #[test]
    fn it_deserialize_list_asset_holders_result() {
        let json = r#"{"assetHolders":[{"holder":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"balance":{"total":123.25,"unconfirmed":0}},{"holder":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"},"balance":{"total":150,"unconfirmed":0}}],"hasMore":false}"#;

        let list_asset_holders_result: ListAssetHoldersResult = serde_json::from_str(json).unwrap();

        assert_eq!(list_asset_holders_result, ListAssetHoldersResult {
            asset_holders: vec![
                AssetHolderEntry {
                    holder: DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: None,
                        prod_unique_id: None,
                    },
                    balance: AssetBalance {
                        total: 123.25,
                        unconfirmed: 0.0,
                    },
                },
                AssetHolderEntry {
                    holder: DeviceInfo {
                        device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                        name: None,
                        prod_unique_id: None,
                    },
                    balance: AssetBalance {
                        total: 150.0,
                        unconfirmed: 0.0,
                    },
                },
            ],
            has_more: false,
        });
    }

    #[test]
    fn it_deserialize_list_permission_events_result() {
        let json = r#"{"receive-notify-new-msg":"Receive notification of new message from a device","receive-msg":"Receive message from a device"}"#;

        let list_permission_events_result: ListPermissionEventsResult = serde_json::from_str(json).unwrap();

        assert_eq!(
            list_permission_events_result,
            vec![
                PermissionEvent::ReceiveNotifyNewMsg,
                PermissionEvent::ReceiveMsg,
            ].into_iter().zip(vec![
                String::from("Receive notification of new message from a device"),
                String::from("Receive message from a device"),
            ]).collect(),
        );
    }

    #[test]
    fn it_deserialize_retrieve_permission_rights_result_no_opts() {
        let json = r#"{"system":"allow"}"#;

        let retrieve_permission_rights_result: RetrievePermissionRightsResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_permission_rights_result, RetrievePermissionRightsResult {
            system: PermissionRight::Allow,
            catenis_node: None,
            client: None,
            device: None,
        });
    }

    #[test]
    fn it_deserialize_retrieve_permission_rights_result_all_opts() {
        let json = r#"{"system":"allow","catenisNode":{"allow":["0"]},"client":{"deny":["cEXd845DSMw9g6tM5dhy"]},"device":{"allow":[{"deviceId":"drc3XdxNtzoucpw9xiRp"}]}}"#;

        let retrieve_permission_rights_result: RetrievePermissionRightsResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_permission_rights_result, RetrievePermissionRightsResult {
            system: PermissionRight::Allow,
            catenis_node: Some(PermissionRightsSetting {
                allow: Some(vec![
                    String::from("0"),
                ]),
                deny: None,
            }),
            client: Some(PermissionRightsSetting {
                allow: None,
                deny: Some(vec![
                    String::from("cEXd845DSMw9g6tM5dhy"),
                ]),
            }),
            device: Some(DevicePermissionRightsSetting {
                allow: Some(vec![
                    DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: None,
                        prod_unique_id: None
                    },
                ]),
                deny: None,
            }),
        });
    }

    #[test]
    fn it_deserialize_set_permission_rights_result() {
        let json = r#"{"success":true}"#;

        let set_permission_rights_result: SetPermissionRightsResult = serde_json::from_str(json).unwrap();

        assert_eq!(set_permission_rights_result, SetPermissionRightsResult {
            success: true,
        });
    }

    #[test]
    fn it_deserialize_check_effective_permission_right_result() {
        //CheckEffectivePermissionRightResult
        let json = r#"{"drc3XdxNtzoucpw9xiRp":"allow"}"#;

        let check_effective_permission_right_result: CheckEffectivePermissionRightResult = serde_json::from_str(json).unwrap();

        assert_eq!(
            check_effective_permission_right_result,
            vec![
                String::from("drc3XdxNtzoucpw9xiRp"),
            ].into_iter().zip(vec![
                PermissionRight::Allow,
            ]).collect(),
        );
    }

    #[test]
    fn it_deserialize_retrieve_device_identification_info_result() {
        let json = r#"{"catenisNode":{"ctnNodeIdx":0},"client":{"clientId":"cEXd845DSMw9g6tM5dhy"},"device":{"deviceId":"drc3XdxNtzoucpw9xiRp"}}"#;

        let retrieve_device_identification_info_result: RetrieveDeviceIdentificationInfoResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_device_identification_info_result, RetrieveDeviceIdentificationInfoResult {
            catenis_node: CatenisNodeInfo {
                ctn_node_idx: 0,
                name: None,
                description: None,
            },
            client: ClientInfo {
                client_id: String::from("cEXd845DSMw9g6tM5dhy"),
                name: None,
            },
            device: DeviceInfo{
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
        });
    }

    #[test]
    fn it_deserialize_list_notification_events_result() {
        let json = r#"{"new-msg-received":"A new message has been received","final-msg-progress":"Progress of asynchronous message processing has come to an end"}"#;

        let list_notification_events_result: ListNotificationEventsResult = serde_json::from_str(json).unwrap();

        assert_eq!(
            list_notification_events_result,
            vec![
                NotificationEvent::NewMsgReceived,
                NotificationEvent::FinalMsgProgress,
            ].into_iter().zip(vec![
                String::from("A new message has been received"),
                String::from("Progress of asynchronous message processing has come to an end"),
            ]).collect(),
        );
    }

    #[test]
    fn it_serialize_log_message_request_no_opts() {
        let log_message_request = LogMessageRequest {
            message: String::from("Test message"),
            options: None,
        };

        let json = serde_json::to_string(&log_message_request).unwrap();

        assert_eq!(json, r#"{"message":"Test message"}"#);
    }

    #[test]
    fn it_serialize_log_message_request_all_opts() {
        let log_message_request = LogMessageRequest {
            message: String::from("Test message"),
            options: Some(LogMessageOptions {
                encoding: Some(Encoding::UTF8),
                encrypt: None,
                off_chain: None,
                storage: None,
                async_: None,
            }),
        };

        let json = serde_json::to_string(&log_message_request).unwrap();

        assert_eq!(json, r#"{"message":"Test message","options":{"encoding":"utf8"}}"#);
    }

    #[test]
    fn it_serialize_log_chunked_message_request_no_opts() {
        let log_chunked_message_request = LogChunkedMessageRequest {
            message: ChunkedMessage {
                data: Some(String::from("Test message")),
                is_final: Some(false),
                continuation_token: None,
            },
            options: None,
        };

        let json = serde_json::to_string(&log_chunked_message_request).unwrap();

        assert_eq!(json, r#"{"message":{"data":"Test message","isFinal":false}}"#);
    }

    #[test]
    fn it_serialize_log_chunked_message_request_all_opts() {
        let log_chunked_message_request = LogChunkedMessageRequest {
            message: ChunkedMessage {
                data: Some(String::from("Test message")),
                is_final: Some(false),
                continuation_token: None,
            },
            options: Some(LogMessageOptions {
                encoding: Some(Encoding::UTF8),
                encrypt: None,
                off_chain: None,
                storage: None,
                async_: None,
            }),
        };

        let json = serde_json::to_string(&log_chunked_message_request).unwrap();

        assert_eq!(json, r#"{"message":{"data":"Test message","isFinal":false},"options":{"encoding":"utf8"}}"#);
    }

    #[test]
    fn it_serialize_send_message_request_no_opts() {
        let send_message_request = SendMessageRequest {
            message: String::from("Test message"),
            target_device: DeviceId {
                id: String::from("drc3XdxNtzoucpw9xiRp"),
                is_prod_unique_id: None
            },
            options: None,
        };

        let json = serde_json::to_string(&send_message_request).unwrap();

        assert_eq!(json, r#"{"message":"Test message","targetDevice":{"id":"drc3XdxNtzoucpw9xiRp"}}"#);
    }

    #[test]
    fn it_serialize_send_message_request_all_opts() {
        let send_message_request = SendMessageRequest {
            message: String::from("Test message"),
            target_device: DeviceId {
                id: String::from("drc3XdxNtzoucpw9xiRp"),
                is_prod_unique_id: None
            },
            options: Some(SendMessageOptions {
                encoding: Some(Encoding::UTF8),
                encrypt: None,
                off_chain: None,
                storage: None,
                read_confirmation: None,
                async_: None,
            }),
        };

        let json = serde_json::to_string(&send_message_request).unwrap();

        assert_eq!(json, r#"{"message":"Test message","targetDevice":{"id":"drc3XdxNtzoucpw9xiRp"},"options":{"encoding":"utf8"}}"#);
    }

    #[test]
    fn it_serialize_send_chunked_message_request_no_opts() {
        let send_chunked_message_request = SendChunkedMessageRequest {
            message: ChunkedMessage {
                data: Some(String::from("Test message")),
                is_final: Some(false),
                continuation_token: None,
            },
            target_device: DeviceId {
                id: String::from("drc3XdxNtzoucpw9xiRp"),
                is_prod_unique_id: None
            },
            options: None,
        };

        let json = serde_json::to_string(&send_chunked_message_request).unwrap();

        assert_eq!(json, r#"{"message":{"data":"Test message","isFinal":false},"targetDevice":{"id":"drc3XdxNtzoucpw9xiRp"}}"#);
    }

    #[test]
    fn it_serialize_send_chunked_message_request_all_opts() {
        let send_chunked_message_request = SendChunkedMessageRequest {
            message: ChunkedMessage {
                data: Some(String::from("Test message")),
                is_final: Some(false),
                continuation_token: None,
            },
            target_device: DeviceId {
                id: String::from("drc3XdxNtzoucpw9xiRp"),
                is_prod_unique_id: None
            },
            options: Some(SendMessageOptions {
                encoding: Some(Encoding::UTF8),
                encrypt: None,
                off_chain: None,
                storage: None,
                read_confirmation: None,
                async_: None,
            }),
        };

        let json = serde_json::to_string(&send_chunked_message_request).unwrap();

        assert_eq!(json, r#"{"message":{"data":"Test message","isFinal":false},"targetDevice":{"id":"drc3XdxNtzoucpw9xiRp"},"options":{"encoding":"utf8"}}"#);
    }

    #[test]
    fn it_serialize_issue_asset_request_no_opts() {
        let issue_asset_request = IssueAssetRequest {
            asset_info: NewAssetInfo {
                name: String::from("TestAsset_1"),
                description: None,
                can_reissue: false,
                decimal_places: 2,
            },
            amount: 123.0,
            holding_device: None,
        };

        let json = serde_json::to_string(&issue_asset_request).unwrap();

        assert_eq!(json, r#"{"assetInfo":{"name":"TestAsset_1","canReissue":false,"decimalPlaces":2},"amount":123.0}"#);
    }

    #[test]
    fn it_serialize_issue_asset_request_all_opts() {
        let issue_asset_request = IssueAssetRequest {
            asset_info: NewAssetInfo {
                name: String::from("TestAsset_1"),
                description: None,
                can_reissue: false,
                decimal_places: 2,
            },
            amount: 123.0,
            holding_device: Some(DeviceId {
                id: String::from("drc3XdxNtzoucpw9xiRp"),
                is_prod_unique_id: None,
            }),
        };

        let json = serde_json::to_string(&issue_asset_request).unwrap();

        assert_eq!(json, r#"{"assetInfo":{"name":"TestAsset_1","canReissue":false,"decimalPlaces":2},"amount":123.0,"holdingDevice":{"id":"drc3XdxNtzoucpw9xiRp"}}"#);
    }

    #[test]
    fn it_serialize_reissue_asset_request_no_opts() {
        let reissue_asset_request = ReissueAssetRequest {
            amount: 123.0,
            holding_device: None,
        };

        let json = serde_json::to_string(&reissue_asset_request).unwrap();

        assert_eq!(json, r#"{"amount":123.0}"#);
    }

    #[test]
    fn it_serialize_reissue_asset_request_all_opts() {
        let reissue_asset_request = ReissueAssetRequest {
            amount: 123.0,
            holding_device: Some(DeviceId {
                id: String::from("drc3XdxNtzoucpw9xiRp"),
                is_prod_unique_id: None,
            }),
        };

        let json = serde_json::to_string(&reissue_asset_request).unwrap();

        assert_eq!(json, r#"{"amount":123.0,"holdingDevice":{"id":"drc3XdxNtzoucpw9xiRp"}}"#);
    }

    #[test]
    fn it_serialize_transfer_asset_request() {
        let transfer_asset_request = TransferAssetRequest {
            amount: 54.0,
            receiving_device: DeviceId {
                id: String::from("drc3XdxNtzoucpw9xiRp"),
                is_prod_unique_id: None,
            },
        };

        let json = serde_json::to_string(&transfer_asset_request).unwrap();

        assert_eq!(json, r#"{"amount":54.0,"receivingDevice":{"id":"drc3XdxNtzoucpw9xiRp"}}"#);
    }

    #[test]
    fn it_deserialize_log_message_response() {
        let json = r#"{"status":"success","data":{"continuationToken":"kjXP2CZaSdkTKCi2jDi2","messageId":"mWg4PS76thiFdTbA56Tg","provisionalMessageId":"pJiMtfdB94YkvRvXp7dA"}}"#;

        let log_message_response: LogMessageResponse = serde_json::from_str(json).unwrap();

        assert_eq!(log_message_response, LogMessageResponse {
            status: String::from("success"),
            data: LogMessageResult {
                continuation_token: Some(String::from("kjXP2CZaSdkTKCi2jDi2")),
                message_id: Some(String::from("mWg4PS76thiFdTbA56Tg")),
                provisional_message_id: Some(String::from("pJiMtfdB94YkvRvXp7dA")),
            },
        });
    }

    #[test]
    fn it_deserialize_send_message_response() {
        let json = r#"{"status":"success","data":{"continuationToken":"kjXP2CZaSdkTKCi2jDi2","messageId":"mWg4PS76thiFdTbA56Tg","provisionalMessageId":"pJiMtfdB94YkvRvXp7dA"}}"#;

        let send_message_response: SendMessageResponse = serde_json::from_str(json).unwrap();

        assert_eq!(send_message_response, SendMessageResponse {
            status: String::from("success"),
            data: SendMessageResult {
                continuation_token: Some(String::from("kjXP2CZaSdkTKCi2jDi2")),
                message_id: Some(String::from("mWg4PS76thiFdTbA56Tg")),
                provisional_message_id: Some(String::from("pJiMtfdB94YkvRvXp7dA")),
            },
        });
    }

    #[test]
    fn it_deserialize_read_message_response() {
        let json = r#"{"status":"success","data":{"msgInfo":{"action":"log"},"msgData":"Test message","continuationToken":"kjXP2CZaSdkTKCi2jDi2","cachedMessageId":"hEXMdtTMzkhyJ4WssQmp"}}"#;

        let read_message_response: ReadMessageResponse = serde_json::from_str(json).unwrap();

        assert_eq!(read_message_response, ReadMessageResponse {
            status: String::from("success"),
            data: ReadMessageResult {
                msg_info: Some(MessageInfo {
                    action: RecordMessageAction::Log,
                    from: None,
                }),
                msg_data: Some(String::from("Test message")),
                continuation_token: Some(String::from("kjXP2CZaSdkTKCi2jDi2")),
                cached_message_id: Some(String::from("hEXMdtTMzkhyJ4WssQmp")),
            },
        });
    }

    #[test]
    fn it_deserialize_retrieve_message_container_response() {
        let json = r#"{"status":"success","data":{"offChain":{"cid":"QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"},"blockchain":{"txid":"505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7","confirmed":true},"externalStorage":{"ipfs":"Qmd2FBqC4dGTbiNe7HyYEJR2grpLWPzDNfBgajkDNRhK1X"}}}"#;

        let retrieve_message_container_response: RetrieveMessageContainerResponse = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_container_response, RetrieveMessageContainerResponse {
            status: String::from("success"),
            data: RetrieveMessageContainerResult {
                off_chain: Some(OffChainContainer {
                    cid: String::from("QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"),
                }),
                blockchain: Some(BlockchainContainer {
                    txid: String::from("505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7"),
                    confirmed: true,
                }),
                external_storage: Some(IpfsStorage {
                    ipfs: String::from("Qmd2FBqC4dGTbiNe7HyYEJR2grpLWPzDNfBgajkDNRhK1X"),
                }),
            },
        });
    }

    #[test]
    fn it_deserialize_retrieve_message_origin_response() {
        let json = r#"{"status":"success","data":{"tx":{"txid":"da00834123604aa1cb6bf911b95f352e2129ee9514478a6d3ac06b04b76cf8cf","type":"Send Message"},"offChainMsgEnvelope":{"cid":"QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX","type":"Send Message"},"proof":{"message":"This is only a test","signature":"IEn9KpwdIitSilCuERUz5Dg+siFe+tW7Bh4ezHDer5NBCQD1jfWJgYL2SnKnzBnGbay/WXL7eykuK8N4o4gRLNo="}}}"#;

        let retrieve_message_origin_response: RetrieveMessageOriginResponse = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_origin_response, RetrieveMessageOriginResponse {
            status: String::from("success"),
            data: RetrieveMessageOriginResult {
                tx: Some(BlockchainTransaction {
                    txid: String::from("da00834123604aa1cb6bf911b95f352e2129ee9514478a6d3ac06b04b76cf8cf"),
                    type_: TransactionType::SendMessage,
                    batch_doc: None,
                    origin_device: None,
                }),
                off_chain_msg_envelope: Some(OffChainMsgEnvelope {
                    cid: String::from("QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"),
                    type_: OffChainMessageType::SendMessage,
                    origin_device: None,
                }),
                proof: ProofInfo {
                    message: String::from("This is only a test"),
                    signature: String::from("IEn9KpwdIitSilCuERUz5Dg+siFe+tW7Bh4ezHDer5NBCQD1jfWJgYL2SnKnzBnGbay/WXL7eykuK8N4o4gRLNo=")
                },
            },
        });
    }

    #[test]
    fn it_deserialize_retrieve_message_progress_response() {
        let json = r#"{"status":"success","data":{"action":"log","progress":{"bytesProcessed":0,"done":false},"result":{"messageId":"mt7ZYbBYpM3zcgAf3H8X"}}}"#;

        let retrieve_message_progress_response: RetrieveMessageProgressResponse = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_progress_response, RetrieveMessageProgressResponse {
            status: String::from("success"),
            data: RetrieveMessageProgressResult {
                action: MessageAction::Log,
                progress: MessageProcessProgress {
                    bytes_processed: 0,
                    done: false,
                    success: None,
                    error: None,
                    finish_date: None,
                },
                result: Some(MessageProcessSuccess {
                    message_id: String::from("mt7ZYbBYpM3zcgAf3H8X"),
                    continuation_token: None,
                }),
            },
        });
    }

    #[test]
    fn it_deserialize_list_messages_response() {
        let json = r#"{"status":"success","data":{"messages":[{"messageId":"moiyHzHxFH8DsyzGjPsh","action":"log","date":"2020-10-23T10:43:21.611Z"},{"messageId":"mWg4PS76thiFdTbA56Tg","action":"log","date":"2020-10-22T14:11:29.692Z"}],"msgCount":2,"hasMore":false}}"#;

        let list_messages_response: ListMessagesResponse = serde_json::from_str(json).unwrap();

        assert_eq!(list_messages_response, ListMessagesResponse {
            status: String::from("success"),
            data: ListMessagesResult {
                messages: vec![
                    MessageEntry {
                        message_id: String::from("moiyHzHxFH8DsyzGjPsh"),
                        action: RecordMessageAction::Log,
                        direction: None,
                        from: None,
                        to: None,
                        read_confirmation_enabled: None,
                        read: None,
                        date: "2020-10-23T10:43:21.611Z".into(),
                    },
                    MessageEntry {
                        message_id: String::from("mWg4PS76thiFdTbA56Tg"),
                        action: RecordMessageAction::Log,
                        direction: None,
                        from: None,
                        to: None,
                        read_confirmation_enabled: None,
                        read: None,
                        date: "2020-10-22T14:11:29.692Z".into(),
                    },
                ],
                msg_count: 2,
                has_more: false,
            },
        });
    }

    #[test]
    fn it_deserialize_issue_asset_response() {
        let json = r#"{"status":"success","data":{"assetId":"aQjlzShmrnEZeeYBZihc"}}"#;

        let issue_asset_response: IssueAssetResponse = serde_json::from_str(json).unwrap();

        assert_eq!(issue_asset_response, IssueAssetResponse {
            status: String::from("success"),
            data: IssueAssetResult {
                asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
            },
        });
    }

    #[test]
    fn it_deserialize_reissue_asset_response() {
        let json = r#"{"status":"success","data":{"totalExistentBalance":123.25}}"#;

        let reissue_asset_response: ReissueAssetResponse = serde_json::from_str(json).unwrap();

        assert_eq!(reissue_asset_response, ReissueAssetResponse {
            status: String::from("success"),
            data: ReissueAssetResult {
                total_existent_balance: 123.25,
            },
        });
    }

    #[test]
    fn it_deserialize_transfer_asset_response() {
        let json = r#"{"status":"success","data":{"remainingBalance":54}}"#;

        let transfer_asset_response: TransferAssetResponse = serde_json::from_str(json).unwrap();

        assert_eq!(transfer_asset_response, TransferAssetResponse {
            status: String::from("success"),
            data: TransferAssetResult {
                remaining_balance: 54.0,
            },
        });
    }

    #[test]
    fn it_deserialize_retrieve_asset_info_response() {
        let json = r#"{"status":"success","data":{"assetId":"aQjlzShmrnEZeeYBZihc","name":"TestAsset_1","description":"First asset issued for test","canReissue":false,"decimalPlaces":2,"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"totalExistentBalance":123.25}}"#;

        let retrieve_asset_info_response: RetrieveAssetInfoResponse = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_asset_info_response, RetrieveAssetInfoResponse {
            status: String::from("success"),
            data: RetrieveAssetInfoResult {
                asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
                name: String::from("TestAsset_1"),
                description: String::from("First asset issued for test"),
                can_reissue: false,
                decimal_places: 2,
                issuer: DeviceInfo {
                    device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                    name: None,
                    prod_unique_id: None,
                },
                total_existent_balance: 123.25,
            },
        });
    }

    #[test]
    fn it_deserialize_get_asset_balance_response() {
        let json = r#"{"status":"success","data":{"total":123.25,"unconfirmed":0}}"#;

        let get_asset_balance_response: GetAssetBalanceResponse = serde_json::from_str(json).unwrap();

        assert_eq!(get_asset_balance_response, GetAssetBalanceResponse {
            status: String::from("success"),
            data: GetAssetBalanceResult {
                total: 123.25,
                unconfirmed: 0.0,
            },
        });
    }

    #[test]
    fn it_deserialize_list_owned_assets_response() {
        let json = r#"{"status":"success","data":{"ownedAssets":[{"assetId":"aQjlzShmrnEZeeYBZihc","balance":{"total":123.25,"unconfirmed":0}},{"assetId":"a7Sq2A5NXdkoNfbJcdtA","balance":{"total":150,"unconfirmed":0}}],"hasMore":false}}"#;

        let list_owned_assets_response: ListOwnedAssetsResponse = serde_json::from_str(json).unwrap();

        assert_eq!(list_owned_assets_response, ListOwnedAssetsResponse {
            status: String::from("success"),
            data: ListOwnedAssetsResult {
                owned_assets: vec![
                    OwnedAssetEntry {
                        asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
                        balance: AssetBalance {
                            total: 123.25,
                            unconfirmed: 0.0,
                        },
                    },
                    OwnedAssetEntry {
                        asset_id: String::from("a7Sq2A5NXdkoNfbJcdtA"),
                        balance: AssetBalance {
                            total: 150.0,
                            unconfirmed: 0.0,
                        },
                    },
                ],
                has_more: false,
            },
        });
    }

    #[test]
    fn it_deserialize_list_issued_assets_response() {
        let json = r#"{"status":"success","data":{"issuedAssets":[{"assetId":"aQjlzShmrnEZeeYBZihc","totalExistentBalance":123.25},{"assetId":"a7Sq2A5NXdkoNfbJcdtA","totalExistentBalance":150}],"hasMore":false}}"#;

        let list_issued_assets_response: ListIssuedAssetsResponse = serde_json::from_str(json).unwrap();

        assert_eq!(list_issued_assets_response, ListIssuedAssetsResponse {
            status: String::from("success"),
            data: ListIssuedAssetsResult {
                issued_assets: vec![
                    IssuedAssetEntry {
                        asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
                        total_existent_balance: 123.25,
                    },
                    IssuedAssetEntry {
                        asset_id: String::from("a7Sq2A5NXdkoNfbJcdtA"),
                        total_existent_balance: 150.0,
                    },
                ],
                has_more: false,
            },
        });
    }

    #[test]
    fn it_deserialize_retrieve_asset_issuance_history_response() {
        let json = r#"{"status":"success","data":{"issuanceEvents":[{"amount":123,"holdingDevice":{"deviceId":"drc3XdxNtzoucpw9xiRp"}},{"amount":150,"holdingDevice":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"}}],"hasMore":false}}"#;

        let retrieve_asset_issuance_history_response: RetrieveAssetIssuanceHistoryResponse = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_asset_issuance_history_response, RetrieveAssetIssuanceHistoryResponse {
            status: String::from("success"),
            data: RetrieveAssetIssuanceHistoryResult {
                issuance_events: vec![
                    AssetIssuanceEventEntry {
                        amount: 123.0,
                        holding_device: DeviceInfo {
                            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                            name: None,
                            prod_unique_id: None,
                        },
                    },
                    AssetIssuanceEventEntry {
                        amount: 150.0,
                        holding_device: DeviceInfo {
                            device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                            name: None,
                            prod_unique_id: None,
                        },
                    },
                ],
                has_more: false,
            },
        });
    }

    #[test]
    fn it_deserialize_list_asset_holders_response() {
        let json = r#"{"status":"success","data":{"assetHolders":[{"holder":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"balance":{"total":123.25,"unconfirmed":0}},{"holder":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"},"balance":{"total":150,"unconfirmed":0}}],"hasMore":false}}"#;

        let list_asset_holders_response: ListAssetHoldersResponse = serde_json::from_str(json).unwrap();

        assert_eq!(list_asset_holders_response, ListAssetHoldersResponse {
            status: String::from("success"),
            data: ListAssetHoldersResult {
                asset_holders: vec![
                    AssetHolderEntry {
                        holder: DeviceInfo {
                            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                            name: None,
                            prod_unique_id: None,
                        },
                        balance: AssetBalance {
                            total: 123.25,
                            unconfirmed: 0.0,
                        },
                    },
                    AssetHolderEntry {
                        holder: DeviceInfo {
                            device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                            name: None,
                            prod_unique_id: None,
                        },
                        balance: AssetBalance {
                            total: 150.0,
                            unconfirmed: 0.0,
                        },
                    },
                ],
                has_more: false,
            },
        });
    }

    #[test]
    fn it_deserialize_list_permission_events_response() {
        let json = r#"{"status":"success","data":{"receive-notify-new-msg":"Receive notification of new message from a device","receive-msg":"Receive message from a device"}}"#;

        let list_permission_events_response: ListPermissionEventsResponse = serde_json::from_str(json).unwrap();

        assert_eq!(
            list_permission_events_response,
            ListPermissionEventsResponse {
                status: String::from("success"),
                data: vec![
                    PermissionEvent::ReceiveNotifyNewMsg,
                    PermissionEvent::ReceiveMsg,
                ].into_iter().zip(vec![
                    String::from("Receive notification of new message from a device"),
                    String::from("Receive message from a device"),
                ]).collect(),
            },
        );
    }

    #[test]
    fn it_deserialize_retrieve_permission_rights_response() {
        let json = r#"{"status":"success","data":{"system":"allow","catenisNode":{"allow":["0"]},"client":{"deny":["cEXd845DSMw9g6tM5dhy"]},"device":{"allow":[{"deviceId":"drc3XdxNtzoucpw9xiRp"}]}}}"#;

        let retrieve_permission_rights_response: RetrievePermissionRightsResponse = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_permission_rights_response, RetrievePermissionRightsResponse {
            status: String::from("success"),
            data: RetrievePermissionRightsResult {
                system: PermissionRight::Allow,
                catenis_node: Some(PermissionRightsSetting {
                    allow: Some(vec![
                        String::from("0"),
                    ]),
                    deny: None,
                }),
                client: Some(PermissionRightsSetting {
                    allow: None,
                    deny: Some(vec![
                        String::from("cEXd845DSMw9g6tM5dhy"),
                    ]),
                }),
                device: Some(DevicePermissionRightsSetting {
                    allow: Some(vec![
                        DeviceInfo {
                            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                            name: None,
                            prod_unique_id: None
                        },
                    ]),
                    deny: None,
                }),
            },
        });
    }

    #[test]
    fn it_deserialize_set_permission_rights_response() {
        let json = r#"{"status":"success","data":{"success":true}}"#;

        let set_permission_rights_response: SetPermissionRightsResponse = serde_json::from_str(json).unwrap();

        assert_eq!(set_permission_rights_response, SetPermissionRightsResponse {
            status: String::from("success"),
            data: SetPermissionRightsResult {
                success: true,
            },
        });
    }

    #[test]
    fn it_deserialize_check_effective_permission_right_response() {
        //CheckEffectivePermissionRightResponse
        let json = r#"{"status":"success","data":{"drc3XdxNtzoucpw9xiRp":"allow"}}"#;

        let check_effective_permission_right_response: CheckEffectivePermissionRightResponse = serde_json::from_str(json).unwrap();

        assert_eq!(
            check_effective_permission_right_response,
            CheckEffectivePermissionRightResponse {
                status: String::from("success"),
                data: vec![
                    String::from("drc3XdxNtzoucpw9xiRp"),
                ].into_iter().zip(vec![
                    PermissionRight::Allow,
                ]).collect(),
            },
        );
    }

    #[test]
    fn it_deserialize_retrieve_device_identification_info_response() {
        let json = r#"{"status":"success","data":{"catenisNode":{"ctnNodeIdx":0},"client":{"clientId":"cEXd845DSMw9g6tM5dhy"},"device":{"deviceId":"drc3XdxNtzoucpw9xiRp"}}}"#;

        let retrieve_device_identification_info_response: RetrieveDeviceIdentificationInfoResponse = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_device_identification_info_response, RetrieveDeviceIdentificationInfoResponse {
            status: String::from("success"),
            data: RetrieveDeviceIdentificationInfoResult {
                catenis_node: CatenisNodeInfo {
                    ctn_node_idx: 0,
                    name: None,
                    description: None,
                },
                client: ClientInfo {
                    client_id: String::from("cEXd845DSMw9g6tM5dhy"),
                    name: None,
                },
                device: DeviceInfo{
                    device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                    name: None,
                    prod_unique_id: None,
                },
            },
        });
    }

    #[test]
    fn it_deserialize_list_notification_events_response() {
        let json = r#"{"status":"success","data":{"new-msg-received":"A new message has been received","final-msg-progress":"Progress of asynchronous message processing has come to an end"}}"#;

        let list_notification_events_response: ListNotificationEventsResponse = serde_json::from_str(json).unwrap();

        assert_eq!(
            list_notification_events_response,
            ListNotificationEventsResponse {
                status: String::from("success"),
                data: vec![
                    NotificationEvent::NewMsgReceived,
                    NotificationEvent::FinalMsgProgress,
                ].into_iter().zip(vec![
                    String::from("A new message has been received"),
                    String::from("Progress of asynchronous message processing has come to an end"),
                ]).collect(),
            },
        );
    }
}