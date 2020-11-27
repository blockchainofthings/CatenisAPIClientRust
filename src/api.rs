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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceId {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_prod_unique_id: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub device_id: String,
    pub name: Option<String>,
    pub prod_unique_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkedMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_final: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Encoding {
    UTF8,
    Base64,
    Hex,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Storage {
    Auto,
    Embedded,
    External,
}

#[derive(Debug, Serialize)]
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
    pub async_: Option<bool>,
}

#[derive(Debug, Serialize)]
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
    pub async_: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadMessageOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_chunk_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub async_: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordMessageAction {
    Log,
    Send,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    pub action: RecordMessageAction,
    pub from: Option<DeviceInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OffChainContainer {
    pub cid: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainContainer {
    pub txid: String,
    pub confirmed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpfsStorage {
    pub ipfs: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceOwner {
    pub company: Option<String>,
    pub contact: Option<String>,
    pub name: Option<String>,
    pub domains: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginDeviceInfo {
    pub address: String,
    pub device_id: String,
    pub name: Option<String>,
    pub prod_unique_id: Option<String>,
    pub owned_by: DeviceOwner,
}

#[derive(Debug, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "Send Message")]
    SendMessage,
    #[serde(rename = "Log Message")]
    LogMessage,
    #[serde(rename = "Settle Off-Chain Message")]
    SettleOffChainMessage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDocRef {
    pub cid: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainTransaction {
    pub txid: String,
    pub type_: TransactionType,
    pub batch_doc: Option<BatchDocRef>,
    pub origin_device: Option<OriginDeviceInfo>,
}

#[derive(Debug, Deserialize)]
pub enum OffChainMessageType {
    #[serde(rename = "Send Message")]
    SendMessage,
    #[serde(rename = "Log Message")]
    LogMessage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OffChainMsgEnvelope {
    pub cid: String,
    pub type_: OffChainMessageType,
    pub origin_device: Option<OffChainOriginDeviceInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofInfo {
    pub message: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OffChainOriginDeviceInfo {
    pub pub_key_hash: String,
    pub device_id: String,
    pub name: Option<String>,
    pub prod_unique_id: Option<String>,
    pub owned_by: DeviceOwner,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessError {
    pub code: u16,
    pub message: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessProgress {
    pub bytes_processed: usize,
    pub done: bool,
    pub success: Option<bool>,
    pub error: Option<MessageProcessError>,
    pub finish_date: Option<UtcDateTime>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessSuccess {
    pub message_id: String,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageAction {
    Log,
    Send,
    Read,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageActionOption {
    Log,
    Send,
    Any,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirectionOption {
    Inbound,
    Outbound,
    Any,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageReadStateOption {
    Read,
    Unread,
    Any,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMessagesOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<MessageActionOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<MessageDirectionOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_devices: Option<Vec<DeviceId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_devices: Option<Vec<DeviceId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_state: Option<MessageReadStateOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<UtcDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<UtcDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAssetInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub can_reissue: bool,
    pub decimal_places: u8
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetBalance {
    pub total: f64,
    pub unconfirmed: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedAssetEntry {
    pub asset_id: String,
    pub balance: AssetBalance,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuedAssetEntry {
    pub asset_id: String,
    pub total_existent_balance: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIssuanceEventEntry {
    pub amount: f64,
    pub holding_device: DeviceInfo,

}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetHolderEntry {
    pub holder: DeviceInfo,
    pub balance: AssetBalance,
}

#[derive(Debug, PartialEq, Eq, Hash)]
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
            PermissionEvent::UnknownEvent(s) => s.as_str()
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionRight {
    Allow,
    Deny,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRightsSetting {
    pub allow: Option<Vec<String>>,
    pub deny: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePermissionRightsSetting {
    pub allow: Option<Vec<DeviceInfo>>,
    pub deny: Option<Vec<DeviceInfo>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRightsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub none: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePermissionRightsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<DeviceId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<DeviceId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub none: Option<Vec<DeviceId>>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatenisNodeInfo {
    pub ctn_node_idx: u32,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub client_id: String,
    pub name: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NotificationEvent {
    NewMsgReceived,
    SentMsgRead,
    AssetReceived,
    AssetConfirmed,
    FinalMessageProgress,
    InvalidEvent,
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
            NotificationEvent::FinalMessageProgress => "final-msg-progress",
            NotificationEvent::InvalidEvent => "invalid-event",
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
            "final-msg-progress" => NotificationEvent::FinalMessageProgress,
            _ => NotificationEvent::InvalidEvent,
        }
    }
}

// Result (used in response) data structures

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogMessageResult {
    pub continuation_token: Option<String>,
    pub message_id: Option<String>,
    pub provisional_message_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResult {
    pub continuation_token: Option<String>,
    pub message_id: Option<String>,
    pub provisional_message_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadMessageResult {
    pub msg_info: Option<MessageInfo>,
    pub msg_data: Option<String>,
    pub continuation_toke: Option<String>,
    pub cached_message_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageContainerResult {
    pub off_chain: Option<OffChainContainer>,
    pub blockchain: Option<BlockchainContainer>,
    pub external_storage: Option<IpfsStorage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageOriginResult {
    pub tx: Option<BlockchainTransaction>,
    pub off_chain_msg_envelope: Option<OffChainMsgEnvelope>,
    pub proof: ProofInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageProgressResult {
    pub action: MessageAction,
    pub progress: MessageProcessProgress,
    pub result: Option<MessageProcessSuccess>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMessagesResult {
    pub messages: Vec<MessageEntry>,
    pub msg_count: u16,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueAssetResult {
    pub asset_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReissueAssetResult {
    pub total_existent_balance: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferAssetResult {
    pub remaining_balance: f64,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetBalanceResult {
    pub total: f64,
    pub unconfirmed: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOwnedAssetsResult {
    pub owned_assets: Vec<OwnedAssetEntry>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIssuedAssetsResult {
    pub issued_assets: Vec<IssuedAssetEntry>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveAssetIssuanceHistoryResult {
    pub issuance_events: Vec<AssetIssuanceEventEntry>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAssetHoldersResult {
    pub asset_holders: Vec<AssetHolderEntry>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievePermissionRightsResult {
    pub system: PermissionRight,
    pub catenis_node: Option<PermissionRightsSetting>,
    pub client: Option<PermissionRightsSetting>,
    pub device: Option<DevicePermissionRightsSetting>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPermissionRightsResult {
    pub success: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveDeviceIdentificationInfoResult {
    pub catenis_node: CatenisNodeInfo,
    pub client: ClientInfo,
    pub device: DeviceInfo,
}

// Request data structures

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogMessageRequest {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<LogMessageOptions>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogChunkedMessageRequest {
    pub message: ChunkedMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<LogMessageOptions>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SendMessageRequest {
    pub message: String,
    pub target_device: DeviceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<SendMessageOptions>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SendChunkedMessageRequest {
    pub message: ChunkedMessage,
    pub target_device: DeviceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<SendMessageOptions>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssueAssetRequest {
    pub asset_info: NewAssetInfo,
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holding_device: Option<DeviceId>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReissueAssetRequest {
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holding_device: Option<DeviceId>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferAssetRequest {
    pub amount: f64,
    pub receiving_device: DeviceId,
}

// Response data structures

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogMessageResponse {
    pub status: String,
    pub data: LogMessageResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse {
    pub status: String,
    pub data: SendMessageResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadMessageResponse {
    pub status: String,
    pub data: ReadMessageResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageContainerResponse {
    pub status: String,
    pub data: RetrieveMessageContainerResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageOriginResponse {
    pub status: String,
    pub data: RetrieveMessageOriginResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageProgressResponse {
    pub status: String,
    pub data: RetrieveMessageProgressResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMessagesResponse {
    pub status: String,
    pub data: ListMessagesResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueAssetResponse {
    pub status: String,
    pub data: IssueAssetResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReissueAssetResponse {
    pub status: String,
    pub data: ReissueAssetResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferAssetResponse {
    pub status: String,
    pub data: TransferAssetResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveAssetInfoResponse {
    pub status: String,
    pub data: RetrieveAssetInfoResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetBalanceResponse {
    pub status: String,
    pub data: GetAssetBalanceResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOwnedAssetsResponse {
    pub status: String,
    pub data: ListOwnedAssetsResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIssuedAssetsResponse {
    pub status: String,
    pub data: ListIssuedAssetsResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveAssetIssuanceHistoryResponse {
    pub status: String,
    pub data: RetrieveAssetIssuanceHistoryResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAssetHoldersResponse {
    pub status: String,
    pub data: ListAssetHoldersResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPermissionEventsResponse {
    pub status: String,
    pub data: ListPermissionEventsResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievePermissionRightsResponse {
    pub status: String,
    pub data: RetrievePermissionRightsResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPermissionRightsResponse {
    pub status: String,
    pub data: SetPermissionRightsResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckEffectivePermissionRightResponse {
    pub status: String,
    pub data: CheckEffectivePermissionRightResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveDeviceIdentificationInfoResponse {
    pub status: String,
    pub data: RetrieveDeviceIdentificationInfoResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListNotificationEventsResponse {
    pub status: String,
    pub data: ListNotificationEventsResult,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NewAssetInfo;

    #[test]
    fn float_precision() {
        println!(">>>>>> Result (0.1): {}", 0.1_f64);
        println!(">>>>>> Result (0.2): {}", 0.2_f64);
        println!(">>>>>> Result (0.1 + 0.2): {}", 0.12345678 + 0.23456789);
        println!(">>>>>> Result (0.1 str): {}", 0.12345678.to_string());

        let precision = 2;

        let _d = NewAssetInfo {
            name: String::from("Test asset"),
            description: None,
            can_reissue: true,
            decimal_places: precision,
        };
    }

    #[test]
    fn it_serialize_deserialize_none() {
        let device_id = DeviceId {
            id: String::from("dfjkafkjsajflsjl"),
            is_prod_unique_id: Some(true),
        };

        let dev_id_json = serde_json::to_string(&device_id).unwrap();

        println!(">>>>>> Serialized Device ID: {:?}", dev_id_json);

        let dev_info_json = r#"{
            "deviceId": "dfjkafkjsajflsjl",
            "prodUniqueId": "XYZ-0001"
        }"#;

        let device_info: DeviceInfo = serde_json::from_str(dev_info_json).unwrap();

        println!(">>>>>> Deserialized Device Info: {:?}", device_info);
    }

    #[test]
    fn it_converts_notification_event() {
        let event_str = NotificationEvent::SentMsgRead.to_string();
        let event: NotificationEvent = event_str.as_str().into();
        let event2: NotificationEvent = "new-msg-received".into();

        println!(">>>>>> event_str: {}", event_str);
        println!(">>>>>> event: {:?}", event);
        println!(">>>>>> event2: {:?}", event2);
    }

    #[test]
    fn it_deserialize_permission_event() {
        let json = r#""anything""#;

        let event: PermissionEvent = serde_json::from_str(json).unwrap();

        println!(">>>>>> Deserialized permission event: {:?}", event);
    }

    #[test]
    fn it_build_permission_map() {
        let mut map = HashMap::<PermissionEvent, String>::new();

        map.insert(PermissionEvent::ReceiveNotifyConfirmAssetOf, String::from("First value"));
        map.insert(PermissionEvent::UnknownEvent(String::from("anything")), String::from("Second value"));
        map.insert(PermissionEvent::UnknownEvent(String::from("something else")), String::from("Third value"));
        map.insert(PermissionEvent::UnknownEvent(String::from("anything")), String::from("Forth value"));

        println!(">>>>>> Built permission event map: {:?}", map);
    }
}