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
use crate::{
    JsonValue,
    JsonMap,
};

use crate::date_time::UtcDateTime;

/// Data returned from a successful call to *List Permission Events* API method.
pub type ListPermissionEventsResult = HashMap<PermissionEvent, String>;
/// Data returned from a successful call to *Check Effective Permission Right* API method.
pub type CheckEffectivePermissionRightResult = HashMap<String, PermissionRight>;
/// Data returned from a successful call to *List Notification Events* API method.
pub type ListNotificationEventsResult = HashMap<NotificationEvent, String>;
/// Representation of a JSON object
pub type JsonObject = JsonMap<String, JsonValue>;

pub trait IntoJsonObj {
    fn json_obj(self) -> Option<JsonObject>;
}

impl IntoJsonObj for JsonValue {
    fn json_obj(self) -> Option<JsonObject> {
        if let JsonValue::Object(obj) = self {
            Some(obj)
        } else {
            None
        }
    }
}

/// Identifies a given virtual device.
#[derive(Debug, Serialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct DeviceId {
    /// The ID of the device.
    pub id: String,
    /// Indicates whether the contained ID is a product unique ID. Otherwise, it is a Catenis
    /// device ID.
    ///
    /// Default value: **`false`**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_prod_unique_id: Option<bool>,
}

/// Basic identification information of a virtual device.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    /// The device ID.
    pub device_id: String,
    /// The device's name.
    ///
    /// > **Note**: only returned if it is defined for this virtual device.
    pub name: Option<String>,
    /// The device's product unique ID.
    ///
    /// > **Note**: only returned if it is defined for this virtual device.
    pub prod_unique_id: Option<String>,
}

/// Data structure used to pass message in chunks.
#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChunkedMessage {
    /// The current message data chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Indicates whether this is the final message data chunk.
    ///
    /// Default value: **`true`**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_final: Option<bool>,
    /// The continuation token returned for the previously sent message data chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}

/// A message to be recorded to the blockchain.
#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum Message {
    /// A complete message.
    Whole(String),
    /// A message chunk.
    Chunk(ChunkedMessage),
}

/// Text encoding.
#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Encoding {
    /// Text is formatted as UTF-8 characters.
    UTF8,
    /// Text is base64 encoded.
    Base64,
    /// Text is in hexadecimal.
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

/// Storage scheme used for the message.
#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Storage {
    /// Store message in the blockchain transaction if it fits. Otherwise, store message in the
    /// external repository.
    Auto,
    /// Store message in the blockchain transaction.
    Embedded,
    /// Store message in an external repository.
    External,
}

/// Option settings for logging a message.
#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogMessageOptions {
    /// The text encoding of the message.
    ///
    /// Default value: **`Encoding::UTF8`**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,
    /// Indicates whether message should be encrypted before storing it.
    ///
    /// Default value: **`true`**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypt: Option<bool>,
    /// Indicates whether message should be logged as a Catenis off-chain message.
    ///
    /// Default value: **`true`**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub off_chain: Option<bool>,
    /// Identifies where the message should be stored.
    ///
    /// Default value: **`Storage::Auto`**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<Storage>,
    /// Indicates whether processing — storage of message to the blockchain — should be done
    /// asynchronously.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "async")]
    pub async_: Option<bool>,
}

/// Option settings for sending a message.
#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageOptions {
    /// The text encoding of the message.
    ///
    /// Default value: **`Encoding::UTF8`**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,
    /// Indicates whether message should be encrypted before storing it.
    ///
    /// Default value: **`true`**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypt: Option<bool>,
    /// Indicates whether message should be sent as a Catenis off-chain message.
    ///
    /// Default value: **`true`**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub off_chain: Option<bool>,
    /// Identifies where the message should be stored.
    ///
    /// Default value: **`Storage::Auto`**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<Storage>,
    /// Indicates whether message should be sent with read confirmation enabled.
    ///
    /// Default value: **`false`**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_confirmation: Option<bool>,
    /// Indicates whether processing — storage of message to the blockchain — should be done
    /// asynchronously.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "async")]
    pub async_: Option<bool>,
}

/// Option settings for reading a message.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReadMessageOptions {
    /// The text encoding to be used for the message's contents.
    ///
    /// Default value: **`Encoding::UTF8`**.
    pub encoding: Option<Encoding>,
    /// Continuation token for reading the next message data chunk.
    ///
    /// > **Note**: this should be supplied when reading the message in chunks, or when reading
    /// the message asynchronously after the processing was complete.
    pub continuation_token: Option<String>,
    /// Size, in bytes, of the largest message data chunk that should be returned.
    ///
    /// It must be an integer value between 1,024 (1 KB) and 15,728,640 (15 MB).
    pub data_chunk_size: Option<usize>,
    /// Indicates whether processing — retrieval of message from the blockchain — should be done
    /// asynchronously.
    pub async_: Option<bool>,
}

/// Action used to record a message to the blockchain.
#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecordMessageAction {
    /// Record message on the blockchain (for that same virtual device).
    Log,
    /// Record message on the blockchain directing it to another virtual device.
    Send,
}

/// Information about a previously recorded message.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    /// Action originally performed on the message.
    pub action: RecordMessageAction,
    /// Identifies the virtual device that sent the message — the *origin device*.
    pub from: Option<DeviceInfo>,
}

/// Information about off-chain message container.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OffChainContainer {
    /// IPFS CID of Catenis off-chain message envelope data structure that holds the off-chain
    /// message's contents.
    pub cid: String,
}

/// Information about message container on the blockchain.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainContainer {
    /// The ID of the blockchain transaction where the message was recorded.
    pub txid: String,
    /// Indicates whether the blockchain transaction has already been confirmed.
    pub is_confirmed: bool,
}

/// Information about message container on the IPFS external storage.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IpfsStorage {
    /// The hash used to reference the message on IPFS.
    pub ipfs: String,
}

/// Information about the virtual device's owner.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceOwner {
    /// The name of the company that owns the virtual device.
    ///
    /// > **Note**: only returned if it is defined for that virtual device.
    pub company: Option<String>,
    /// The name of the company's contact.
    ///
    /// > **Note**: only returned if both a company and a contact are defined for that virtual
    /// device.
    pub contact: Option<String>,
    /// The name of the person who owns the virtual device.
    ///
    /// > **Note**: only returned if a contact without a company is defined for that virtual device.
    pub name: Option<String>,
    /// List of Internet domains owned by this company or person.
    pub domains: Option<Vec<String>>,
}

/// Information about the virtual device that recorded a message.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OriginDeviceInfo {
    /// Virtual device's blockchain address used to generate the Catenis message transaction.
    pub address: String,
    /// The device ID.
    pub device_id: String,
    /// The device's name.
    ///
    /// > **Note**: only returned if it is defined for this virtual device.
    pub name: Option<String>,
    /// The device's product unique ID.
    ///
    /// > **Note**: only returned if it is defined for this virtual device.
    pub prod_unique_id: Option<String>,
    /// Virtual device owner info.
    pub owned_by: DeviceOwner,
}

/// Type of a Catenis message transaction.
#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum TransactionType {
    /// Send message transaction.
    #[serde(rename = "Send Message")]
    SendMessage,
    /// Log message transaction.
    #[serde(rename = "Log Message")]
    LogMessage,
    /// Settle off-chain messages transaction.
    #[serde(rename = "Settle Off-Chain Messages")]
    SettleOffChainMessages,
}

/// Reference to the batch document used to settle off-chain messages on the blockchain.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BatchDocRef {
    /// Content ID (CID) of batch document on IPFS.
    pub cid: String,
}

/// Information about the blockchain transaction used to record a message.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainTransaction {
    /// Blockchain transaction ID.
    pub txid: String,
    /// Catenis message transaction type.
    #[serde(rename = "type")]
    pub type_: TransactionType,
    /// Off-chain message batch document reference.
    ///
    /// > **Note**: only returned for off-chain messages.
    pub batch_doc: Option<BatchDocRef>,
    /// Origin device info.
    ///
    /// > **Note**: not returned for off-chain messages.
    pub origin_device: Option<OriginDeviceInfo>,
}

/// Type of an off-chain message.
#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum OffChainMessageType {
    /// Message sent to another virtual device.
    #[serde(rename = "Send Message")]
    SendMessage,
    /// Message logged (for that same virtual device).
    #[serde(rename = "Log Message")]
    LogMessage,
}

/// Information about the off-chain message envelope used to record an off-chain message.
///
/// **Note**: the off-chain message is recorded on IPFS.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OffChainMsgEnvelope {
    /// Content ID (CID) of off-chain message envelope on IPFS.
    pub cid: String,
    /// Off-chain message type.
    #[serde(rename = "type")]
    pub type_: OffChainMessageType,
    /// Origin device info.
    pub origin_device: Option<OffChainOriginDeviceInfo>,
}

/// Data used to proof the origin of a message.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProofInfo {
    /// User provided text message for which the signature was generated.
    pub message: String,
    /// Base64-encoded message's signature generated using origin device's private key.
    pub signature: String,
}

/// Information about the virtual device that recorded an off-chain message.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OffChainOriginDeviceInfo {
    /// Hex-encoded hash of virtual device's public key used to generate the off-chain message
    /// envelope.
    pub pub_key_hash: String,
    /// The device ID.
    pub device_id: String,
    /// The device's name.
    ///
    /// > **Note**: only returned if it is defined for this virtual device.
    pub name: Option<String>,
    /// The device's product unique ID.
    ///
    /// > **Note**: only returned if it is defined for this virtual device.
    pub prod_unique_id: Option<String>,
    /// Virtual device owner info.
    pub owned_by: DeviceOwner,
}

/// Information about an error processing a message asynchronously.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessError {
    /// Numeric code — equivalent to an HTML status code — of the error that took place while
    /// processing the message.
    pub code: u16,
    /// Text describing the error that took place while processing the message.
    pub message: String
}

/// Report on the asynchronous processing status of a message.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessProgress {
    /// Total number of bytes of message that had already been processed.
    pub bytes_processed: usize,
    /// Indicates whether processing has been finished.
    pub done: bool,
    /// Indicates whether the message has been successfully processed.
    ///
    /// > **Note**: only returned if processing has already been finished.
    pub success: Option<bool>,
    /// Processing error.
    ///
    /// > **Note**: only returned if processing finished with error.
    pub error: Option<MessageProcessError>,
    /// Date and time when processing was finalized.
    ///
    /// > **Note**: only returned if processing has already been finished.
    pub finish_date: Option<UtcDateTime>,
}

/// The successful outcome of the asynchronous processing of a message.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessSuccess {
    /// The message ID.
    ///
    /// When logging or sending a message, this is the resulting ID of the message.
    ///
    /// When reading a message, it is the ID of the message being read.
    pub message_id: String,
    /// The token that should be used to complete the read of the message.
    ///
    /// > **Note**: only returned if reading a message.
    pub continuation_token: Option<String>,
}

/// Action to be asynchronously processed on a message.
#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageAction {
    /// Record message on the blockchain (for that same virtual device).
    Log,
    /// Record message on the blockchain directing it to another virtual device.
    Send,
    /// Reads a previously recorded message from the blockchain.
    Read,
}

/// Option for filtering messages according to the action performed on it.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MessageActionOption {
    /// Logged messages.
    Log,
    /// Sent messages.
    Send,
    /// Any message action.
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

/// Option for filtering messages according to its direction.
///
/// > **Note**: it only applies to sent messages.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MessageDirectionOption {
    /// Messages sent to the virtual device making the query.
    Inbound,
    /// Messages sent from the virtual device making the query.
    Outbound,
    /// Any message direction.
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

/// Option for filtering messages according to its read state.
///
/// > **Note**: it only applies to sent messages.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MessageReadStateOption {
    /// Messages already read.
    Read,
    /// Messages not yet read.
    Unread,
    /// Any message read state.
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

/// Options for filtering messages.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ListMessagesOptions {
    /// Message action.
    pub action: Option<MessageActionOption>,
    /// Message direction.
    pub direction: Option<MessageDirectionOption>,
    /// Devices that sent the message.
    ///
    /// > **Note**: it only applies to inbound sent messages.
    pub from_devices: Option<Vec<DeviceId>>,
    /// Devices to which the message was sent.
    ///
    /// > **Note**: it only applies to outbound sent messages.
    pub to_devices: Option<Vec<DeviceId>>,
    /// Message read state.
    pub read_state: Option<MessageReadStateOption>,
    /// Start of time period within which message was:
    ///
    /// - *logged*, for logged messages.
    /// - *sent*, for outbound sent messages.
    /// - *received*, for inbound sent messages.
    pub start_date: Option<UtcDateTime>,
    /// End of time period within which message was:
    ///
    /// - *logged*, for logged messages.
    /// - *sent*, for outbound sent messages.
    /// - *received*, for inbound sent messages.
    pub end_date: Option<UtcDateTime>,
    /// Maximum number of messages that should be returned.
    ///
    /// > **Note**: must be a positive integer value not greater than 500.
    pub limit: Option<u16>,
    /// Number of messages that should be skipped.
    ///
    /// > **Note**: must be a non-negative (includes zero) integer value.
    pub skip: Option<usize>,
}

/// Direction of a sent message.
#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    /// Message sent to the virtual device making the query.
    Inbound,
    /// Message sent from the virtual device making the query.
    Outbound,
}

/// Information about a listed message.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageEntry {
    /// The message ID.
    pub message_id: String,
    /// Action originally performed on the message.
    pub action: RecordMessageAction,
    /// Message direction.
    ///
    /// > **Note**: only returned for sent messages.
    pub direction: Option<MessageDirection>,
    /// The virtual device that sent the message.
    ///
    /// > **Note**: only returned for sent messages.
    pub from: Option<DeviceInfo>,
    /// The virtual device to which the message was sent.
    ///
    /// > **Note**: only returned for sent messages.
    pub to: Option<DeviceInfo>,
    /// Indicates whether the message had been sent with read confirmation enabled.
    ///
    /// > **Note**: only returned for outbound sent messages.
    pub read_confirmation_enabled: Option<bool>,
    /// Indicates whether the message had already been read.
    ///
    /// > **Note**: not returned for outbound sent messages sent with read confirmation not enabled.
    pub read: Option<bool>,
    /// Date and time when the message was:
    ///
    /// - *logged*, for logged messages.
    /// - *sent*, for outbound sent messages.
    /// - *received*, for inbound sent messages.
    pub date: UtcDateTime,
}

/// Properties for a new asset.
#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewAssetInfo {
    /// The asset name.
    pub name: String,
    /// A description about the asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Indicates whether more units of the asset can be issued at another time.
    pub can_reissue: bool,
    /// The maximum number of decimal places that can be used to specify a fractional amount
    /// of the asset.
    pub decimal_places: u8,
}

/// Balance of an asset.
#[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetBalance {
    /// Current balance of that asset held by the virtual device that makes the request.
    pub total: f64,
    /// The amount from the balance that is not yet confirmed.
    pub unconfirmed: f64,
}

/// Information about an owned asset.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OwnedAssetEntry {
    /// The asset ID.
    pub asset_id: String,
    /// The asset balance.
    pub balance: AssetBalance,
}

/// Information about an issued asset.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IssuedAssetEntry {
    /// The asset ID.
    pub asset_id: String,
    /// Current total balance of that asset in existence.
    pub total_existent_balance: f64,
}

/// Information about a regular (fungible) asset issuance event.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RegularAssetIssuanceEventEntry {
    /// The issued amount of the asset.
    pub amount: f64,
    /// The virtual device to which the issued amount was assigned.
    pub holding_device: DeviceInfo,
    /// Date and time when the asset was issued.
    pub date: UtcDateTime,
}

/// Information about a non-fungible asset issuance event.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NonFungibleAssetIssuanceEventEntry {
    /// List of the IDs of the non-fungible tokens of the asset that have been issued.
    pub nf_token_ids: Vec<String>,
    /// List of the virtual devices to which the issued non-fungible tokens were assigned.
    pub holding_devices: Vec<DeviceInfo>,
    /// Date and time when the asset was issued.
    pub date: UtcDateTime,
}

/// Asset issuance event entry
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum AssetIssuanceEventEntry {
    /// Entry for regular (fungible) assets.
    Regular(RegularAssetIssuanceEventEntry),
    /// Entry for non-fungible assets.
    NonFungible(NonFungibleAssetIssuanceEventEntry),
}

/// Information about an asset holder.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetHolderEntry {
    /// The virtual device that holds an amount of the asset.
    ///
    /// > **Note**: not returned for the special entry reporting the migrated asset amount.
    pub holder: Option<DeviceInfo>,
    /// The value **`true`** indicating that this is the special entry reporting the migrated asset
    /// amount.
    ///
    /// > **Note**: only returned for the special entry reporting the migrated asset amount.
    pub migrated: Option<bool>,
    /// The asset balance.
    pub balance: AssetBalance,
}

/// Catenis permission event.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum PermissionEvent {
    /// The *receive-notify-new-msg* permission event.
    ReceiveNotifyNewMsg,
    /// The *receive-notify-msg-read* permission event.
    ReceiveNotifyMsgRead,
    /// The *receive-notify-asset-of* permission event.
    ReceiveNotifyAssetOf,
    /// The *receive-notify-asset-from* permission event.
    ReceiveNotifyAssetFrom,
    /// The *receive-notify-confirm-asset-of* permission event.
    ReceiveNotifyConfirmAssetOf,
    /// The *receive-notify-confirm-asset-from* permission event.
    ReceiveNotifyConfirmAssetFrom,
    /// The *send-read-msg-confirm* permission event.
    SendReadMsgConfirm,
    /// The *receive-msg* permission event.
    ReceiveMsg,
    /// The *disclose-main-props* permission event.
    DiscloseMainProps,
    /// The *disclose-identify-info* permission event.
    DiscloseIdentityInfo,
    /// The *receive-asset-of* permission event.
    ReceiveAssetOf,
    /// The *receive-asset-from* permission event.
    ReceiveAssetFrom,
    /// The *receive-nf-token-of* permission event.
    ReceiveNFTokenOf,
    /// The *receive-nf-token-from* permission event.
    ReceiveNFTokenFrom,
    /// Any unknown permission event.
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
            PermissionEvent::ReceiveNFTokenOf => "receive-nf-token-of",
            PermissionEvent::ReceiveNFTokenFrom => "receive-nf-token-from",
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
            "receive-nf-token-of" => PermissionEvent::ReceiveNFTokenOf,
            "receive-nf-token-from" => PermissionEvent::ReceiveNFTokenFrom,
            s @ _ => PermissionEvent::UnknownEvent(String::from(s)),
        }
    }
}

/// Defined permission right.
#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionRight {
    /// Permission granted.
    Allow,
    /// Permission not granted.
    Deny,
}

/// Current permission rights set for different entities.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRightsSetting {
    /// List of entities to which permission is granted.
    ///
    /// > **Note**: the entity is identified by its ID: either a Catenis node index, or a client ID.
    pub allow: Option<Vec<String>>,
    /// List of entities to which permission is not granted.
    ///
    /// > **Note**: the entity is identified by its ID: either a Catenis node index, or a client ID.
    pub deny: Option<Vec<String>>,
}

/// Current permission rights set for different virtual devices.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DevicePermissionRightsSetting {
    /// List of virtual devices to which permission is granted.
    pub allow: Option<Vec<DeviceInfo>>,
    /// List of virtual devices to which permission is not granted.
    pub deny: Option<Vec<DeviceInfo>>,
}

/// Data for changing permission rights for different entities.
#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRightsUpdate {
    /// List of entities to which permission should be granted.
    ///
    /// > **Note**: the entity is identified by its ID: either a Catenis node index, or a client ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
    /// List of entities to which permission should not be granted.
    ///
    /// > **Note**: the entity is identified by its ID: either a Catenis node index, or a client ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
    /// List of entities from which permission should be cleared.
    ///
    /// > **Note**: the entity is identified by its ID: either a Catenis node index, or a client ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub none: Option<Vec<String>>,
}

/// Data for changing permission rights for different virtual devices.
#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DevicePermissionRightsUpdate {
    /// List of virtual devices to which permission should be granted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<DeviceId>>,
    /// List of virtual devices to which permission should not be granted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<DeviceId>>,
    /// List of virtual devices from which permission should be cleared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub none: Option<Vec<DeviceId>>,
}

/// Data for changing permissions rights at all levels.
#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AllPermissionRightsUpdate {
    /// Updated permission right at system level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<PermissionRight>,
    /// Updated permission rights at Catenis node level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catenis_node: Option<PermissionRightsUpdate>,
    /// Updated permission rights at client level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<PermissionRightsUpdate>,
    /// Updated permission rights at device level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<DevicePermissionRightsUpdate>,
}

/// Identification information of a Catenis node.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatenisNodeInfo {
    /// The Catenis node index.
    pub ctn_node_index: u32,
    /// The Catenis node's name.
    ///
    /// > **Note**: only returned if it is defined for this Catenis node.
    pub name: Option<String>,
    /// A description about the Catenis node.
    ///
    /// > **Note**: only returned if it is defined for this Catenis node.
    pub description: Option<String>,
}

/// Identification information of a client.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    /// The client ID.
    pub client_id: String,
    /// The client's name.
    ///
    /// > **Note**: only returned if it is defined for this client.
    pub name: Option<String>,
}

/// Catenis notification event.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum NotificationEvent {
    /// The *new-msg-received* notification event.
    NewMsgReceived,
    /// The *sent-msg-read* notification event.
    SentMsgRead,
    /// The *asset-received* notification event.
    AssetReceived,
    /// The *asset-confirmed* notification event.
    AssetConfirmed,
    /// The *final-msg-progress* notification event.
    FinalMsgProgress,
    /// The *asset-export-outcome* notification event.
    AssetExportOutcome,
    /// The *asset-migration-outcome* notification event.
    AssetMigrationOutcome,
    /// The *nf-asset-issuance-outcome* notification event.
    NFAssetIssuanceOutcome,
    /// The *nf-token-received* notification event.
    NFTokenReceived,
    /// The *nf-token-confirmed* notification event.
    NFTokenConfirmed,
    /// The *nf-token-retrieval-outcome* notification event.
    NFTokenRetrievalOutcome,
    /// The *nf-token-transfer-outcome* notification event.
    NFTokenTransferOutcome,
    /// Any unknown notification event.
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
            NotificationEvent::AssetExportOutcome => "asset-export-outcome",
            NotificationEvent::AssetMigrationOutcome => "asset-migration-outcome",
            NotificationEvent::NFAssetIssuanceOutcome => "nf-asset-issuance-outcome",
            NotificationEvent::NFTokenReceived => "nf-token-received",
            NotificationEvent::NFTokenConfirmed => "nf-token-confirmed",
            NotificationEvent::NFTokenRetrievalOutcome => "nf-token-retrieval-outcome",
            NotificationEvent::NFTokenTransferOutcome => "nf-token-transfer-outcome",
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
            "asset-export-outcome" => NotificationEvent::AssetExportOutcome,
            "asset-migration-outcome" => NotificationEvent::AssetMigrationOutcome,
            "nf-asset-issuance-outcome" => NotificationEvent::NFAssetIssuanceOutcome,
            "nf-token-received" => NotificationEvent::NFTokenReceived,
            "nf-token-confirmed" => NotificationEvent::NFTokenConfirmed,
            "nf-token-retrieval-outcome" => NotificationEvent::NFTokenRetrievalOutcome,
            "nf-token-transfer-outcome" => NotificationEvent::NFTokenTransferOutcome,
            s @ _ => NotificationEvent::UnknownEvent(String::from(s)),
        }
    }
}

/// Foreign blockchain.
#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ForeignBlockchain {
    /// Ethereum blockchain.
    Ethereum,
    /// Binance Smart Chain blockchain.
    Binance,
    /// Polygon PoS Chain blockchain.
    Polygon,
}

impl ToString for ForeignBlockchain {
    fn to_string(&self) -> String {
        String::from(match self {
            ForeignBlockchain::Ethereum => "ethereum",
            ForeignBlockchain::Binance => "binance",
            ForeignBlockchain::Polygon => "polygon",
        })
    }
}

/// Properties for a new foreign blockchain token.
#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewForeignTokenInfo {
    /// The token name.
    pub name: String,
    /// The token symbol.
    pub symbol: String,
}

/// Foreign blockchain's native coin consumption profile.
#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ForeignConsumptionProfile {
    /// Pay premium to have the foreign blockchain transaction executed as fast as possible.
    Fastest,
    /// Pay a little less but make sure that the foreign blockchain transaction is executed faster
    ///  than most transactions.
    Fast,
    /// Pay an average price to execute the foreign blockchain transaction.
    Average,
    /// Pay just enough to make sure that the foreign blockchain transaction will get executed.
    Slow,
}

/// Option settings for exporting an asset.
#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportAssetOptions {
    /// Name of the foreign blockchain's native coin consumption profile to use.
    ///
    /// > **Note**: if not specified, the value currently set (via Catenis's client admin UI) for
    /// the virtual device's client foreign blockchain consumption profile is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumption_profile: Option<ForeignConsumptionProfile>,
    /// When set, indicates that no asset export should be executed but only the estimated price (in
    ///  the foreign blockchain's native coin) to fulfill the operation should be returned.
    ///
    /// Default value: **`false`**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate_only: Option<bool>,
}

/// Information about an issued foreign blockchain transaction.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ForeignTransactionInfo {
    /// The ID (or hash) of the foreign blockchain transaction.
    pub txid: String,
    /// Indicates whether the foreign blockchain transaction is yet to be executed.
    pub is_pending: bool,
    /// Indicates whether the foreign blockchain transaction has been successfully executed or not.
    ///
    /// > **Note**: only returned after the foreign blockchain transaction is executed.
    pub success: Option<bool>,
    /// An error message describing what went wrong when executing the transaction.
    ///
    /// > **Note**: only returned if the foreign blockchain transaction's execution has failed.
    pub error: Option<String>,
}

/// Information about a foreign blockchain token.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ForeignTokenInfo {
    /// The token name.
    pub name: String,
    /// The token symbol.
    pub symbol: String,
    /// The ID (or address) of the token on the foreign blockchain.
    ///
    /// > **Note**: only returned if the token has been successfully created on the foreign
    /// blockchain.
    pub id: Option<String>,
}

/// Asset export status.
#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AssetExportStatus {
    /// The asset export has not yet reached its final state.
    Pending,
    /// The asset export has been successfully finalized.
    Success,
    /// The asset export has failed.
    Error,
}

impl ToString for AssetExportStatus {
    fn to_string(&self) -> String {
        String::from(match self {
            AssetExportStatus::Pending => "pending",
            AssetExportStatus::Success => "success",
            AssetExportStatus::Error => "error",
        })
    }
}

/// Direction of asset migration.
#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AssetMigrationDirection {
    /// Migrate asset amount from Catenis to the foreign blockchain.
    Outward,
    /// Migrate asset amount from the foreign blockchain back to Catenis.
    Inward,
}

impl ToString for AssetMigrationDirection {
    fn to_string(&self) -> String {
        String::from(match self {
            AssetMigrationDirection::Outward => "outward",
            AssetMigrationDirection::Inward => "inward",
        })
    }
}

/// Information about an asset migration.
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetMigrationInfo {
    /// The direction of the migration.
    pub direction: AssetMigrationDirection,
    /// The amount of the asset to be migrated.
    pub amount: f64,
    /// The address of the account on the foreign blockchain that should be credited with the
    ///  specified amount of the foreign token.
    ///
    /// > **Note**: only required for an out-migration (`direction`:
    /// **`AssetMigrationDirection::Outward`**).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest_address: Option<String>,
}

/// Data type used for referencing an asset migration.
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum AssetMigration {
    /// The asset migration info.
    Info(AssetMigrationInfo),
    /// The asset migration ID.
    ID(String),
}

/// Option settings for exporting an asset.
#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MigrateAssetOptions {
    /// Name of the foreign blockchain's native coin consumption profile to use.
    ///
    /// > **Note**: if not specified, the value currently set (via Catenis's client admin UI) for
    /// the virtual device's client foreign blockchain consumption profile is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumption_profile: Option<ForeignConsumptionProfile>,
    /// When set, indicates that no asset migration should be executed but only the estimated price
    ///  (in the foreign blockchain's native coin) to fulfill the operation should be returned.
    ///
    /// Default value: **`false`**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate_only: Option<bool>,
}

/// Catenis service's execution status.
#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CatenisServiceStatus {
    /// The execution of the Catenis service has not started yet.
    Awaiting,
    /// The Catenis service's execution has failed.
    Failure,
    /// The Catenis service has been successfully executed.
    Fulfilled,
}

/// Information about a Catenis service.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatenisServiceInfo {
    /// The current state of the service's execution.
    pub status: CatenisServiceStatus,
    /// The ID of the Catenis transaction issued to fulfill the service.
    ///
    /// > **Note**: only returned if the service is successfully fulfilled.
    pub txid: Option<String>,
    /// An error message describing what went wrong when executing the service.
    ///
    /// > **Note**: only returned if the service's execution has failed.
    pub error: Option<String>,
}

/// Asset migration status.
#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AssetMigrationStatus {
    /// The asset migration has not yet reached its final state.
    Pending,
    /// Migration started (first step completed successfully) but failed during its second step.
    ///
    /// >**Note**: this represents an inconsistent state, and migration should be retried.
    Interrupted,
    /// The asset migration has been successfully finalized.
    Success,
    /// The asset migration has failed.
    Error,
}

impl ToString for AssetMigrationStatus {
    fn to_string(&self) -> String {
        String::from(match self {
            AssetMigrationStatus::Pending => "pending",
            AssetMigrationStatus::Interrupted => "interrupted",
            AssetMigrationStatus::Success => "success",
            AssetMigrationStatus::Error => "error",
        })
    }
}

/// Information about a listed asset export.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetExportEntry {
    /// The ID of the exported asset.
    pub asset_id: String,
    /// The foreign blockchain to where the asset has been exported.
    pub foreign_blockchain: ForeignBlockchain,
    /// Information about the transaction issued on the foreign blockchain to create the resulting
    ///  foreign token.
    pub foreign_transaction: ForeignTransactionInfo,
    /// Information about the resulting foreign token.
    pub token: ForeignTokenInfo,
    /// The current state of the asset export.
    pub status: AssetExportStatus,
    /// Date and time when the asset has been exported.
    pub date: UtcDateTime,
}

/// Information about a listed asset migration.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetMigrationEntry {
    /// The ID of the asset migration.
    pub migration_id: String,
    /// The ID of the asset the amount of which has been migrated.
    pub asset_id: String,
    /// The foreign blockchain to/from where the asset amount has been migrated.
    pub foreign_blockchain: ForeignBlockchain,
    /// The direction of the migration
    pub direction: AssetMigrationDirection,
    /// The migrated asset amount.
    pub amount: f64,
    /// Information about the execution of the migrate asset Catenis service.
    pub catenis_service: CatenisServiceInfo,
    /// Information about the transaction issued on the foreign blockchain to mint/burn the amount
    ///  of the foreign token..
    pub foreign_transaction: ForeignTransactionInfo,
    /// The current state of the asset migration.
    pub status: AssetMigrationStatus,
    /// Date and time when the asset amount has been migrated.
    pub date: UtcDateTime,
}

/// Options for filtering asset exports.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ListExportedAssetsOptions {
    /// The ID of the exported asset.
    pub asset_id: Option<String>,
    /// The foreign blockchain to where the asset has been exported.
    pub foreign_blockchain: Option<ForeignBlockchain>,
    /// The symbol of the resulting foreign token.
    pub token_symbol: Option<String>,
    /// List with one or more statuses to include.
    pub status: Option<Vec<AssetExportStatus>>,
    /// Indicates whether the specified statuses should be excluded instead.
    ///
    /// Default value: **`false`**.
    pub negate_status: Option<bool>,
    /// Start of time period within which the asset has been exported.
    pub start_date: Option<UtcDateTime>,
    /// End of time period within which the asset has been exported.
    pub end_date: Option<UtcDateTime>,
    /// Maximum number of asset exports that should be returned.
    ///
    /// > **Note**: must be a positive integer value not greater than 500.
    pub limit: Option<u16>,
    /// Number of asset exports that should be skipped.
    ///
    /// > **Note**: must be a non-negative (includes zero) integer value.
    pub skip: Option<usize>,
}

/// Options for filtering asset migrations.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ListAssetMigrationsOptions {
    /// The ID of the asset the amount of which has been migrated.
    pub asset_id: Option<String>,
    /// The foreign blockchain to/from where the asset amount has been migrated.
    pub foreign_blockchain: Option<ForeignBlockchain>,
    /// The direction of the migration.
    pub direction: Option<AssetMigrationDirection>,
    /// List with one or more statuses to include.
    pub status: Option<Vec<AssetMigrationStatus>>,
    /// Indicates whether the specified statuses should be excluded instead.
    ///
    /// Default value: **`false`**.
    pub negate_status: Option<bool>,
    /// Start of time period within which the asset amount has been exported.
    pub start_date: Option<UtcDateTime>,
    /// End of time period within which the asset amount has been exported.
    pub end_date: Option<UtcDateTime>,
    /// Maximum number of asset migrations that should be returned.
    ///
    /// > **Note**: must be a positive integer value not greater than 500.
    pub limit: Option<u16>,
    /// Number of asset asset migrations that should be skipped.
    ///
    /// > **Note**: must be a non-negative (includes zero) integer value.
    pub skip: Option<usize>,
}

/// Properties for a new non-fungible asset.
#[derive(Debug, Serialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct NewNonFungibleAssetInfo {
    /// The non-fungible asset name.
    pub name: String,
    /// A description about the non-fungible asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Indicates whether more non-fungible tokens of that non-fungible asset can be issued at
    /// another time.
    pub can_reissue: bool,
}

/// Properties for issuing a new non-fungible asset.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NonFungibleAssetIssuanceInfo {
    /// Properties of the new non-fungible asset to create.
    ///
    /// > **Note**: should be omitted in a continuation call.
    pub asset_info: Option<NewNonFungibleAssetInfo>,
    /// Indicates whether the contents of the non-fungible tokens being issued should be encrypted
    /// before being stored.
    ///
    /// > **Note**: should be omitted in a continuation call.
    ///
    /// Default value: **`true`**.
    pub encrypt_nft_contents: Option<bool>,
    /// List of virtual devices that will hold the issued non-fungible tokens.
    ///
    /// > **Note**: should be omitted in a continuation call.
    pub holding_devices: Option<Vec<DeviceId>>,
    /// Indicates whether processing should be done asynchronously.
    ///
    /// > **Note**: should be omitted in a continuation call.
    ///
    /// Default value: **`false`**.
    pub async_: Option<bool>,
}

/// Non-fungible asset issuance or continuation token.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum NFAssetIssuanceInfoOrContToken {
    /// Non-fungible asset issuance info.
    IssuanceInfo(NonFungibleAssetIssuanceInfo),
    /// Non-fungible asset issuance continuation token.
    ContinuationToken(String),
}

/// Metadata for a new non-fungible token.
#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewNonFungibleTokenMetadata {
    /// The name of the non-fungible token.
    pub name: String,
    /// A description of the non-fungible token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// User defined, custom properties of the non-fungible token.
    ///
    /// > **Note**: sensitive properties, which are saved in an encrypted form, should be nested
    /// within a special property named `sensitiveProps`.
    ///
    /// > **Tip**: use the [`json_obj`] helper macro to define custom properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<JsonObject>,
}

/// Data type used for defining the contents for a new non-fungible token.
#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewNonFungibleTokenContents {
    /// An additional chunk of data of the non-fungible token's contents.
    pub data: String,
    /// The encoding of the contents data chunk.
    pub encoding: Encoding,
}

/// Properties for a new non-fungible token.
#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewNonFungibleTokenInfo {
    /// The properties of the non-fungible token to issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<NewNonFungibleTokenMetadata>,
    /// The contents of the non-fungible token to issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<NewNonFungibleTokenContents>,
}

/// Properties for reissuing more non-fungible tokens of an existing non-fungible asset.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NonFungibleAssetReissuanceInfo {
    /// Indicates whether the contents of the non-fungible tokens being issued should be encrypted
    /// before being stored.
    ///
    /// > **Note**: should be omitted in a continuation call.
    ///
    /// Default value: **`true`**.
    pub encrypt_nft_contents: Option<bool>,
    /// List of virtual devices that will hold the issued non-fungible tokens.
    ///
    /// > **Note**: should be omitted in a continuation call.
    pub holding_devices: Option<Vec<DeviceId>>,
    /// Indicates whether processing should be done asynchronously.
    ///
    /// > **Note**: should be omitted in a continuation call.
    ///
    /// Default value: **`false`**.
    pub async_: Option<bool>,
}

/// Non-fungible asset reissuance or continuation token.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum NFAssetReissuanceInfoOrContToken {
    /// Non-fungible asset reissuance info.
    ReissuanceInfo(NonFungibleAssetReissuanceInfo),
    /// Non-fungible asset issuance continuation token.
    ContinuationToken(String),
}

/// Information about the error processing a non-fungible asset issuance asynchronously.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFAssetIssuanceProcessError {
    /// Numeric code — equivalent to an HTML status code — of the error that took place while
    /// processing the asset issuance.
    pub code: u16,
    /// Text describing the error that took place while processing the asset issuance.
    pub message: String
}

/// Report on the asynchronous processing status of a non-fungible asset issuance.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFAssetIssuanceProcessProgress {
    /// The percentage of the total processing that has been already completed.
    pub percent_processed: u8,
    /// Indicates whether the processing has been finished.
    pub done: bool,
    /// Indicates whether the asset issuance has been successfully completed.
    ///
    /// > **Note**: only returned if the processing has already been finished.
    pub success: Option<bool>,
    /// Processing error.
    ///
    /// > **Note**: only returned if the processing finished with error.
    pub error: Option<NFAssetIssuanceProcessError>,
    /// Date and time when processing was finalized.
    ///
    /// > **Note**: only returned if the processing has already been finished.
    pub finish_date: Option<UtcDateTime>,
}

/// Result of a non-fungible asset issuance.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFAssetIssuanceResult {
    /// The ID of the newly created non-fungible asset.
    ///
    /// > **Note**: not returned in case of re-issuance.
    pub asset_id: Option<String>,
    /// List of the IDs of the newly issued non-fungible tokens.
    pub nf_token_ids: Vec<String>,
}

/// Option settings for retrieving a non-fungible token.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RetrieveNonFungibleTokenOptions {
    /// Indicates whether the contents of the non-fungible token should be retrieved or not.
    ///
    /// > **Note**: should be omitted in a continuation call.
    ///
    /// Default value: **`true`**.
    pub retrieve_contents: Option<bool>,
    /// Indicates whether only the contents of the non-fungible token should be retrieved.
    ///
    /// > **Note**: should be omitted in a continuation call.
    ///
    /// Default value: **`false`**.
    pub contents_only: Option<bool>,
    /// The encoding with which the retrieved chunk of non-fungible token contents data will be
    /// encoded.
    ///
    /// > **Note**: should be omitted in a continuation call.
    ///
    /// Default value: **`Encoding::Base64`**.
    pub contents_encoding: Option<Encoding>,
    /// Size, in bytes, of the largest chunk of non-fungible token contents data that should be
    /// returned.
    ///
    /// It must be an integer value between 1,024 (1 KB) and 15,728,640 (15 MB).
    ///
    /// > **Note**: should be omitted in a continuation call.
    pub data_chunk_size: Option<usize>,
    /// Indicates whether the processing should be done asynchronously.
    ///
    /// > **Note**: should be omitted in a continuation call.
    ///
    /// Default value: **`false`**.
    pub async_: Option<bool>,
    /// This signals a continuation call of the non-fungible token retrieval.
    ///
    /// It should be filled with the continuation token returned by the previous call.
    pub continuation_token: Option<String>,
}

/// Metadata of a non-fungible token.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NonFungibleTokenMetadata {
    /// The name of the non-fungible token.
    pub name: String,
    /// A description of the non-fungible token.
    pub description: Option<String>,
    /// Indicates whether the stored contents is encrypted.
    pub contents_encrypted: bool,
    /// URL pointing to the non-fungible token's contents stored on IPFS.
    #[serde(rename = "contentsURL")]
    pub contents_url: String,
    /// User defined, custom properties of the non-fungible token.
    ///
    /// > **Note**: sensitive properties, which are saved in an encrypted form, are nested within
    /// a special property named `sensitiveProps`.
    pub custom: Option<JsonObject>,
}

/// Represents a chunk of the contents data of a non-fungible token.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NonFungibleTokenContents {
    /// The text encoded non-fungible token contents data.
    pub data: String,
}

/// Retrieved information about a non-fungible token.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NonFungibleTokenInfo {
    /// The ID of the non-fungible asset to which the non-fungible token belongs.
    ///
    /// > **Note**: not returned for a continuation call or when retrieving only the contents.
    pub asset_id: Option<String>,
    /// The non-fungible token metadata.
    ///
    /// > **Note**: not returned for a continuation call or when retrieving only the contents.
    pub metadata: Option<NonFungibleTokenMetadata>,
    /// The retrieved non-fungible token contents data.
    pub contents: Option<NonFungibleTokenContents>,
}

/// Information about the error processing a non-fungible token retrieval asynchronously.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFTokenRetrievalProcessError {
    /// Numeric code — equivalent to an HTML status code — of the error that took place while
    /// processing the token retrieval.
    pub code: u16,
    /// Text describing the error that took place while processing the token retrieval.
    pub message: String
}

/// Report on the asynchronous processing status of a non-fungible token retrieval.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFTokenRetrievalProcessProgress {
    /// Number of bytes of non-fungible token data that have been retrieved.
    pub bytes_retrieved: usize,
    /// Indicates whether the processing has been finished.
    pub done: bool,
    /// Indicates whether all the non-fungible token data has been successfully retrieved.
    ///
    /// > **Note**: only returned if the processing has already been finished.
    pub success: Option<bool>,
    /// Processing error.
    ///
    /// > **Note**: only returned if the processing finished with error.
    pub error: Option<NFTokenRetrievalProcessError>,
    /// Date and time when data retrieval has been finalized.
    ///
    /// > **Note**: only returned if the processing has already been finished.
    pub finish_date: Option<UtcDateTime>,
}

/// Information about manipulation of non-fungible token data for transfer.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFTokenDataManipulationProgress {
    /// Number of bytes of non-fungible token data that have been read.
    pub bytes_read: usize,
    /// Number of bytes of non-fungible token data that have been written.
    ///
    /// > **Note**: only returned if data needed to be re-encrypted.
    pub bytes_written: Option<usize>,
}

/// Information about the error processing a non-fungible token transfer asynchronously.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFTokenTransferProcessError {
    /// Numeric code — equivalent to an HTML status code — of the error that took place while
    /// processing the token transfer.
    pub code: u16,
    /// Text describing the error that took place while processing the token transfer.
    pub message: String
}

/// Report on the asynchronous processing status of a non-fungible token transfer.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFTokenTransferProcessProgress {
    /// Progress of the non-fungible token data manipulation: reading and rewriting it after
    /// re-encryption (if required).
    pub data_manipulation: NFTokenDataManipulationProgress,
    /// Indicates whether the processing has been finished.
    pub done: bool,
    ///  Indicates whether the non-fungible token has been successfully transferred.
    ///
    /// > **Note**: only returned if the processing has already been finished.
    pub success: Option<bool>,
    /// Processing error.
    ///
    /// > **Note**: only returned if the processing finished with error.
    pub error: Option<NFTokenRetrievalProcessError>,
    /// Date and time when the non-fungible token transfer has been finalized.
    ///
    /// > **Note**: only returned if the processing has already been finished.
    pub finish_date: Option<UtcDateTime>,
}

// Result (used in response) data structures

/// Data returned from a successful call to *Log Message* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogMessageResult {
    /// Token to be used when sending the following message data chunk.
    ///
    /// > **Note**: only returned if passing message in chunks, and last message data chunk was
    /// not final.
    pub continuation_token: Option<String>,
    /// The ID of the logged message.
    ///
    /// > **Note**: only returned after message is fully processed.
    pub message_id: Option<String>,
    /// The ID of the provisional message.
    ///
    /// > **Note**: only returned if doing asynchronous processing, and the whole message's
    /// contents was passed.
    pub provisional_message_id: Option<String>,
}

/// Data returned from a successful call to *Send Message* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResult {
    /// Token to be used when sending the following message data chunk.
    ///
    /// > **Note**: only returned if passing message in chunks, and last message data chunk was not
    /// final.
    pub continuation_token: Option<String>,
    /// The ID of the sent message.
    ///
    /// > **Note**: only returned after message is fully processed.
    pub message_id: Option<String>,
    /// The ID of the provisional message.
    ///
    /// > **Note**: only returned if doing asynchronous processing, and the whole message's
    /// contents was passed.
    pub provisional_message_id: Option<String>,
}

/// Data returned from a successful call to *Read Message* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReadMessageResult {
    /// Info about the read message.
    ///
    /// > **Note**: when reading the message in chunks, this will be returned only for the first
    /// read message data chunk.
    pub msg_info: Option<MessageInfo>,
    /// The message's contents formatted using the specified text encoding.
    ///
    /// > **Note**: when reading the message in chunks, this corresponds to a message data chunk.
    pub msg_data: Option<String>,
    /// Token to be used when requesting the following message data chunk.
    ///
    /// > **Note**: only returned if reading the message in chunks and the whole message's contents
    /// was not read yet.
    pub continuation_token: Option<String>,
    /// The cached message ID.
    ///
    /// > **Note**: only returned if doing asynchronous processing.
    pub cached_message_id: Option<String>,
}

/// Data returned from a successful call to *Retrieve Message Container* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageContainerResult {
    /// Off-chain message container info.
    pub off_chain: Option<OffChainContainer>,
    /// Blockchain container info.
    pub blockchain: Option<BlockchainContainer>,
    /// External storage container info.
    pub external_storage: Option<IpfsStorage>,
}

/// Data returned from a successful call to *Retrieve Message Origin* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageOriginResult {
    /// Blockchain transaction info.
    ///
    /// > **Note**: not returned for off-chain messages not yet settled to the blockchain.
    pub tx: Option<BlockchainTransaction>,
    /// Off-chain message envelope info.
    pub off_chain_msg_envelope: Option<OffChainMsgEnvelope>,
    /// Message origin proof.
    pub proof: Option<ProofInfo>,
}

/// Data returned from a successful call to *Retrieve Message Progress* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveMessageProgressResult {
    /// The action that was to be asynchronously performed on the message.
    pub action: MessageAction,
    /// Asynchronous processing status.
    pub progress: MessageProcessProgress,
    /// Asynchronous processing result.
    pub result: Option<MessageProcessSuccess>,
}

/// Data returned from a successful call to *List Messages* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListMessagesResult {
    /// List of returned messages.
    pub messages: Vec<MessageEntry>,
    /// Number of returned messages.
    pub msg_count: u16,
    /// Indicates whether there are more messages that satisfy the search criteria yet to be
    /// returned.
    pub has_more: bool,
}

/// Data returned from a successful call to *Issue Asset* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IssueAssetResult {
    /// ID of the newly issued asset.
    pub asset_id: String,
}

/// Data returned from a successful call to *Reissue Asset* API method.
#[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReissueAssetResult {
    /// Total balance of the asset in existence after specified amount was issued.
    pub total_existent_balance: f64,
}

/// Data returned from a successful call to *Transfer Asset* API method.
#[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransferAssetResult {
    /// Total balance of the asset still held by the virtual device that made the transfer.
    pub remaining_balance: f64,
}

/// Data returned from a successful call to *Retrieve Asset Info* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveAssetInfoResult {
    /// The asset ID.
    pub asset_id: String,
    /// The asset name.
    pub name: String,
    /// Description about the asset.
    pub description: String,
    /// Indicates whether this is a non-fungible asset.
    pub is_non_fungible: bool,
    /// Indicates whether more units of this asset can be issued.
    pub can_reissue: bool,
    /// The maximum number of decimal places that can be used to specify a fractional amount of
    /// this asset.
    pub decimal_places: u8,
    /// The virtual device that originally issued this asset.
    pub issuer: DeviceInfo,
    /// Current total balance of the asset in existence.
    pub total_existent_balance: f64,
}

/// Data returned from a successful call to *Get Asset Balance* API method.
#[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetBalanceResult {
    /// Current balance of the asset held by the virtual device that made the request.
    pub total: f64,
    /// The amount from the balance that is not yet confirmed.
    pub unconfirmed: f64,
}

/// Data returned from a successful call to *List Owned Assets* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListOwnedAssetsResult {
    /// List of returned owned assets.
    pub owned_assets: Vec<OwnedAssetEntry>,
    /// Indicates whether there are more entries that have not been included in the returned list.
    pub has_more: bool,
}

/// Data returned from a successful call to *List Issued Assets* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListIssuedAssetsResult {
    /// List of returned issued assets.
    pub issued_assets: Vec<IssuedAssetEntry>,
    /// Indicates whether there are more entries that have not been included in the returned list.
    pub has_more: bool,
}

/// Data returned from a successful call to *Retrieve Asset Issuance History* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveAssetIssuanceHistoryResult {
    /// List of returned asset issuance events.
    pub issuance_events: Vec<AssetIssuanceEventEntry>,
    /// Indicates whether there are more asset issuance events that satisfy the search criteria
    /// yet to be returned.
    pub has_more: bool,
}

/// Data returned from a successful call to *List Asset Holders* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListAssetHoldersResult {
    /// List of returned asset holders.
    pub asset_holders: Vec<AssetHolderEntry>,
    /// Indicates whether there are more entries that have not been included in the returned list.
    pub has_more: bool,
}

/// Data returned from a successful call to *Retrieve Permission Rights* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievePermissionRightsResult {
    /// Permission right set at the system level.
    pub system: PermissionRight,
    /// Permission rights set at the Catenis node level.
    pub catenis_node: Option<PermissionRightsSetting>,
    /// Permission rights set at the client level.
    pub client: Option<PermissionRightsSetting>,
    /// Permission rights set at the device level.
    pub device: Option<DevicePermissionRightsSetting>,
}

/// Data returned from a successful call to *Set Permission Rights* API method.
#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetPermissionRightsResult {
    /// Indicates that permission rights have been successfully set.
    pub success: bool,
}

/// Data returned from a successful call to *Retrieve Device Identification Info* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveDeviceIdentificationInfoResult {
    /// Information about the Catenis node where the virtual device's client is defined.
    pub catenis_node: CatenisNodeInfo,
    /// Information about the client to which the virtual device belongs.
    pub client: ClientInfo,
    /// Information about the virtual device itself.
    pub device: DeviceInfo,
}

/// Data returned from a successful call to *Export Asset* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportAssetResult {
    /// Information about the transaction issued on the foreign blockchain to create the resulting
    ///  foreign token.
    pub foreign_transaction: Option<ForeignTransactionInfo>,
    /// Information about the resulting foreign token.
    pub token: Option<ForeignTokenInfo>,
    /// The current state of the asset export.
    pub status: Option<AssetExportStatus>,
    /// Date and time when the asset has been exported.
    pub date: Option<UtcDateTime>,
    /// A text value representing the price, in the foreign blockchain's native coin, required to
    ///  execute the foreign blockchain transaction.
    pub estimated_price: Option<String>,
}

/// Data returned from a successful call to *Migrate Asset* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MigrateAssetResult {
    /// A unique ID used to identify this asset migration.
    pub migration_id: Option<String>,
    /// Information about the execution of the migrate asset Catenis service.
    pub catenis_service: Option<CatenisServiceInfo>,
    /// Information about the transaction issued on the foreign blockchain to mint/burn the amount
    ///  of the foreign token.
    pub foreign_transaction: Option<ForeignTransactionInfo>,
    /// The current state of the asset migration.
    pub status: Option<AssetMigrationStatus>,
    /// Date and time when the asset amount has been migrated.
    pub date: Option<UtcDateTime>,
    /// A text value representing the price, in the foreign blockchain's native coin, required to
    ///  execute the foreign blockchain transaction.
    pub estimated_price: Option<String>,
}

/// Data returned from a successful call to *Asset Export Outcome* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetExportOutcomeResult {
    /// Information about the transaction issued on the foreign blockchain to create the resulting
    ///  foreign token.
    pub foreign_transaction: ForeignTransactionInfo,
    /// Information about the resulting foreign token.
    pub token: ForeignTokenInfo,
    /// The current state of the asset export.
    pub status: AssetExportStatus,
    /// Date and time when the asset has been exported.
    pub date: UtcDateTime,
}

/// Data returned from a successful call to *Asset Migration Outcome* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetMigrationOutcomeResult {
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
    pub foreign_transaction: ForeignTransactionInfo,
    /// The current state of the asset migration.
    pub status: AssetMigrationStatus,
    /// Date and time when the asset amount has been migrated.
    pub date: UtcDateTime,
}

/// Data returned from a successful call to *List Exported Assets* API method.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListExportedAssetsResult {
    /// List of returned issued asset exports.
    ///
    /// > **Note**: the list is sorted in ascending order in regard to the returned `date` field.
    pub exported_assets: Vec<AssetExportEntry>,
    /// Indicates whether there are more asset exports that satisfy the search criteria yet to be
    ///  returned.
    pub has_more: bool,
}

/// Data returned from a successful call to *List Asset Migrations* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListAssetMigrationsResult {
    /// List of returned issued asset migrations.
    ///
    /// > **Note**: the list is sorted in ascending order in regard to the returned `date` field.
    pub asset_migrations: Vec<AssetMigrationEntry>,
    /// Indicates whether there are more asset exports that satisfy the search criteria yet to be
    ///  returned.
    pub has_more: bool,
}

/// Data returned from a successful call to *Issue Non-Fungible Asset* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IssueNonFungibleAssetResult {
    /// The continuation token to be used in the next continuation call.
    ///
    /// > **Note**: only returned for a non-final call.
    pub continuation_token: Option<String>,
    /// The asset issuance ID. Used for retrieving the progress of an asynchronous non-fungible
    /// asset issuance.
    ///
    /// > **Note**: only returned for a final call when doing processing asynchronously.
    pub asset_issuance_id: Option<String>,
    /// The ID of the newly created non-fungible asset.
    ///
    /// > **Note**: only returned for a final call when not doing processing asynchronously.
    pub asset_id: Option<String>,
    /// A list of the IDs of the issued non-fungible tokens.
    ///
    /// > **Note**: only returned for a final call when not doing processing asynchronously.
    pub nf_token_ids: Option<Vec<String>>,
}

/// Data returned from a successful call to *Reissue Non-Fungible Asset* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReissueNonFungibleAssetResult {
    /// The continuation token to be used in the next continuation call.
    ///
    /// > **Note**: only returned for a non-final call.
    pub continuation_token: Option<String>,
    /// The asset issuance ID. Used for retrieving the progress of an asynchronous non-fungible
    /// asset issuance.
    ///
    /// > **Note**: only returned for a final call when doing processing asynchronously.
    pub asset_issuance_id: Option<String>,
    /// A list of the IDs of the issued non-fungible tokens.
    ///
    /// > **Note**: only returned for a final call when not doing processing asynchronously.
    pub nf_token_ids: Option<Vec<String>>,
}

/// Data returned from a successful call to *Retrieve Non-Fungible Asset Issuance Progress* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveNFAssetIssuanceProgressResult {
    /// The ID of the non-fungible asset for which more non-fungible tokens are being issued.
    ///
    /// > **Note**: only returned in case of re-issuance.
    pub asset_id: Option<String>,
    /// Current processing status.
    pub progress: NFAssetIssuanceProcessProgress,
    /// The result of the asset issuance.
    ///
    /// > **Note**: only returned if the processing finished successfully.
    pub result: Option<NFAssetIssuanceResult>,
}

/// Data returned from a successful call to *Retrieve Non-Fungible Token* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveNonFungibleTokenResult {
    /// The continuation token to be used in the next continuation call.
    ///
    /// > **Note**: only returned if there is more data to be retrieved.
    pub continuation_token: Option<String>,
    /// The non-fungible token retrieval ID.
    ///
    /// Used for retrieving the progress of an asynchronous non-fungible token retrieval.
    ///
    /// > **Note**: only returned for the initial call when doing processing asynchronously.
    pub token_retrieval_id: Option<String>,
    /// The retrieved non-fungible token data.
    ///
    /// > **Note**: not returned for the initial call when doing processing asynchronously.
    pub non_fungible_token: Option<NonFungibleTokenInfo>,
}

/// Data returned from a successful call to *Retrieve Non-Fungible Token Retrieval Progress* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveNFTokenRetrievalProgressResult {
    /// Current processing status.
    pub progress: NFTokenRetrievalProcessProgress,
    /// The token that should be used to complete the retrieval of the non-fungible token.
    ///
    /// > **Note**: only returned if the processing finished successfully.
    pub continuation_token: Option<String>,
}

/// Data returned from a successful call to *Transfer Non-Fungible Token* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransferNonFungibleTokenResult {
    /// The non-fungible token transfer ID.
    ///
    /// Used for retrieving the progress of an asynchronous non-fungible token transfer.
    ///
    /// > **Note**: only returned when doing processing asynchronously.
    pub token_transfer_id: Option<String>,
    /// The value **`true`** indicating that the non-fungible token has been successfully
    /// transferred.
    ///
    /// > **Note**: not returned when doing processing asynchronously.
    pub success: Option<bool>,
}

/// Data returned from a successful call to *Retrieve Non-Fungible Token Transfer Progress* API method.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveNFTokenTransferProgressResult {
    /// Current processing status.
    pub progress: NFTokenTransferProcessProgress,
}

// Request data structures

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogMessageRequest {
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<LogMessageOptions>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SendMessageRequest {
    pub message: Message,
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

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportAssetRequest {
    pub token: NewForeignTokenInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ExportAssetOptions>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MigrateAssetRequest {
    pub migration: AssetMigration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<MigrateAssetOptions>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssueNonFungibleAssetRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_info: Option<NewNonFungibleAssetInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "encryptNFTContents")]
    /// Default value: **`true`**.
    pub encrypt_nft_contents: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holding_devices: Option<Vec<DeviceId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "async")]
    /// Default value: **`false`**.
    pub async_: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_fungible_tokens: Option<Vec<Option<NewNonFungibleTokenInfo>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Default value: **`true`**.
    pub is_final: Option<bool>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReissueNonFungibleAssetRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "encryptNFTContents")]
    /// Default value: **`true`**.
    pub encrypt_nft_contents: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holding_devices: Option<Vec<DeviceId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "async")]
    /// Default value: **`false`**.
    pub async_: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_fungible_tokens: Option<Vec<Option<NewNonFungibleTokenInfo>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Default value: **`true`**.
    pub is_final: Option<bool>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferNonFungibleTokenRequest {
    pub receiving_device: DeviceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "async")]
    /// Default value: **`false`**.
    pub async_: Option<bool>,
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

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportAssetResponse {
    pub status: String,
    pub data: ExportAssetResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MigrateAssetResponse {
    pub status: String,
    pub data: MigrateAssetResult,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetExportOutcomeResponse {
    pub status: String,
    pub data: AssetExportOutcomeResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetMigrationOutcomeResponse {
    pub status: String,
    pub data: AssetMigrationOutcomeResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListExportedAssetsResponse {
    pub status: String,
    pub data: ListExportedAssetsResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListAssetMigrationsResponse {
    pub status: String,
    pub data: ListAssetMigrationsResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssueNonFungibleAssetResponse {
    pub status: String,
    pub data: IssueNonFungibleAssetResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReissueNonFungibleAssetResponse {
    pub status: String,
    pub data: ReissueNonFungibleAssetResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RetrieveNFAssetIssuanceProgressResponse {
    pub status: String,
    pub data: RetrieveNFAssetIssuanceProgressResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RetrieveNonFungibleTokenResponse {
    pub status: String,
    pub data: RetrieveNonFungibleTokenResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RetrieveNFTokenRetrievalProgressResponse {
    pub status: String,
    pub data: RetrieveNFTokenRetrievalProgressResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferNonFungibleTokenResponse {
    pub status: String,
    pub data: TransferNonFungibleTokenResult,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RetrieveNFTokenTransferProgressResponse {
    pub status: String,
    pub data: RetrieveNFTokenTransferProgressResult,
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
    fn it_serialize_message_whole() {
        let message = Message::Whole(String::from("Test message"));

        let json = serde_json::to_string(&message).unwrap();

        assert_eq!(json, r#""Test message""#);
    }

    #[test]
    fn it_serialize_message_chunk() {
        let message = Message::Chunk(ChunkedMessage {
            data: Some(String::from("Test message chunk #1")),
            is_final: Some(false),
            continuation_token: Some(String::from("kYkeYcQN2YJdcJLNSEWq")),
        });

        let json = serde_json::to_string(&message).unwrap();

        assert_eq!(json, r#"{"data":"Test message chunk #1","isFinal":false,"continuationToken":"kYkeYcQN2YJdcJLNSEWq"}"#);
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
        let json = r#"{"txid":"505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7","isConfirmed":true}"#;

        let blockchain_container: BlockchainContainer = serde_json::from_str(json).unwrap();

        assert_eq!(blockchain_container, BlockchainContainer {
            txid: String::from("505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7"),
            is_confirmed: true,
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
    fn it_deserialize_regular_asset_issuance_event_entry() {
        let json = r#"{"amount":123,"holdingDevice":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"date":"2020-12-23T10:51:45.935Z"}"#;

        let regular_asset_issuance_event_entry: RegularAssetIssuanceEventEntry = serde_json::from_str(json).unwrap();

        assert_eq!(regular_asset_issuance_event_entry, RegularAssetIssuanceEventEntry {
            amount: 123.0,
            holding_device: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
            date: "2020-12-23T10:51:45.935Z".into(),
        });
    }

    #[test]
    fn it_deserialize_non_fungible_asset_issuance_event_entry() {
        let json = r#"{"nfTokenIds":["tQyJrga3ke65RR23iyr2","tf2rbknDoo9wPsKBkskj"],"holdingDevices":[{"deviceId":"d8YpQ7jgPBJEkBrnvp58"}],"date":"2020-12-24T13:27:02.010Z"}"#;

        let non_fungible_asset_issuance_event_entry: NonFungibleAssetIssuanceEventEntry = serde_json::from_str(json).unwrap();

        assert_eq!(non_fungible_asset_issuance_event_entry, NonFungibleAssetIssuanceEventEntry {
            nf_token_ids: vec![
                String::from("tQyJrga3ke65RR23iyr2"),
                String::from("tf2rbknDoo9wPsKBkskj"),
            ],
            holding_devices: vec![
                DeviceInfo {
                    device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                    name: None,
                    prod_unique_id: None,
                },
            ],
            date: "2020-12-24T13:27:02.010Z".into(),
        });
    }

    #[test]
    fn it_deserialize_asset_issuance_event_entry_regular() {
        let json = r#"{"amount":123,"holdingDevice":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"date":"2020-12-23T10:51:45.935Z"}"#;

        let asset_issuance_event_entry: AssetIssuanceEventEntry = serde_json::from_str(json).unwrap();

        assert_eq!(asset_issuance_event_entry, AssetIssuanceEventEntry::Regular(RegularAssetIssuanceEventEntry {
            amount: 123.0,
            holding_device: DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            },
            date: "2020-12-23T10:51:45.935Z".into(),
        }));
    }

    #[test]
    fn it_deserialize_asset_issuance_event_entry_non_fungible() {
        let json = r#"{"nfTokenIds":["tQyJrga3ke65RR23iyr2","tf2rbknDoo9wPsKBkskj"],"holdingDevices":[{"deviceId":"d8YpQ7jgPBJEkBrnvp58"}],"date":"2020-12-24T13:27:02.010Z"}"#;

        let asset_issuance_event_entry: AssetIssuanceEventEntry = serde_json::from_str(json).unwrap();

        assert_eq!(asset_issuance_event_entry, AssetIssuanceEventEntry::NonFungible(NonFungibleAssetIssuanceEventEntry {
            nf_token_ids: vec![
                String::from("tQyJrga3ke65RR23iyr2"),
                String::from("tf2rbknDoo9wPsKBkskj"),
            ],
            holding_devices: vec![
                DeviceInfo {
                    device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                    name: None,
                    prod_unique_id: None,
                },
            ],
            date: "2020-12-24T13:27:02.010Z".into(),
        }));
    }

    #[test]
    fn it_deserialize_asset_holder_entry() {
        let json = r#"{"holder":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"balance":{"total":123.25,"unconfirmed":0}}"#;

        let asset_holder_entry: AssetHolderEntry = serde_json::from_str(json).unwrap();

        assert_eq!(asset_holder_entry, AssetHolderEntry {
            holder: Some(DeviceInfo {
                device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                name: None,
                prod_unique_id: None,
            }),
            migrated: None,
            balance: AssetBalance {
                total: 123.25,
                unconfirmed: 0.0,
            },
        });
    }

    #[test]
    fn it_deserialize_migrated_asset_holder_entry() {
        let json = r#"{"migrated":true,"balance":{"total":34.75,"unconfirmed":0}}"#;

        let asset_holder_entry: AssetHolderEntry = serde_json::from_str(json).unwrap();

        assert_eq!(asset_holder_entry, AssetHolderEntry {
            holder: None,
            migrated: Some(true),
            balance: AssetBalance {
                total: 34.75,
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
        let json = r#"{"ctnNodeIndex":0}"#;

        let catenis_node_info: CatenisNodeInfo = serde_json::from_str(json).unwrap();

        assert_eq!(catenis_node_info, CatenisNodeInfo {
            ctn_node_index: 0,
            name: None,
            description: None,
        });
    }

    #[test]
    fn it_deserialize_catenis_node_info_all_opts() {
        let json = r#"{"ctnNodeIndex":0,"name":"Catenis Hub","description":"Central Catenis node used to house clients that access the system through the Internet"}"#;

        let catenis_node_info: CatenisNodeInfo = serde_json::from_str(json).unwrap();

        assert_eq!(catenis_node_info, CatenisNodeInfo {
            ctn_node_index: 0,
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
    fn it_deserialize_foreign_blockchain() {
        let json = r#""polygon""#;

        let foreign_blockchain: ForeignBlockchain = serde_json::from_str(json).unwrap();

        assert_eq!(foreign_blockchain, ForeignBlockchain::Polygon);
    }

    #[test]
    fn it_serialize_foreign_blockchain() {
        let foreign_blockchain = ForeignBlockchain::Ethereum;

        let json = serde_json::to_string(&foreign_blockchain).unwrap();

        assert_eq!(json, r#""ethereum""#);
    }

    #[test]
    fn it_convert_foreign_blockchain() {
        let foreign_blockchain_str = ForeignBlockchain::Binance.to_string();

        assert_eq!(foreign_blockchain_str, "binance");
    }

    #[test]
    fn it_serialize_new_foreign_token_info() {
        let new_foreign_token_info = NewForeignTokenInfo {
            name: String::from("Catenis test token #10"),
            symbol: String::from("CTK10"),
        };

        let json = serde_json::to_string(&new_foreign_token_info).unwrap();

        assert_eq!(json, r#"{"name":"Catenis test token #10","symbol":"CTK10"}"#);
    }

    #[test]
    fn it_serialize_foreign_consumption_profile() {
        let foreign_consumption_profile = ForeignConsumptionProfile::Average;

        let json = serde_json::to_string(&foreign_consumption_profile).unwrap();

        assert_eq!(json, r#""average""#);
    }

    #[test]
    fn it_serialize_export_asset_options_no_opts() {
        let export_asset_options = ExportAssetOptions {
            consumption_profile: None,
            estimate_only: None,
        };

        let json = serde_json::to_string(&export_asset_options).unwrap();

        assert_eq!(json, r#"{}"#);
    }

    #[test]
    fn it_serialize_export_asset_options_all_opts() {
        let export_asset_options = ExportAssetOptions {
            consumption_profile: Some(ForeignConsumptionProfile::Fast),
            estimate_only: Some(true),
        };

        let json = serde_json::to_string(&export_asset_options).unwrap();

        assert_eq!(json, r#"{"consumptionProfile":"fast","estimateOnly":true}"#);
    }

    #[test]
    fn it_deserialize_foreign_transaction_info_no_opts() {
        let json = r#"{"txid":"0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a","isPending":true}"#;

        let foreign_transaction_info: ForeignTransactionInfo = serde_json::from_str(json).unwrap();

        assert_eq!(foreign_transaction_info, ForeignTransactionInfo {
            txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
            is_pending: true,
            success: None,
            error: None
        });
    }

    #[test]
    fn it_deserialize_foreign_transaction_info_all_opts() {
        let json = r#"{"txid":"0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a","isPending":false,"success":false,"error":"Simulated error"}"#;

        let foreign_transaction_info: ForeignTransactionInfo = serde_json::from_str(json).unwrap();

        assert_eq!(foreign_transaction_info, ForeignTransactionInfo {
            txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
            is_pending: false,
            success: Some(false),
            error: Some(String::from("Simulated error")),
        });
    }

    #[test]
    fn it_deserialize_foreign_token_info_no_opts() {
        let json = r#"{"name":"Catenis test token #10","symbol":"CTK10"}"#;

        let foreign_token_info: ForeignTokenInfo = serde_json::from_str(json).unwrap();

        assert_eq!(foreign_token_info, ForeignTokenInfo {
            name: String::from("Catenis test token #10"),
            symbol: String::from("CTK10"),
            id: None
        });
    }

    #[test]
    fn it_deserialize_foreign_token_info_all_opts() {
        let json = r#"{"name":"Catenis test token #10","symbol":"CTK10","id":"0x537580164Ba9DB2e8C254a38E254ce15d07fDef9"}"#;

        let foreign_token_info: ForeignTokenInfo = serde_json::from_str(json).unwrap();

        assert_eq!(foreign_token_info, ForeignTokenInfo {
            name: String::from("Catenis test token #10"),
            symbol: String::from("CTK10"),
            id: Some(String::from("0x537580164Ba9DB2e8C254a38E254ce15d07fDef9")),
        });
    }

    #[test]
    fn it_deserialize_asset_export_status() {
        let json = r#""pending""#;

        let asset_export_status: AssetExportStatus = serde_json::from_str(json).unwrap();

        assert_eq!(asset_export_status, AssetExportStatus::Pending);
    }

    #[test]
    fn it_serialize_asset_export_status() {
        let asset_export_status = AssetExportStatus::Success;

        let json = serde_json::to_string(&asset_export_status).unwrap();

        assert_eq!(json, r#""success""#);
    }

    #[test]
    fn it_convert_asset_export_status() {
        let asset_export_status_str = AssetExportStatus::Error.to_string();

        assert_eq!(asset_export_status_str, "error");
    }

    #[test]
    fn it_deserialize_asset_migration_direction() {
        let json = r#""outward""#;

        let asset_migration_direction: AssetMigrationDirection = serde_json::from_str(json).unwrap();

        assert_eq!(asset_migration_direction, AssetMigrationDirection::Outward);
    }

    #[test]
    fn it_serialize_asset_migration_direction() {
        let asset_migration_direction = AssetMigrationDirection::Inward;

        let json = serde_json::to_string(&asset_migration_direction).unwrap();

        assert_eq!(json, r#""inward""#);
    }

    #[test]
    fn it_convert_asset_migration_direction() {
        let asset_migration_direction_str = AssetMigrationDirection::Outward.to_string();

        assert_eq!(asset_migration_direction_str, "outward");
    }

    #[test]
    fn it_serialize_asset_migration_info_no_opts() {
        let asset_migration_info = AssetMigrationInfo {
            direction: AssetMigrationDirection::Outward,
            amount: 123.45,
            dest_address: None,
        };

        let json = serde_json::to_string(&asset_migration_info).unwrap();

        assert_eq!(json, r#"{"direction":"outward","amount":123.45}"#);
    }

    #[test]
    fn it_serialize_asset_migration_info_all_opts() {
        let asset_migration_info = AssetMigrationInfo {
            direction: AssetMigrationDirection::Outward,
            amount: 123.45,
            dest_address: Some(String::from("0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943")),
        };

        let json = serde_json::to_string(&asset_migration_info).unwrap();

        assert_eq!(json, r#"{"direction":"outward","amount":123.45,"destAddress":"0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943"}"#);
    }

    #[test]
    fn it_serialize_migration_info_variant() {
        let migration = AssetMigration::Info(AssetMigrationInfo {
            direction: AssetMigrationDirection::Outward,
            amount: 123.45,
            dest_address: Some(String::from("0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943")),
        });

        let json = serde_json::to_string(&migration).unwrap();

        assert_eq!(json, r#"{"direction":"outward","amount":123.45,"destAddress":"0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943"}"#);
    }

    #[test]
    fn it_serialize_migration_id_variant() {
        let migration = AssetMigration::ID(String::from("gSLb9FTdGxgSLufuNzhR"));

        let json = serde_json::to_string(&migration).unwrap();

        assert_eq!(json, r#""gSLb9FTdGxgSLufuNzhR""#);
    }

    #[test]
    fn it_serialize_migrate_asset_options_no_opts() {
        let migrate_asset_options = MigrateAssetOptions {
            consumption_profile: None,
            estimate_only: None,
        };

        let json = serde_json::to_string(&migrate_asset_options).unwrap();

        assert_eq!(json, r#"{}"#);
    }

    #[test]
    fn it_serialize_migrate_asset_options_all_opts() {
        let migrate_asset_options = MigrateAssetOptions {
            consumption_profile: Some(ForeignConsumptionProfile::Fastest),
            estimate_only: Some(true),
        };

        let json = serde_json::to_string(&migrate_asset_options).unwrap();

        assert_eq!(json, r#"{"consumptionProfile":"fastest","estimateOnly":true}"#);
    }

    #[test]
    fn it_deserialize_catenis_service_status() {
        let json = r#""awaiting""#;

        let catenis_service_status: CatenisServiceStatus = serde_json::from_str(json).unwrap();

        assert_eq!(catenis_service_status, CatenisServiceStatus::Awaiting);
    }

    #[test]
    fn it_deserialize_catenis_service_info_no_opts() {
        let json = r#"{"status":"awaiting"}"#;

        let catenis_service_info: CatenisServiceInfo = serde_json::from_str(json).unwrap();

        assert_eq!(catenis_service_info, CatenisServiceInfo {
            status: CatenisServiceStatus::Awaiting,
            txid: None,
            error: None,
        });
    }

    #[test]
    fn it_deserialize_catenis_service_info_all_opts() {
        let json = r#"{"status":"fulfilled","txid":"61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf","error":"Simulated error"}"#;

        let catenis_service_info: CatenisServiceInfo = serde_json::from_str(json).unwrap();

        assert_eq!(catenis_service_info, CatenisServiceInfo {
            status: CatenisServiceStatus::Fulfilled,
            txid: Some(String::from("61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf")),
            error: Some(String::from("Simulated error")),
        });
    }

    #[test]
    fn it_deserialize_asset_migration_status() {
        let json = r#""pending""#;

        let asset_migration_status: AssetMigrationStatus = serde_json::from_str(json).unwrap();

        assert_eq!(asset_migration_status, AssetMigrationStatus::Pending);
    }

    #[test]
    fn it_serialize_asset_migration_status() {
        let asset_migration_status = AssetMigrationStatus::Interrupted;

        let json = serde_json::to_string(&asset_migration_status).unwrap();

        assert_eq!(json, r#""interrupted""#);
    }

    #[test]
    fn it_convert_asset_migration_status() {
        let asset_migration_status_str = AssetMigrationStatus::Success.to_string();

        assert_eq!(asset_migration_status_str, "success");
    }

    #[test]
    fn it_deserialize_asset_export_entry() {
        let json = r#"{"assetId": "aH2AkrrL55GcThhPNa3J","foreignBlockchain": "ethereum","foreignTransaction": {"txid": "0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a","isPending": false,"success": true},"token": {"name": "Catenis test token #10","symbol": "CTK10","id": "0x537580164Ba9DB2e8C254a38E254ce15d07fDef9"},"status": "success","date": "2021-08-03T18:41:27.679Z"}"#;

        let asset_export_entry: AssetExportEntry = serde_json::from_str(json).unwrap();

        assert_eq!(asset_export_entry, AssetExportEntry {
            asset_id: String::from("aH2AkrrL55GcThhPNa3J"),
            foreign_blockchain: ForeignBlockchain::Ethereum,
            foreign_transaction: ForeignTransactionInfo {
                txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
                is_pending: false,
                success: Some(true),
                error: None,
            },
            token: ForeignTokenInfo {
                name: String::from("Catenis test token #10"),
                symbol: String::from("CTK10"),
                id: Some(String::from("0x537580164Ba9DB2e8C254a38E254ce15d07fDef9")),
            },
            status: AssetExportStatus::Success,
            date: "2021-08-03T18:41:27.679Z".into(),
        });
    }

    #[test]
    fn it_deserialize_asset_migration_entry() {
        let json = r#"{"migrationId": "gSLb9FTdGxgSLufuNzhR","assetId": "aH2AkrrL55GcThhPNa3J","foreignBlockchain": "ethereum","direction": "inward","amount": 4,"catenisService": {"status": "fulfilled","txid": "26d45a275447caf36e0fbcc32f880f37d3aadb37ddceccc39cd8972a7933e3f4"},"foreignTransaction": {"txid": "0x883a4d9e02713b177fdd26b33e871dc765db3c964f2b1ef8e6f97eca24d718ee","isPending": false,"success": true},"status": "success","date": "2021-08-03T19:11:27.804Z"}"#;

        let asset_migration_entry: AssetMigrationEntry = serde_json::from_str(json).unwrap();

        assert_eq!(asset_migration_entry, AssetMigrationEntry {
            migration_id: String::from("gSLb9FTdGxgSLufuNzhR"),
            asset_id: String::from("aH2AkrrL55GcThhPNa3J"),
            foreign_blockchain: ForeignBlockchain::Ethereum,
            direction: AssetMigrationDirection::Inward,
            amount: 4.0,
            catenis_service: CatenisServiceInfo {
                status: CatenisServiceStatus::Fulfilled,
                txid: Some(String::from("26d45a275447caf36e0fbcc32f880f37d3aadb37ddceccc39cd8972a7933e3f4")),
                error: None,
            },
            foreign_transaction: ForeignTransactionInfo {
                txid: String::from("0x883a4d9e02713b177fdd26b33e871dc765db3c964f2b1ef8e6f97eca24d718ee"),
                is_pending: false,
                success: Some(true),
                error: None,
            },
            status: AssetMigrationStatus::Success,
            date: "2021-08-03T19:11:27.804Z".into(),
        });
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
        let json = r#"{"offChain":{"cid":"QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"},"blockchain":{"txid":"505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7","isConfirmed":true},"externalStorage":{"ipfs":"Qmd2FBqC4dGTbiNe7HyYEJR2grpLWPzDNfBgajkDNRhK1X"}}"#;

        let retrieve_message_container_result: RetrieveMessageContainerResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_container_result, RetrieveMessageContainerResult {
            off_chain: Some(OffChainContainer {
                cid: String::from("QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"),
            }),
            blockchain: Some(BlockchainContainer {
                txid: String::from("505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7"),
                is_confirmed: true,
            }),
            external_storage: Some(IpfsStorage {
                ipfs: String::from("Qmd2FBqC4dGTbiNe7HyYEJR2grpLWPzDNfBgajkDNRhK1X"),
            }),
        });
    }

    #[test]
    fn it_deserialize_retrieve_message_origin_result_no_opts() {
        let json = r#"{}"#;

        let retrieve_message_origin_result: RetrieveMessageOriginResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_origin_result, RetrieveMessageOriginResult {
            tx: None,
            off_chain_msg_envelope: None,
            proof: None,
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
            proof: Some(ProofInfo {
                message: String::from("This is only a test"),
                signature: String::from("IEn9KpwdIitSilCuERUz5Dg+siFe+tW7Bh4ezHDer5NBCQD1jfWJgYL2SnKnzBnGbay/WXL7eykuK8N4o4gRLNo=")
            }),
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
        let json = r#"{"assetId":"aQjlzShmrnEZeeYBZihc","name":"TestAsset_1","description":"First asset issued for test","isNonFungible":false,"canReissue":false,"decimalPlaces":2,"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"totalExistentBalance":123.25}"#;

        let retrieve_asset_info_result: RetrieveAssetInfoResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_asset_info_result, RetrieveAssetInfoResult {
            asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
            name: String::from("TestAsset_1"),
            description: String::from("First asset issued for test"),
            is_non_fungible: false,
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
        let json = r#"{"issuanceEvents":[{"amount":123,"holdingDevice":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"date":"2020-12-23T10:51:45.935Z"},{"nfTokenIds":["tQyJrga3ke65RR23iyr2","tf2rbknDoo9wPsKBkskj"],"holdingDevices":[{"deviceId":"d8YpQ7jgPBJEkBrnvp58"}],"date":"2020-12-24T13:27:02.010Z"}],"hasMore":false}"#;

        let retrieve_asset_issuance_history_result: RetrieveAssetIssuanceHistoryResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_asset_issuance_history_result, RetrieveAssetIssuanceHistoryResult {
            issuance_events: vec![
                AssetIssuanceEventEntry::Regular(RegularAssetIssuanceEventEntry {
                    amount: 123.0,
                    holding_device: DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: None,
                        prod_unique_id: None,
                    },
                    date: "2020-12-23T10:51:45.935Z".into(),
                }),
                AssetIssuanceEventEntry::NonFungible(NonFungibleAssetIssuanceEventEntry {
                    nf_token_ids: vec![
                        String::from("tQyJrga3ke65RR23iyr2"),
                        String::from("tf2rbknDoo9wPsKBkskj"),
                    ],
                    holding_devices: vec![
                        DeviceInfo {
                            device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                            name: None,
                            prod_unique_id: None,
                        },
                    ],
                    date: "2020-12-24T13:27:02.010Z".into(),
                }),
            ],
            has_more: false,
        });
    }

    #[test]
    fn it_deserialize_list_asset_holders_result() {
        let json = r#"{"assetHolders":[{"holder":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"balance":{"total":123.25,"unconfirmed":0}},{"holder":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"},"balance":{"total":150,"unconfirmed":0}},{"migrated":true,"balance":{"total":34.75,"unconfirmed":0}}],"hasMore":false}"#;

        let list_asset_holders_result: ListAssetHoldersResult = serde_json::from_str(json).unwrap();

        assert_eq!(list_asset_holders_result, ListAssetHoldersResult {
            asset_holders: vec![
                AssetHolderEntry {
                    holder: Some(DeviceInfo {
                        device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                        name: None,
                        prod_unique_id: None,
                    }),
                    migrated: None,
                    balance: AssetBalance {
                        total: 123.25,
                        unconfirmed: 0.0,
                    },
                },
                AssetHolderEntry {
                    holder: Some(DeviceInfo {
                        device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                        name: None,
                        prod_unique_id: None,
                    }),
                    migrated: None,
                    balance: AssetBalance {
                        total: 150.0,
                        unconfirmed: 0.0,
                    },
                },
                AssetHolderEntry {
                    holder: None,
                    migrated: Some(true),
                    balance: AssetBalance {
                        total: 34.75,
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
            ].into_iter()).collect(),
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
            ].into_iter()).collect(),
        );
    }

    #[test]
    fn it_deserialize_retrieve_device_identification_info_result() {
        let json = r#"{"catenisNode":{"ctnNodeIndex":0},"client":{"clientId":"cEXd845DSMw9g6tM5dhy"},"device":{"deviceId":"drc3XdxNtzoucpw9xiRp"}}"#;

        let retrieve_device_identification_info_result: RetrieveDeviceIdentificationInfoResult = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_device_identification_info_result, RetrieveDeviceIdentificationInfoResult {
            catenis_node: CatenisNodeInfo {
                ctn_node_index: 0,
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
            ].into_iter()).collect(),
        );
    }

    #[test]
    fn it_deserialize_export_asset_result_no_opts() {
        let json = r#"{}"#;

        let export_asset_result: ExportAssetResult = serde_json::from_str(json).unwrap();

        assert_eq!(export_asset_result, ExportAssetResult {
            foreign_transaction: None,
            token: None,
            status: None,
            date: None,
            estimated_price: None,
        });
    }

    #[test]
    fn it_deserialize_export_asset_result_all_opts() {
        let json = r#"{"foreignTransaction":{"txid":"0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a","isPending":true},"token":{"name":"Catenis test token #10","symbol":"CTK10"},"status":"pending","date":"2021-08-03T18:41:11.781Z","estimatedPrice":"0.05850782"}"#;

        let export_asset_result: ExportAssetResult = serde_json::from_str(json).unwrap();

        assert_eq!(export_asset_result, ExportAssetResult {
            foreign_transaction: Some(ForeignTransactionInfo {
                txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
                is_pending: true,
                success: None,
                error: None,
            }),
            token: Some(ForeignTokenInfo {
                name: String::from("Catenis test token #10"),
                symbol: String::from("CTK10"),
                id: None,
            }),
            status: Some(AssetExportStatus::Pending),
            date: Some("2021-08-03T18:41:11.781Z".into()),
            estimated_price: Some(String::from("0.05850782")),
        });
    }

    #[test]
    fn it_deserialize_migrate_asset_result_no_opts() {
        let json = r#"{}"#;

        let migrate_asset_result: MigrateAssetResult = serde_json::from_str(json).unwrap();

        assert_eq!(migrate_asset_result, MigrateAssetResult {
            migration_id: None,
            catenis_service: None,
            foreign_transaction: None,
            status: None,
            date: None,
            estimated_price: None,
        });
    }

    #[test]
    fn it_deserialize_migrate_asset_result_all_opts() {
        let json = r#"{"migrationId":"gq8x3efLpEXTkGQchHTb","catenisService":{"status":"fulfilled","txid":"61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf"},"foreignTransaction":{"txid":"0x212ab54f136a6fc1deae9ec217ef2d0417615178777131e8bb6958447fd20fe7","isPending":true},"status":"pending","date":"2021-08-03T18:51:26.631Z","estimatedPrice":"0.001723913"}"#;

        let migrate_asset_result: MigrateAssetResult = serde_json::from_str(json).unwrap();

        assert_eq!(migrate_asset_result, MigrateAssetResult {
            migration_id: Some(String::from("gq8x3efLpEXTkGQchHTb")),
            catenis_service: Some(CatenisServiceInfo {
                status: CatenisServiceStatus::Fulfilled,
                txid: Some(String::from("61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf")),
                error: None,
            }),
            foreign_transaction: Some(ForeignTransactionInfo {
                txid: String::from("0x212ab54f136a6fc1deae9ec217ef2d0417615178777131e8bb6958447fd20fe7"),
                is_pending: true,
                success: None,
                error: None,
            }),
            status: Some(AssetMigrationStatus::Pending),
            date: Some("2021-08-03T18:51:26.631Z".into()),
            estimated_price: Some(String::from("0.001723913")),
        });
    }

    #[test]
    fn it_deserialize_asset_export_outcome_result() {
        let json = r#"{"foreignTransaction":{"txid":"0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a","isPending":false,"success":true},"token":{"name":"Catenis test token #10","symbol":"CTK10","id":"0x537580164Ba9DB2e8C254a38E254ce15d07fDef9"},"status":"success","date":"2021-08-03T18:41:27.679Z"}"#;

        let asset_export_outcome_result: AssetExportOutcomeResult = serde_json::from_str(json).unwrap();

        assert_eq!(asset_export_outcome_result, AssetExportOutcomeResult {
            foreign_transaction: ForeignTransactionInfo {
                txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
                is_pending: false,
                success: Some(true),
                error: None,
            },
            token: ForeignTokenInfo {
                name: String::from("Catenis test token #10"),
                symbol: String::from("CTK10"),
                id: Some(String::from("0x537580164Ba9DB2e8C254a38E254ce15d07fDef9")),
            },
            status: AssetExportStatus::Success,
            date: "2021-08-03T18:41:27.679Z".into(),
        });
    }

    #[test]
    fn it_deserialize_asset_migration_outcome_result() {
        let json = r#"{"assetId":"aH2AkrrL55GcThhPNa3J","foreignBlockchain":"ethereum","direction":"outward","amount":10,"catenisService":{"status":"fulfilled","txid":"61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf"},"foreignTransaction":{"txid":"0x212ab54f136a6fc1deae9ec217ef2d0417615178777131e8bb6958447fd20fe7","isPending":false,"success":true},"status":"success","date":"2021-08-03T18:51:55.591Z"}"#;

        let asset_migration_outcome_result: AssetMigrationOutcomeResult = serde_json::from_str(json).unwrap();

        assert_eq!(asset_migration_outcome_result, AssetMigrationOutcomeResult {
            asset_id: String::from("aH2AkrrL55GcThhPNa3J"),
            foreign_blockchain: ForeignBlockchain::Ethereum,
            direction: AssetMigrationDirection::Outward,
            amount: 10.0,
            catenis_service: CatenisServiceInfo {
                status: CatenisServiceStatus::Fulfilled,
                txid: Some(String::from("61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf")),
                error: None,
            },
            foreign_transaction: ForeignTransactionInfo {
                txid: String::from("0x212ab54f136a6fc1deae9ec217ef2d0417615178777131e8bb6958447fd20fe7"),
                is_pending: false,
                success: Some(true),
                error: None,
            },
            status: AssetMigrationStatus::Success,
            date: "2021-08-03T18:51:55.591Z".into(),
        });
    }

    #[test]
    fn it_deserialize_list_exported_assets_result() {
        let json = r#"{"exportedAssets":[{"assetId":"aH2AkrrL55GcThhPNa3J","foreignBlockchain":"ethereum","foreignTransaction":{"txid":"0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a","isPending":false,"success":true},"token":{"name":"Catenis test token #10","symbol":"CTK10","id":"0x537580164Ba9DB2e8C254a38E254ce15d07fDef9"},"status":"success","date":"2021-08-03T18:41:27.679Z"},{"assetId":"aCSy24HLjKMbpnvJ8GTx","foreignBlockchain":"ethereum","foreignTransaction":{"txid":"0x6299c35ccfa803ab0cb043e8d8ae4be8d7f3432d85f288ebb81e4d624e566b0a","isPending":false,"success":true},"token":{"name":"Catenis test token #11","symbol":"CTK11","id":"0x5cE78E7204DD8f7d86142fAaA694d5354b997600"},"status":"success","date":"2021-08-10T12:57:24.217Z"}],"hasMore":false}"#;

        let list_exported_assets_result: ListExportedAssetsResult = serde_json::from_str(json).unwrap();

        assert_eq!(list_exported_assets_result, ListExportedAssetsResult {
            exported_assets: vec![
                AssetExportEntry {
                    asset_id: String::from("aH2AkrrL55GcThhPNa3J"),
                    foreign_blockchain: ForeignBlockchain::Ethereum,
                    foreign_transaction: ForeignTransactionInfo {
                        txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
                        is_pending: false,
                        success: Some(true),
                        error: None,
                    },
                    token: ForeignTokenInfo {
                        name: String::from("Catenis test token #10"),
                        symbol: String::from("CTK10"),
                        id: Some(String::from("0x537580164Ba9DB2e8C254a38E254ce15d07fDef9")),
                    },
                    status: AssetExportStatus::Success,
                    date: "2021-08-03T18:41:27.679Z".into(),
                },
                AssetExportEntry {
                    asset_id: String::from("aCSy24HLjKMbpnvJ8GTx"),
                    foreign_blockchain: ForeignBlockchain::Ethereum,
                    foreign_transaction: ForeignTransactionInfo {
                        txid: String::from("0x6299c35ccfa803ab0cb043e8d8ae4be8d7f3432d85f288ebb81e4d624e566b0a"),
                        is_pending: false,
                        success: Some(true),
                        error: None,
                    },
                    token: ForeignTokenInfo {
                        name: String::from("Catenis test token #11"),
                        symbol: String::from("CTK11"),
                        id: Some(String::from("0x5cE78E7204DD8f7d86142fAaA694d5354b997600")),
                    },
                    status: AssetExportStatus::Success,
                    date: "2021-08-10T12:57:24.217Z".into(),
                },
            ],
            has_more: false,
        });
    }
    
    #[test]
    fn it_deserialize_list_asset_migrations_result() {
        let json = r#"{"assetMigrations":[{"migrationId":"gSLb9FTdGxgSLufuNzhR","assetId":"aH2AkrrL55GcThhPNa3J","foreignBlockchain":"ethereum","direction":"inward","amount":4,"catenisService":{"status":"fulfilled","txid":"26d45a275447caf36e0fbcc32f880f37d3aadb37ddceccc39cd8972a7933e3f4"},"foreignTransaction":{"txid":"0x883a4d9e02713b177fdd26b33e871dc765db3c964f2b1ef8e6f97eca24d718ee","isPending":false,"success":true},"status":"success","date":"2021-08-03T19:11:27.804Z"},{"migrationId":"gTQ8Qf5W6kdmdYdEEoD9","assetId":"aCSy24HLjKMbpnvJ8GTx","foreignBlockchain":"ethereum","direction":"outward","amount":5,"catenisService":{"status":"fulfilled","txid":"7d6a20ee009ad2bcbf5c799ee4eac594e4447bdb5007250f8ba038de97f63777"},"foreignTransaction":{"txid":"0x92fb47432e50b623441bb3b55dd65bf879183f87ea4913a16e75503c98792df9","isPending":false,"success":true},"status":"success","date":"2021-08-10T13:00:08.656Z"}],"hasMore":false}"#;
        
        let list_asset_migrations_result: ListAssetMigrationsResult = serde_json::from_str(json).unwrap();
        
        assert_eq!(list_asset_migrations_result, ListAssetMigrationsResult {
            asset_migrations: vec![
                AssetMigrationEntry {
                    migration_id: String::from("gSLb9FTdGxgSLufuNzhR"),
                    asset_id: String::from("aH2AkrrL55GcThhPNa3J"),
                    foreign_blockchain: ForeignBlockchain::Ethereum,
                    direction: AssetMigrationDirection::Inward,
                    amount: 4.0,
                    catenis_service: CatenisServiceInfo {
                        status: CatenisServiceStatus::Fulfilled,
                        txid: Some(String::from("26d45a275447caf36e0fbcc32f880f37d3aadb37ddceccc39cd8972a7933e3f4")),
                        error: None,
                    },
                    foreign_transaction: ForeignTransactionInfo {
                        txid: String::from("0x883a4d9e02713b177fdd26b33e871dc765db3c964f2b1ef8e6f97eca24d718ee"),
                        is_pending: false,
                        success: Some(true),
                        error: None,
                    },
                    status: AssetMigrationStatus::Success,
                    date: "2021-08-03T19:11:27.804Z".into(),
                },
                AssetMigrationEntry {
                    migration_id: String::from("gTQ8Qf5W6kdmdYdEEoD9"),
                    asset_id: String::from("aCSy24HLjKMbpnvJ8GTx"),
                    foreign_blockchain: ForeignBlockchain::Ethereum,
                    direction: AssetMigrationDirection::Outward,
                    amount: 5.0,
                    catenis_service: CatenisServiceInfo {
                        status: CatenisServiceStatus::Fulfilled,
                        txid: Some(String::from("7d6a20ee009ad2bcbf5c799ee4eac594e4447bdb5007250f8ba038de97f63777")),
                        error: None,
                    },
                    foreign_transaction: ForeignTransactionInfo {
                        txid: String::from("0x92fb47432e50b623441bb3b55dd65bf879183f87ea4913a16e75503c98792df9"),
                        is_pending: false,
                        success: Some(true),
                        error: None,
                    },
                    status: AssetMigrationStatus::Success,
                    date: "2021-08-10T13:00:08.656Z".into(),
                },
            ],
            has_more: false,
        });
    }

    #[test]
    fn it_serialize_log_message_request_no_opts() {
        let log_message_request = LogMessageRequest {
            message: Message::Whole(String::from("Test message")),
            options: None,
        };

        let json = serde_json::to_string(&log_message_request).unwrap();

        assert_eq!(json, r#"{"message":"Test message"}"#);
    }

    #[test]
    fn it_serialize_log_message_request_all_opts() {
        let log_message_request = LogMessageRequest {
            message: Message::Chunk(ChunkedMessage {
                data: Some(String::from("Test message chunk #1")),
                is_final: Some(false),
                continuation_token: Some(String::from("kYkeYcQN2YJdcJLNSEWq")),
            }),
            options: Some(LogMessageOptions {
                encoding: Some(Encoding::UTF8),
                encrypt: None,
                off_chain: None,
                storage: None,
                async_: None,
            }),
        };

        let json = serde_json::to_string(&log_message_request).unwrap();

        assert_eq!(json, r#"{"message":{"data":"Test message chunk #1","isFinal":false,"continuationToken":"kYkeYcQN2YJdcJLNSEWq"},"options":{"encoding":"utf8"}}"#);
    }

    #[test]
    fn it_serialize_send_message_request_no_opts() {
        let send_message_request = SendMessageRequest {
            message: Message::Whole(String::from("Test message")),
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
            message: Message::Chunk(ChunkedMessage {
                data: Some(String::from("Test message chunk #1")),
                is_final: Some(false),
                continuation_token: Some(String::from("kYkeYcQN2YJdcJLNSEWq")),
            }),
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

        assert_eq!(json, r#"{"message":{"data":"Test message chunk #1","isFinal":false,"continuationToken":"kYkeYcQN2YJdcJLNSEWq"},"targetDevice":{"id":"drc3XdxNtzoucpw9xiRp"},"options":{"encoding":"utf8"}}"#);
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
    fn it_serialize_export_asset_request_no_opts() {
        let export_asset_request = ExportAssetRequest {
            token: NewForeignTokenInfo {
                name: String::from("Catenis test token #11"),
                symbol: String::from("CTK11"),
            },
            options: None,
        };

        let json = serde_json::to_string(&export_asset_request).unwrap();

        assert_eq!(json, r#"{"token":{"name":"Catenis test token #11","symbol":"CTK11"}}"#);
    }

    #[test]
    fn it_serialize_export_asset_request_all_opts() {
        let export_asset_request = ExportAssetRequest {
            token: NewForeignTokenInfo {
                name: String::from("Catenis test token #11"),
                symbol: String::from("CTK11"),
            },
            options: Some(ExportAssetOptions {
                consumption_profile: Some(ForeignConsumptionProfile::Average),
                estimate_only: Some(true),
            }),
        };

        let json = serde_json::to_string(&export_asset_request).unwrap();

        assert_eq!(json, r#"{"token":{"name":"Catenis test token #11","symbol":"CTK11"},"options":{"consumptionProfile":"average","estimateOnly":true}}"#);
    }

    #[test]
    fn it_serialize_migrate_asset_request_migration_info_no_opts() {
        let migrate_asset_request = MigrateAssetRequest {
            migration: AssetMigration::Info(AssetMigrationInfo {
                direction: AssetMigrationDirection::Inward,
                amount: 13.0,
                dest_address: None,
            }),
            options: None,
        };

        let json = serde_json::to_string(&migrate_asset_request).unwrap();

        assert_eq!(json, r#"{"migration":{"direction":"inward","amount":13.0}}"#);
    }

    #[test]
    fn it_serialize_migrate_asset_request_migration_info_all_opts() {
        let migrate_asset_request = MigrateAssetRequest {
            migration: AssetMigration::Info(AssetMigrationInfo {
                direction: AssetMigrationDirection::Outward,
                amount: 27.0,
                dest_address: Some(String::from("0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943")),
            }),
            options: Some(MigrateAssetOptions {
                consumption_profile: Some(ForeignConsumptionProfile::Average),
                estimate_only: Some(true),
            }),
        };

        let json = serde_json::to_string(&migrate_asset_request).unwrap();

        assert_eq!(json, r#"{"migration":{"direction":"outward","amount":27.0,"destAddress":"0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943"},"options":{"consumptionProfile":"average","estimateOnly":true}}"#);
    }

    #[test]
    fn it_serialize_migrate_asset_request_migration_id_no_opts() {
        let migrate_asset_request = MigrateAssetRequest {
            migration: AssetMigration::ID(String::from("gSLb9FTdGxgSLufuNzhR")),
            options: None,
        };

        let json = serde_json::to_string(&migrate_asset_request).unwrap();

        assert_eq!(json, r#"{"migration":"gSLb9FTdGxgSLufuNzhR"}"#);
    }

    #[test]
    fn it_serialize_migrate_asset_request_migration_id_all_opts() {
        let migrate_asset_request = MigrateAssetRequest {
            migration: AssetMigration::ID(String::from("gSLb9FTdGxgSLufuNzhR")),
            options: Some(MigrateAssetOptions {
                consumption_profile: Some(ForeignConsumptionProfile::Average),
                estimate_only: Some(true),
            }),
        };

        let json = serde_json::to_string(&migrate_asset_request).unwrap();

        assert_eq!(json, r#"{"migration":"gSLb9FTdGxgSLufuNzhR","options":{"consumptionProfile":"average","estimateOnly":true}}"#);
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
        let json = r#"{"status":"success","data":{"offChain":{"cid":"QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"},"blockchain":{"txid":"505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7","isConfirmed":true},"externalStorage":{"ipfs":"Qmd2FBqC4dGTbiNe7HyYEJR2grpLWPzDNfBgajkDNRhK1X"}}}"#;

        let retrieve_message_container_response: RetrieveMessageContainerResponse = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_message_container_response, RetrieveMessageContainerResponse {
            status: String::from("success"),
            data: RetrieveMessageContainerResult {
                off_chain: Some(OffChainContainer {
                    cid: String::from("QmZZAweh5MvVxhCMggaCD8MNykTYdgDx6XLjCK7LhwWtoX"),
                }),
                blockchain: Some(BlockchainContainer {
                    txid: String::from("505cf8efc6c2f73a1dd25ee49c2419bfedb6e545a17c8768f740c20b3c4f85c7"),
                    is_confirmed: true,
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
                proof: Some(ProofInfo {
                    message: String::from("This is only a test"),
                    signature: String::from("IEn9KpwdIitSilCuERUz5Dg+siFe+tW7Bh4ezHDer5NBCQD1jfWJgYL2SnKnzBnGbay/WXL7eykuK8N4o4gRLNo=")
                }),
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
        let json = r#"{"status":"success","data":{"assetId":"aQjlzShmrnEZeeYBZihc","name":"TestAsset_1","description":"First asset issued for test","isNonFungible":false,"canReissue":false,"decimalPlaces":2,"issuer":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"totalExistentBalance":123.25}}"#;

        let retrieve_asset_info_response: RetrieveAssetInfoResponse = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_asset_info_response, RetrieveAssetInfoResponse {
            status: String::from("success"),
            data: RetrieveAssetInfoResult {
                asset_id: String::from("aQjlzShmrnEZeeYBZihc"),
                name: String::from("TestAsset_1"),
                description: String::from("First asset issued for test"),
                is_non_fungible: false,
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
        let json = r#"{"status":"success","data":{"issuanceEvents":[{"amount":123,"holdingDevice":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"date":"2020-12-23T10:51:45.935Z"},{"nfTokenIds":["tQyJrga3ke65RR23iyr2","tf2rbknDoo9wPsKBkskj"],"holdingDevices":[{"deviceId":"d8YpQ7jgPBJEkBrnvp58"}],"date":"2020-12-24T13:27:02.010Z"}],"hasMore":false}}"#;

        let retrieve_asset_issuance_history_response: RetrieveAssetIssuanceHistoryResponse = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_asset_issuance_history_response, RetrieveAssetIssuanceHistoryResponse {
            status: String::from("success"),
            data: RetrieveAssetIssuanceHistoryResult {
                issuance_events: vec![
                    AssetIssuanceEventEntry::Regular(RegularAssetIssuanceEventEntry {
                        amount: 123.0,
                        holding_device: DeviceInfo {
                            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                            name: None,
                            prod_unique_id: None,
                        },
                        date: "2020-12-23T10:51:45.935Z".into(),
                    }),
                    AssetIssuanceEventEntry::NonFungible(NonFungibleAssetIssuanceEventEntry {
                        nf_token_ids: vec![
                            String::from("tQyJrga3ke65RR23iyr2"),
                            String::from("tf2rbknDoo9wPsKBkskj"),
                        ],
                        holding_devices: vec![
                            DeviceInfo {
                                device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                                name: None,
                                prod_unique_id: None,
                            },
                        ],
                        date: "2020-12-24T13:27:02.010Z".into(),
                    }),
                ],
                has_more: false,
            },
        });
    }

    #[test]
    fn it_deserialize_list_asset_holders_response() {
        let json = r#"{"status":"success","data":{"assetHolders":[{"holder":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"balance":{"total":123.25,"unconfirmed":0}},{"holder":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"},"balance":{"total":150,"unconfirmed":0}},{"migrated":true,"balance":{"total":34.75,"unconfirmed":0}}],"hasMore":false}}"#;

        let list_asset_holders_response: ListAssetHoldersResponse = serde_json::from_str(json).unwrap();

        assert_eq!(list_asset_holders_response, ListAssetHoldersResponse {
            status: String::from("success"),
            data: ListAssetHoldersResult {
                asset_holders: vec![
                    AssetHolderEntry {
                        holder: Some(DeviceInfo {
                            device_id: String::from("drc3XdxNtzoucpw9xiRp"),
                            name: None,
                            prod_unique_id: None,
                        }),
                        migrated: None,
                        balance: AssetBalance {
                            total: 123.25,
                            unconfirmed: 0.0,
                        },
                    },
                    AssetHolderEntry {
                        holder: Some(DeviceInfo {
                            device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                            name: None,
                            prod_unique_id: None,
                        }),
                        migrated: None,
                        balance: AssetBalance {
                            total: 150.0,
                            unconfirmed: 0.0,
                        },
                    },
                    AssetHolderEntry {
                        holder: None,
                        migrated: Some(true),
                        balance: AssetBalance {
                            total: 34.75,
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
                ].into_iter()).collect(),
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
                ].into_iter()).collect(),
            },
        );
    }

    #[test]
    fn it_deserialize_retrieve_device_identification_info_response() {
        let json = r#"{"status":"success","data":{"catenisNode":{"ctnNodeIndex":0},"client":{"clientId":"cEXd845DSMw9g6tM5dhy"},"device":{"deviceId":"drc3XdxNtzoucpw9xiRp"}}}"#;

        let retrieve_device_identification_info_response: RetrieveDeviceIdentificationInfoResponse = serde_json::from_str(json).unwrap();

        assert_eq!(retrieve_device_identification_info_response, RetrieveDeviceIdentificationInfoResponse {
            status: String::from("success"),
            data: RetrieveDeviceIdentificationInfoResult {
                catenis_node: CatenisNodeInfo {
                    ctn_node_index: 0,
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
                ].into_iter()).collect(),
            },
        );
    }

    #[test]
    fn it_deserialize_export_asset_response() {
        let json = r#"{"status":"success","data":{"foreignTransaction":{"txid":"0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a","isPending":true},"token":{"name":"Catenis test token #10","symbol":"CTK10"},"status":"pending","date":"2021-08-03T18:41:11.781Z"}}"#;

        let export_asset_response: ExportAssetResponse = serde_json::from_str(json).unwrap();

        assert_eq!(export_asset_response, ExportAssetResponse {
            status: String::from("success"),
            data: ExportAssetResult {
                foreign_transaction: Some(ForeignTransactionInfo {
                    txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
                    is_pending: true,
                    success: None,
                    error: None,
                }),
                token: Some(ForeignTokenInfo {
                    name: String::from("Catenis test token #10"),
                    symbol: String::from("CTK10"),
                    id: None,
                }),
                status: Some(AssetExportStatus::Pending),
                date: Some("2021-08-03T18:41:11.781Z".into()),
                estimated_price: None,
            },
        });
    }

    #[test]
    fn it_deserialize_migrate_asset_response() {
        let json = r#"{"status":"success","data":{"migrationId":"gq8x3efLpEXTkGQchHTb","catenisService":{"status":"fulfilled","txid":"61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf"},"foreignTransaction":{"txid":"0x212ab54f136a6fc1deae9ec217ef2d0417615178777131e8bb6958447fd20fe7","isPending":true},"status":"pending","date":"2021-08-03T18:51:26.631Z"}}"#;

        let migrate_asset_response: MigrateAssetResponse = serde_json::from_str(json).unwrap();

        assert_eq!(migrate_asset_response, MigrateAssetResponse {
            status: String::from("success"),
            data: MigrateAssetResult {
                migration_id: Some(String::from("gq8x3efLpEXTkGQchHTb")),
                catenis_service: Some(CatenisServiceInfo {
                    status: CatenisServiceStatus::Fulfilled,
                    txid: Some(String::from("61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf")),
                    error: None,
                }),
                foreign_transaction: Some(ForeignTransactionInfo {
                    txid: String::from("0x212ab54f136a6fc1deae9ec217ef2d0417615178777131e8bb6958447fd20fe7"),
                    is_pending: true,
                    success: None,
                    error: None,
                }),
                status: Some(AssetMigrationStatus::Pending),
                date: Some("2021-08-03T18:51:26.631Z".into()),
                estimated_price: None,
            },
        });
    }

    #[test]
    fn it_deserialize_asset_export_outcome_response() {
        let json = r#"{"status":"success","data":{"foreignTransaction":{"txid":"0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a","isPending":false,"success":true},"token":{"name":"Catenis test token #10","symbol":"CTK10","id":"0x537580164Ba9DB2e8C254a38E254ce15d07fDef9"},"status":"success","date":"2021-08-03T18:41:27.679Z"}}"#;

        let asset_export_outcome_response: AssetExportOutcomeResponse = serde_json::from_str(json).unwrap();

        assert_eq!(asset_export_outcome_response, AssetExportOutcomeResponse {
            status: String::from("success"),
            data: AssetExportOutcomeResult {
                foreign_transaction: ForeignTransactionInfo {
                    txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
                    is_pending: false,
                    success: Some(true),
                    error: None,
                },
                token: ForeignTokenInfo {
                    name: String::from("Catenis test token #10"),
                    symbol: String::from("CTK10"),
                    id: Some(String::from("0x537580164Ba9DB2e8C254a38E254ce15d07fDef9")),
                },
                status: AssetExportStatus::Success,
                date: "2021-08-03T18:41:27.679Z".into(),
            },
        });
    }

    #[test]
    fn it_deserialize_asset_migration_outcome_response() {
        let json = r#"{"status":"success","data":{"assetId":"aH2AkrrL55GcThhPNa3J","foreignBlockchain":"ethereum","direction":"outward","amount":10,"catenisService":{"status":"fulfilled","txid":"61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf"},"foreignTransaction":{"txid":"0x212ab54f136a6fc1deae9ec217ef2d0417615178777131e8bb6958447fd20fe7","isPending":false,"success":true},"status":"success","date":"2021-08-03T18:51:55.591Z"}}"#;

        let assert_migration_outcome_response: AssetMigrationOutcomeResponse = serde_json::from_str(json).unwrap();

        assert_eq!(assert_migration_outcome_response, AssetMigrationOutcomeResponse {
            status: String::from("success"),
            data: AssetMigrationOutcomeResult {
                asset_id: String::from("aH2AkrrL55GcThhPNa3J"),
                foreign_blockchain: ForeignBlockchain::Ethereum,
                direction: AssetMigrationDirection::Outward,
                amount: 10.0,
                catenis_service: CatenisServiceInfo {
                    status: CatenisServiceStatus::Fulfilled,
                    txid: Some(String::from("61fcb4feb64ecf3b39b4bb6d64eb9cc68a58ba1d892f981ef568d07b7aa11fdf")),
                    error: None,
                },
                foreign_transaction: ForeignTransactionInfo {
                    txid: String::from("0x212ab54f136a6fc1deae9ec217ef2d0417615178777131e8bb6958447fd20fe7"),
                    is_pending: false,
                    success: Some(true),
                    error: None,
                },
                status: AssetMigrationStatus::Success,
                date: "2021-08-03T18:51:55.591Z".into(),
            },
        });
    }

    #[test]
    fn it_deserialize_list_exported_assets_response() {
        let json = r#"{"status":"success","data":{"exportedAssets":[{"assetId":"aH2AkrrL55GcThhPNa3J","foreignBlockchain":"ethereum","foreignTransaction":{"txid":"0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a","isPending":false,"success":true},"token":{"name":"Catenis test token #10","symbol":"CTK10","id":"0x537580164Ba9DB2e8C254a38E254ce15d07fDef9"},"status":"success","date":"2021-08-03T18:41:27.679Z"},{"assetId":"aCSy24HLjKMbpnvJ8GTx","foreignBlockchain":"ethereum","foreignTransaction":{"txid":"0x6299c35ccfa803ab0cb043e8d8ae4be8d7f3432d85f288ebb81e4d624e566b0a","isPending":false,"success":true},"token":{"name":"Catenis test token #11","symbol":"CTK11","id":"0x5cE78E7204DD8f7d86142fAaA694d5354b997600"},"status":"success","date":"2021-08-10T12:57:24.217Z"}],"hasMore":false}}"#;

        let list_exported_assets_response: ListExportedAssetsResponse = serde_json::from_str(json).unwrap();

        assert_eq!(list_exported_assets_response, ListExportedAssetsResponse {
            status: String::from("success"),
            data: ListExportedAssetsResult {
                exported_assets: vec![
                    AssetExportEntry {
                        asset_id: String::from("aH2AkrrL55GcThhPNa3J"),
                        foreign_blockchain: ForeignBlockchain::Ethereum,
                        foreign_transaction: ForeignTransactionInfo {
                            txid: String::from("0x1f14474f441557056055a186ccf6839bd4dfce79e0b134d77084b6ef4274dc1a"),
                            is_pending: false,
                            success: Some(true),
                            error: None,
                        },
                        token: ForeignTokenInfo {
                            name: String::from("Catenis test token #10"),
                            symbol: String::from("CTK10"),
                            id: Some(String::from("0x537580164Ba9DB2e8C254a38E254ce15d07fDef9")),
                        },
                        status: AssetExportStatus::Success,
                        date: "2021-08-03T18:41:27.679Z".into(),
                    },
                    AssetExportEntry {
                        asset_id: String::from("aCSy24HLjKMbpnvJ8GTx"),
                        foreign_blockchain: ForeignBlockchain::Ethereum,
                        foreign_transaction: ForeignTransactionInfo {
                            txid: String::from("0x6299c35ccfa803ab0cb043e8d8ae4be8d7f3432d85f288ebb81e4d624e566b0a"),
                            is_pending: false,
                            success: Some(true),
                            error: None,
                        },
                        token: ForeignTokenInfo {
                            name: String::from("Catenis test token #11"),
                            symbol: String::from("CTK11"),
                            id: Some(String::from("0x5cE78E7204DD8f7d86142fAaA694d5354b997600")),
                        },
                        status: AssetExportStatus::Success,
                        date: "2021-08-10T12:57:24.217Z".into(),
                    },
                ],
                has_more: false,
            },
        });
    }

    #[test]
    fn it_deserialize_list_asset_migrations_response() {
        let json = r#"{"status":"success","data":{"assetMigrations":[{"migrationId":"gSLb9FTdGxgSLufuNzhR","assetId":"aH2AkrrL55GcThhPNa3J","foreignBlockchain":"ethereum","direction":"inward","amount":4,"catenisService":{"status":"fulfilled","txid":"26d45a275447caf36e0fbcc32f880f37d3aadb37ddceccc39cd8972a7933e3f4"},"foreignTransaction":{"txid":"0x883a4d9e02713b177fdd26b33e871dc765db3c964f2b1ef8e6f97eca24d718ee","isPending":false,"success":true},"status":"success","date":"2021-08-03T19:11:27.804Z"},{"migrationId":"gTQ8Qf5W6kdmdYdEEoD9","assetId":"aCSy24HLjKMbpnvJ8GTx","foreignBlockchain":"ethereum","direction":"outward","amount":5,"catenisService":{"status":"fulfilled","txid":"7d6a20ee009ad2bcbf5c799ee4eac594e4447bdb5007250f8ba038de97f63777"},"foreignTransaction":{"txid":"0x92fb47432e50b623441bb3b55dd65bf879183f87ea4913a16e75503c98792df9","isPending":false,"success":true},"status":"success","date":"2021-08-10T13:00:08.656Z"}],"hasMore":false}}"#;

        let list_asset_migrations_response: ListAssetMigrationsResponse = serde_json::from_str(json).unwrap();

        assert_eq!(list_asset_migrations_response, ListAssetMigrationsResponse {
            status: String::from("success"),
            data: ListAssetMigrationsResult {
                asset_migrations: vec![
                    AssetMigrationEntry {
                        migration_id: String::from("gSLb9FTdGxgSLufuNzhR"),
                        asset_id: String::from("aH2AkrrL55GcThhPNa3J"),
                        foreign_blockchain: ForeignBlockchain::Ethereum,
                        direction: AssetMigrationDirection::Inward,
                        amount: 4.0,
                        catenis_service: CatenisServiceInfo {
                            status: CatenisServiceStatus::Fulfilled,
                            txid: Some(String::from("26d45a275447caf36e0fbcc32f880f37d3aadb37ddceccc39cd8972a7933e3f4")),
                            error: None,
                        },
                        foreign_transaction: ForeignTransactionInfo {
                            txid: String::from("0x883a4d9e02713b177fdd26b33e871dc765db3c964f2b1ef8e6f97eca24d718ee"),
                            is_pending: false,
                            success: Some(true),
                            error: None,
                        },
                        status: AssetMigrationStatus::Success,
                        date: "2021-08-03T19:11:27.804Z".into(),
                    },
                    AssetMigrationEntry {
                        migration_id: String::from("gTQ8Qf5W6kdmdYdEEoD9"),
                        asset_id: String::from("aCSy24HLjKMbpnvJ8GTx"),
                        foreign_blockchain: ForeignBlockchain::Ethereum,
                        direction: AssetMigrationDirection::Outward,
                        amount: 5.0,
                        catenis_service: CatenisServiceInfo {
                            status: CatenisServiceStatus::Fulfilled,
                            txid: Some(String::from("7d6a20ee009ad2bcbf5c799ee4eac594e4447bdb5007250f8ba038de97f63777")),
                            error: None,
                        },
                        foreign_transaction: ForeignTransactionInfo {
                            txid: String::from("0x92fb47432e50b623441bb3b55dd65bf879183f87ea4913a16e75503c98792df9"),
                            is_pending: false,
                            success: Some(true),
                            error: None,
                        },
                        status: AssetMigrationStatus::Success,
                        date: "2021-08-10T13:00:08.656Z".into(),
                    },
                ],
                has_more: false,
            },
        });
    }

    mod nf_assets_tests {
        use super::*;

        #[test]
        fn it_serialize_new_non_fungible_asset_info_no_opts() {
            let new_nf_asset_info = NewNonFungibleAssetInfo {
                name: String::from("TestNFAsset_1"),
                description: None,
                can_reissue: false,
            };

            let json = serde_json::to_string(&new_nf_asset_info).unwrap();

            assert_eq!(json, r#"{"name":"TestNFAsset_1","canReissue":false}"#);
        }

        #[test]
        fn it_serialize_new_asset_info_all_opts() {
            let new_nf_asset_info = NewNonFungibleAssetInfo {
                name: String::from("TestNFAsset_1"),
                description: Some(String::from("First non-fungible asset issued for test")),
                can_reissue: true,
            };

            let json = serde_json::to_string(&new_nf_asset_info).unwrap();

            assert_eq!(json, r#"{"name":"TestNFAsset_1","description":"First non-fungible asset issued for test","canReissue":true}"#);
        }

        #[test]
        fn it_serialize_new_non_fungible_token_metadata_no_opts() {
            let new_nf_token_metadata = NewNonFungibleTokenMetadata {
                name: String::from("TestNFToken_1"),
                description: None,
                custom: None,
            };

            let json = serde_json::to_string(&new_nf_token_metadata).unwrap();

            assert_eq!(json, r#"{"name":"TestNFToken_1"}"#);
        }

        #[test]
        fn it_serialize_new_non_fungible_token_metadata_all_opts() {
            let new_nf_token_metadata = NewNonFungibleTokenMetadata {
                name: String::from("TestNFToken_1"),
                description: Some(String::from("First non-fungible token issued for test")),
                custom: json_obj!({
                    "propNum": 5,
                    "propStr": "ABC",
                    "propBool": true
                }),
            };

            let json = serde_json::to_string(&new_nf_token_metadata).unwrap();

            assert_eq!(json, r#"{"name":"TestNFToken_1","description":"First non-fungible token issued for test","custom":{"propNum":5,"propStr":"ABC","propBool":true}}"#);
        }

        #[test]
        fn it_serialize_new_non_fungible_token_metadata_all_opts_sensitive_props() {
            let new_nf_token_metadata = NewNonFungibleTokenMetadata {
                name: String::from("TestNFToken_1"),
                description: Some(String::from("First non-fungible token issued for test")),
                custom: json_obj!({
                    "sensitiveProps": {
                        "senseProp1": "XYZ",
                        "senseProp2": "456"
                    },
                    "propNum": 5,
                    "propStr": "ABC",
                    "propBool": true
                }),
            };

            let json = serde_json::to_string(&new_nf_token_metadata).unwrap();

            assert_eq!(json, r#"{"name":"TestNFToken_1","description":"First non-fungible token issued for test","custom":{"sensitiveProps":{"senseProp1":"XYZ","senseProp2":"456"},"propNum":5,"propStr":"ABC","propBool":true}}"#);
        }

        #[test]
        fn it_serialize_new_non_fungible_token_contents() {
            let new_nf_token_contents = NewNonFungibleTokenContents {
                data: String::from("This is the contents of non-fungible token #1"),
                encoding: Encoding::UTF8,
            };

            let json = serde_json::to_string(&new_nf_token_contents).unwrap();

            assert_eq!(json, r#"{"data":"This is the contents of non-fungible token #1","encoding":"utf8"}"#);
        }

        #[test]
        fn it_serialize_new_non_fungible_token_info_no_opts() {
            let new_nf_token_info = NewNonFungibleTokenInfo {
                metadata: None,
                contents: None,
            };

            let json = serde_json::to_string(&new_nf_token_info).unwrap();

            assert_eq!(json, r#"{}"#);
        }

        #[test]
        fn it_serialize_new_non_fungible_token_info_all_opts() {
            let new_nf_token_info = NewNonFungibleTokenInfo {
                metadata: Some(NewNonFungibleTokenMetadata {
                    name: String::from("TestNFToken_1"),
                    description: Some(String::from("First non-fungible token issued for test")),
                    custom: json_obj!({
                        "sensitiveProps": {
                            "senseProp1": "XYZ",
                            "senseProp2": "456"
                        },
                        "propNum": 5,
                        "propStr": "ABC",
                        "propBool": true
                    }),
                }),
                contents: Some(NewNonFungibleTokenContents {
                    data: String::from("This is the contents of non-fungible token #1"),
                    encoding: Encoding::UTF8,
                }),
            };

            let json = serde_json::to_string(&new_nf_token_info).unwrap();

            assert_eq!(json, r#"{"metadata":{"name":"TestNFToken_1","description":"First non-fungible token issued for test","custom":{"sensitiveProps":{"senseProp1":"XYZ","senseProp2":"456"},"propNum":5,"propStr":"ABC","propBool":true}},"contents":{"data":"This is the contents of non-fungible token #1","encoding":"utf8"}}"#);
        }

        #[test]
        fn it_serialize_issue_non_fungible_asset_request_cont_call_single_token() {
            let issue_non_fungible_asset_request = IssueNonFungibleAssetRequest {
                asset_info: None,
                encrypt_nft_contents: None,
                holding_devices: None,
                async_: None,
                continuation_token: Some(String::from("bRQDsLZpksdHyMPxFk3J")),
                non_fungible_tokens: Some(vec![
                    None,
                    Some(NewNonFungibleTokenInfo {
                        metadata: None,
                        contents: Some(NewNonFungibleTokenContents {
                            data: String::from("Final part of non-fungible token contents"),
                            encoding: Encoding::UTF8
                        })
                    })
                ]),
                is_final: Some(false),
            };

            let json = serde_json::to_string(&issue_non_fungible_asset_request).unwrap();

            assert_eq!(json, r#"{"continuationToken":"bRQDsLZpksdHyMPxFk3J","nonFungibleTokens":[null,{"contents":{"data":"Final part of non-fungible token contents","encoding":"utf8"}}],"isFinal":false}"#);
        }

        #[test]
        fn it_serialize_issue_non_fungible_asset_request_cont_call_no_tokens() {
            let issue_non_fungible_asset_request = IssueNonFungibleAssetRequest {
                asset_info: None,
                encrypt_nft_contents: None,
                holding_devices: None,
                async_: None,
                continuation_token: Some(String::from("bRQDsLZpksdHyMPxFk3J")),
                non_fungible_tokens: None,
                is_final: Some(true),
            };

            let json = serde_json::to_string(&issue_non_fungible_asset_request).unwrap();

            assert_eq!(json, r#"{"continuationToken":"bRQDsLZpksdHyMPxFk3J","isFinal":true}"#);
        }

        #[test]
        fn it_serialize_issue_non_fungible_asset_request_non_cont_call_no_opts() {
            let issue_non_fungible_asset_request = IssueNonFungibleAssetRequest {
                asset_info: Some(NewNonFungibleAssetInfo {
                    name: String::from("TestNFAsset_1"),
                    description: Some(String::from("First non-fungible asset issued for test")),
                    can_reissue: true,
                }),
                encrypt_nft_contents: None,
                holding_devices: None,
                async_: None,
                continuation_token: None,
                non_fungible_tokens: Some(vec![
                    Some(NewNonFungibleTokenInfo {
                        metadata: Some(NewNonFungibleTokenMetadata {
                            name: String::from("TestNFToken_1"),
                            description: Some(String::from("First non-fungible token issued for test")),
                            custom: None,
                        }),
                        contents: Some(NewNonFungibleTokenContents {
                            data: String::from("This is the contents of non-fungible token #1"),
                            encoding: Encoding::UTF8,
                        }),
                    }),
                ]),
                is_final: None,
            };

            let json = serde_json::to_string(&issue_non_fungible_asset_request).unwrap();

            assert_eq!(json, r#"{"assetInfo":{"name":"TestNFAsset_1","description":"First non-fungible asset issued for test","canReissue":true},"nonFungibleTokens":[{"metadata":{"name":"TestNFToken_1","description":"First non-fungible token issued for test"},"contents":{"data":"This is the contents of non-fungible token #1","encoding":"utf8"}}]}"#);
        }

        #[test]
        fn it_serialize_issue_non_fungible_asset_request_non_cont_call_all_opts() {
            let issue_non_fungible_asset_request = IssueNonFungibleAssetRequest {
                asset_info: Some(NewNonFungibleAssetInfo {
                    name: String::from("TestNFAsset_1"),
                    description: Some(String::from("First non-fungible asset issued for test")),
                    can_reissue: true,
                }),
                encrypt_nft_contents: Some(true),
                holding_devices: Some(vec![
                    DeviceId {
                        id: String::from("drc3XdxNtzoucpw9xiRp"),
                        is_prod_unique_id: Some(false)
                    },
                ]),
                async_: Some(false),
                continuation_token: None,
                non_fungible_tokens: Some(vec![
                    Some(NewNonFungibleTokenInfo {
                        metadata: Some(NewNonFungibleTokenMetadata {
                            name: String::from("TestNFToken_1"),
                            description: Some(String::from("First non-fungible token issued for test")),
                            custom: None,
                        }),
                        contents: Some(NewNonFungibleTokenContents {
                            data: String::from("This is the contents of non-fungible token #1"),
                            encoding: Encoding::UTF8,
                        }),
                    }),
                ]),
                is_final: Some(true),
            };

            let json = serde_json::to_string(&issue_non_fungible_asset_request).unwrap();

            assert_eq!(json, r#"{"assetInfo":{"name":"TestNFAsset_1","description":"First non-fungible asset issued for test","canReissue":true},"encryptNFTContents":true,"holdingDevices":[{"id":"drc3XdxNtzoucpw9xiRp","isProdUniqueId":false}],"async":false,"nonFungibleTokens":[{"metadata":{"name":"TestNFToken_1","description":"First non-fungible token issued for test"},"contents":{"data":"This is the contents of non-fungible token #1","encoding":"utf8"}}],"isFinal":true}"#);
        }

        #[test]
        fn it_deserialize_issue_non_fungible_asset_result_non_final() {
            let json = r#"{"continuationToken":"bRQDsLZpksdHyMPxFk3J"}"#;

            let issue_nf_asset_result: IssueNonFungibleAssetResult = serde_json::from_str(json).unwrap();

            assert_eq!(issue_nf_asset_result, IssueNonFungibleAssetResult {
                continuation_token: Some(String::from("bRQDsLZpksdHyMPxFk3J")),
                asset_issuance_id: None,
                asset_id: None,
                nf_token_ids: None,
            });
        }

        #[test]
        fn it_deserialize_issue_non_fungible_asset_result_async() {
            let json = r#"{"assetIssuanceId":"iNQfcL3jCJm4yYZQN9pk"}"#;

            let issue_nf_asset_result: IssueNonFungibleAssetResult = serde_json::from_str(json).unwrap();

            assert_eq!(issue_nf_asset_result, IssueNonFungibleAssetResult {
                continuation_token: None,
                asset_issuance_id: Some(String::from("iNQfcL3jCJm4yYZQN9pk")),
                asset_id: None,
                nf_token_ids: None,
            });
        }

        #[test]
        fn it_deserialize_issue_non_fungible_asset_result_final() {
            let json = r#"{"assetId":"ahfTzqgWAXnMR6Z57mcp","nfTokenIds":["tSWtJurhbkSJLGjjbN4R","t76Yzrbqcjbtehk6Wecf"]}"#;

            let issue_nf_asset_result: IssueNonFungibleAssetResult = serde_json::from_str(json).unwrap();

            assert_eq!(issue_nf_asset_result, IssueNonFungibleAssetResult {
                continuation_token: None,
                asset_issuance_id: None,
                asset_id: Some(String::from("ahfTzqgWAXnMR6Z57mcp")),
                nf_token_ids: Some(vec![
                    String::from("tSWtJurhbkSJLGjjbN4R"),
                    String::from("t76Yzrbqcjbtehk6Wecf"),
                ]),
            });
        }

        #[test]
        fn it_deserialize_issue_non_fungible_asset_response() {
            let json = r#"{"status":"success","data":{"assetId":"ahfTzqgWAXnMR6Z57mcp","nfTokenIds":["tSWtJurhbkSJLGjjbN4R","t76Yzrbqcjbtehk6Wecf"]}}"#;

            let issue_nf_asset_response: IssueNonFungibleAssetResponse = serde_json::from_str(json).unwrap();

            assert_eq!(issue_nf_asset_response, IssueNonFungibleAssetResponse {
                status: String::from("success"),
                data: IssueNonFungibleAssetResult {
                    continuation_token: None,
                    asset_issuance_id: None,
                    asset_id: Some(String::from("ahfTzqgWAXnMR6Z57mcp")),
                    nf_token_ids: Some(vec![
                        String::from("tSWtJurhbkSJLGjjbN4R"),
                        String::from("t76Yzrbqcjbtehk6Wecf"),
                    ]),
                },
            });
        }

        #[test]
        fn it_serialize_reissue_non_fungible_asset_request_cont_call_single_token() {
            let reissue_non_fungible_asset_request = ReissueNonFungibleAssetRequest {
                encrypt_nft_contents: None,
                holding_devices: None,
                async_: None,
                continuation_token: Some(String::from("bXdG5i8EAPwA5RDdtXb3")),
                non_fungible_tokens: Some(vec![
                    None,
                    Some(NewNonFungibleTokenInfo {
                        metadata: None,
                        contents: Some(NewNonFungibleTokenContents {
                            data: "Final part of non-fungible token contents".to_string(),
                            encoding: Encoding::UTF8
                        })
                    })
                ]),
                is_final: Some(false),
            };

            let json = serde_json::to_string(&reissue_non_fungible_asset_request).unwrap();

            assert_eq!(json, r#"{"continuationToken":"bXdG5i8EAPwA5RDdtXb3","nonFungibleTokens":[null,{"contents":{"data":"Final part of non-fungible token contents","encoding":"utf8"}}],"isFinal":false}"#);
        }

        #[test]
        fn it_serialize_reissue_non_fungible_asset_request_cont_call_no_tokens() {
            let reissue_non_fungible_asset_request = ReissueNonFungibleAssetRequest {
                encrypt_nft_contents: None,
                holding_devices: None,
                async_: None,
                continuation_token: Some(String::from("bXdG5i8EAPwA5RDdtXb3")),
                non_fungible_tokens: None,
                is_final: Some(true),
            };

            let json = serde_json::to_string(&reissue_non_fungible_asset_request).unwrap();

            assert_eq!(json, r#"{"continuationToken":"bXdG5i8EAPwA5RDdtXb3","isFinal":true}"#);
        }

        #[test]
        fn it_serialize_reissue_non_fungible_asset_request_non_cont_call_no_opts() {
            let reissue_non_fungible_asset_request = ReissueNonFungibleAssetRequest {
                encrypt_nft_contents: None,
                holding_devices: None,
                async_: None,
                continuation_token: None,
                non_fungible_tokens: Some(vec![
                    Some(NewNonFungibleTokenInfo {
                        metadata: Some(NewNonFungibleTokenMetadata {
                            name: String::from("TestNFToken_2"),
                            description: Some(String::from("Second non-fungible token issued for test")),
                            custom: None,
                        }),
                        contents: Some(NewNonFungibleTokenContents {
                            data: String::from("This is the contents of non-fungible token #2"),
                            encoding: Encoding::UTF8,
                        }),
                    }),
                ]),
                is_final: None,
            };

            let json = serde_json::to_string(&reissue_non_fungible_asset_request).unwrap();

            assert_eq!(json, r#"{"nonFungibleTokens":[{"metadata":{"name":"TestNFToken_2","description":"Second non-fungible token issued for test"},"contents":{"data":"This is the contents of non-fungible token #2","encoding":"utf8"}}]}"#);
        }

        #[test]
        fn it_serialize_reissue_non_fungible_asset_request_non_cont_call_all_opts() {
            let reissue_non_fungible_asset_request = ReissueNonFungibleAssetRequest {
                encrypt_nft_contents: Some(true),
                holding_devices: Some(vec![
                    DeviceId {
                        id: String::from("drc3XdxNtzoucpw9xiRp"),
                        is_prod_unique_id: Some(false)
                    },
                ]),
                async_: Some(false),
                continuation_token: None,
                non_fungible_tokens: Some(vec![
                    Some(NewNonFungibleTokenInfo {
                        metadata: Some(NewNonFungibleTokenMetadata {
                            name: String::from("TestNFToken_2"),
                            description: Some(String::from("Second non-fungible token issued for test")),
                            custom: None,
                        }),
                        contents: Some(NewNonFungibleTokenContents {
                            data: String::from("This is the contents of non-fungible token #2"),
                            encoding: Encoding::UTF8,
                        }),
                    }),
                ]),
                is_final: Some(true),
            };

            let json = serde_json::to_string(&reissue_non_fungible_asset_request).unwrap();

            assert_eq!(json, r#"{"encryptNFTContents":true,"holdingDevices":[{"id":"drc3XdxNtzoucpw9xiRp","isProdUniqueId":false}],"async":false,"nonFungibleTokens":[{"metadata":{"name":"TestNFToken_2","description":"Second non-fungible token issued for test"},"contents":{"data":"This is the contents of non-fungible token #2","encoding":"utf8"}}],"isFinal":true}"#);
        }

        #[test]
        fn it_deserialize_reissue_non_fungible_asset_result_non_final() {
            let json = r#"{"continuationToken":"bvveXeSB9KjR4j7EJ98i"}"#;

            let reissue_nf_asset_result: ReissueNonFungibleAssetResult = serde_json::from_str(json).unwrap();

            assert_eq!(reissue_nf_asset_result, ReissueNonFungibleAssetResult {
                continuation_token: Some(String::from("bvveXeSB9KjR4j7EJ98i")),
                asset_issuance_id: None,
                nf_token_ids: None,
            });
        }

        #[test]
        fn it_deserialize_reissue_non_fungible_asset_result_async() {
            let json = r#"{"assetIssuanceId":"iWWKqTx6svmErabyCZKM"}"#;

            let reissue_nf_asset_result: ReissueNonFungibleAssetResult = serde_json::from_str(json).unwrap();

            assert_eq!(reissue_nf_asset_result, ReissueNonFungibleAssetResult {
                continuation_token: None,
                asset_issuance_id: Some(String::from("iWWKqTx6svmErabyCZKM")),
                nf_token_ids: None,
            });
        }

        #[test]
        fn it_deserialize_reissue_non_fungible_asset_result_final() {
            let json = r#"{"nfTokenIds":["tDGQpGy627J6uAw4grYq","tpekkQwM8XA9Dt6q5XLk"]}"#;

            let reissue_nf_asset_result: ReissueNonFungibleAssetResult = serde_json::from_str(json).unwrap();

            assert_eq!(reissue_nf_asset_result, ReissueNonFungibleAssetResult {
                continuation_token: None,
                asset_issuance_id: None,
                nf_token_ids: Some(vec![
                    String::from("tDGQpGy627J6uAw4grYq"),
                    String::from("tpekkQwM8XA9Dt6q5XLk"),
                ]),
            });
        }

        #[test]
        fn it_deserialize_reissue_non_fungible_asset_response() {
            let json = r#"{"status":"success","data":{"nfTokenIds":["tDGQpGy627J6uAw4grYq","tpekkQwM8XA9Dt6q5XLk"]}}"#;

            let reissue_nf_asset_response: ReissueNonFungibleAssetResponse = serde_json::from_str(json).unwrap();

            assert_eq!(reissue_nf_asset_response, ReissueNonFungibleAssetResponse {
                status: String::from("success"),
                data: ReissueNonFungibleAssetResult {
                    continuation_token: None,
                    asset_issuance_id: None,
                    nf_token_ids: Some(vec![
                        String::from("tDGQpGy627J6uAw4grYq"),
                        String::from("tpekkQwM8XA9Dt6q5XLk"),
                    ]),
                },
            });
        }

        #[test]
        fn it_deserialize_nf_asset_issuance_process_error() {
            let json = r#"{"code":500,"message":"Internal server error"}"#;

            let nf_asset_issuance_process_error: NFAssetIssuanceProcessError = serde_json::from_str(json).unwrap();

            assert_eq!(nf_asset_issuance_process_error, NFAssetIssuanceProcessError {
                code: 500,
                message: String::from("Internal server error"),
            });
        }

        #[test]
        fn it_deserialize_nf_asset_issuance_process_progress_not_finished() {
            let json = r#"{"percentProcessed":0,"done":false}"#;

            let nf_asset_issuance_process_progress: NFAssetIssuanceProcessProgress = serde_json::from_str(json).unwrap();

            assert_eq!(nf_asset_issuance_process_progress, NFAssetIssuanceProcessProgress {
                percent_processed: 0,
                done: false,
                success: None,
                error: None,
                finish_date: None,
            });
        }

        #[test]
        fn it_deserialize_nf_asset_issuance_process_progress_finished_error() {
            let json = r#"{"percentProcessed":25,"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2022-11-01T16:55:21.000"}"#;

            let nf_asset_issuance_process_progress: NFAssetIssuanceProcessProgress = serde_json::from_str(json).unwrap();

            assert_eq!(nf_asset_issuance_process_progress, NFAssetIssuanceProcessProgress {
                percent_processed: 25,
                done: true,
                success: Some(false),
                error: Some(NFAssetIssuanceProcessError {
                    code: 500,
                    message: String::from("Internal server error"),
                }),
                finish_date: Some("2022-11-01T16:55:21.000".into()),
            });
        }

        #[test]
        fn it_deserialize_nf_asset_issuance_process_progress_finished_success() {
            let json = r#"{"percentProcessed":100,"done":true,"success":true,"finishDate":"2022-11-01T16:57:46.123"}"#;

            let nf_asset_issuance_process_progress: NFAssetIssuanceProcessProgress = serde_json::from_str(json).unwrap();

            assert_eq!(nf_asset_issuance_process_progress, NFAssetIssuanceProcessProgress {
                percent_processed: 100,
                done: true,
                success: Some(true),
                error: None,
                finish_date: Some("2022-11-01T16:57:46.123".into()),
            });
        }

        #[test]
        fn it_deserialize_nf_asset_issuance_result_not_reissuance() {
            let json = r#"{"assetId":"ahfTzqgWAXnMR6Z57mcp","nfTokenIds":["tSWtJurhbkSJLGjjbN4R","t76Yzrbqcjbtehk6Wecf"]}"#;

            let nf_asset_issuance_result: NFAssetIssuanceResult = serde_json::from_str(json).unwrap();

            assert_eq!(nf_asset_issuance_result, NFAssetIssuanceResult {
                asset_id: Some(String::from("ahfTzqgWAXnMR6Z57mcp")),
                nf_token_ids: vec![
                    String::from("tSWtJurhbkSJLGjjbN4R"),
                    String::from("t76Yzrbqcjbtehk6Wecf"),
                ],
            });
        }

        #[test]
        fn it_deserialize_nf_asset_issuance_result_reissuance() {
            let json = r#"{"nfTokenIds":["tDGQpGy627J6uAw4grYq"]}"#;

            let nf_asset_issuance_result: NFAssetIssuanceResult = serde_json::from_str(json).unwrap();

            assert_eq!(nf_asset_issuance_result, NFAssetIssuanceResult {
                asset_id: None,
                nf_token_ids: vec![
                    String::from("tDGQpGy627J6uAw4grYq"),
                ],
            });
        }

        #[test]
        fn it_deserialize_retrieve_nf_asset_issuance_progress_result_no_opts() {
            let json = r#"{"progress":{"percentProcessed":25,"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2022-11-01T16:55:21.000"}}"#;

            let nf_asset_issuance_progress_result: RetrieveNFAssetIssuanceProgressResult = serde_json::from_str(json).unwrap();

            assert_eq!(nf_asset_issuance_progress_result, RetrieveNFAssetIssuanceProgressResult {
                asset_id: None,
                progress: NFAssetIssuanceProcessProgress {
                    percent_processed: 25,
                    done: true,
                    success: Some(false),
                    error: Some(NFAssetIssuanceProcessError {
                        code: 500,
                        message: String::from("Internal server error"),
                    }),
                    finish_date: Some("2022-11-01T16:55:21.000".into()),
                },
                result: None
            });
        }

        #[test]
        fn it_deserialize_retrieve_nf_asset_issuance_progress_result_all_opts() {
            let json = r#"{"assetId":"ahfTzqgWAXnMR6Z57mcp","progress":{"percentProcessed":100,"done":true,"success":true,"finishDate":"2022-11-01T16:57:46.123"},"result":{"nfTokenIds":["tDGQpGy627J6uAw4grYq"]}}"#;

            let nf_asset_issuance_progress_result: RetrieveNFAssetIssuanceProgressResult = serde_json::from_str(json).unwrap();

            assert_eq!(nf_asset_issuance_progress_result, RetrieveNFAssetIssuanceProgressResult {
                asset_id: Some(String::from("ahfTzqgWAXnMR6Z57mcp")),
                progress: NFAssetIssuanceProcessProgress {
                    percent_processed: 100,
                    done: true,
                    success: Some(true),
                    error: None,
                    finish_date: Some("2022-11-01T16:57:46.123".into()),
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
        fn it_deserialize_retrieve_nf_asset_issuance_progress_response() {
            let json = r#"{"status":"success","data":{"assetId":"ahfTzqgWAXnMR6Z57mcp","progress":{"percentProcessed":100,"done":true,"success":true,"finishDate":"2022-11-01T16:57:46.123"},"result":{"nfTokenIds":["tDGQpGy627J6uAw4grYq"]}}}"#;
            let nf_asset_issuance_progress_response: RetrieveNFAssetIssuanceProgressResponse = serde_json::from_str(json).unwrap();

            assert_eq!(nf_asset_issuance_progress_response, RetrieveNFAssetIssuanceProgressResponse {
                status: String::from("success"),
                data: RetrieveNFAssetIssuanceProgressResult {
                    asset_id: Some(String::from("ahfTzqgWAXnMR6Z57mcp")),
                    progress: NFAssetIssuanceProcessProgress {
                        percent_processed: 100,
                        done: true,
                        success: Some(true),
                        error: None,
                        finish_date: Some("2022-11-01T16:57:46.123".into()),
                    },
                    result: Some(NFAssetIssuanceResult {
                        asset_id: None,
                        nf_token_ids: vec![
                            String::from("tDGQpGy627J6uAw4grYq"),
                        ],
                    }),
                },
            });
        }

        #[test]
        fn it_deserialize_non_fungible_token_metadata_no_opts() {
            let json = r#"{"name":"TestNFToken_1","contentsEncrypted":true,"contentsURL":"https://localhost:8080/ipfs/QmeJKgZ638x2pfFVVaZAB9XjgLCSHh3e6qjMSCmSXKcuUq"}"#;
            let non_fungible_token_metadata: NonFungibleTokenMetadata = serde_json::from_str(json).unwrap();

            assert_eq!(non_fungible_token_metadata, NonFungibleTokenMetadata {
                name: String::from("TestNFToken_1"),
                description: None,
                contents_encrypted: true,
                contents_url: String::from("https://localhost:8080/ipfs/QmeJKgZ638x2pfFVVaZAB9XjgLCSHh3e6qjMSCmSXKcuUq"),
                custom: None,
            });
        }

        #[test]
        fn it_deserialize_non_fungible_token_metadata_all_opts() {
            let json = r#"{"name":"TestNFToken_1","description":"First non-fungible token issued for test","contentsEncrypted":true,"contentsURL":"https://localhost:8080/ipfs/QmeJKgZ638x2pfFVVaZAB9XjgLCSHh3e6qjMSCmSXKcuUq","custom":{"sensitiveProps":{"senseProp1":"XYZ","senseProp2":"456"},"propNum":5,"propStr":"ABC","propBool":true}}"#;
            let non_fungible_token_metadata: NonFungibleTokenMetadata = serde_json::from_str(json).unwrap();

            assert_eq!(non_fungible_token_metadata, NonFungibleTokenMetadata {
                name: String::from("TestNFToken_1"),
                description: Some(String::from("First non-fungible token issued for test")),
                contents_encrypted: true,
                contents_url: String::from("https://localhost:8080/ipfs/QmeJKgZ638x2pfFVVaZAB9XjgLCSHh3e6qjMSCmSXKcuUq"),
                custom: json_obj!({
                    "sensitiveProps": {
                        "senseProp1": "XYZ",
                        "senseProp2": "456"
                    },
                    "propNum": 5,
                    "propStr": "ABC",
                    "propBool": true
                }),
            });
        }

        #[test]
        fn it_deserialize_non_fungible_token_contents() {
            let json = r#"{"data":"Q29udGVudHMgb2YgZmlyc3QgdG9rZW4gb2YgQ2F0ZW5pcyBub24tZnVuZ2libGUgYXNzZXQgIzEw"}"#;
            let non_fungible_token_contents: NonFungibleTokenContents = serde_json::from_str(json).unwrap();

            assert_eq!(non_fungible_token_contents, NonFungibleTokenContents {
                data: String::from("Q29udGVudHMgb2YgZmlyc3QgdG9rZW4gb2YgQ2F0ZW5pcyBub24tZnVuZ2libGUgYXNzZXQgIzEw"),
            });
        }

        #[test]
        fn it_deserialize_non_fungible_token_info_no_contents() {
            let json = r#"{"assetId":"a5sCytXhvRCCGZ7PhQ6o","metadata":{"name":"TestNFToken_1","description":"First non-fungible token issued for test","contentsEncrypted":true,"contentsURL":"https://localhost:8080/ipfs/QmeJKgZ638x2pfFVVaZAB9XjgLCSHh3e6qjMSCmSXKcuUq","custom":{"sensitiveProps":{"senseProp1":"XYZ","senseProp2":"456"},"propNum":5,"propStr":"ABC","propBool":true}}}"#;
            let non_fungible_token_info: NonFungibleTokenInfo = serde_json::from_str(json).unwrap();

            assert_eq!(non_fungible_token_info, NonFungibleTokenInfo {
                asset_id: Some(String::from("a5sCytXhvRCCGZ7PhQ6o")),
                metadata: Some(NonFungibleTokenMetadata {
                    name: String::from("TestNFToken_1"),
                    description: Some(String::from("First non-fungible token issued for test")),
                    contents_encrypted: true,
                    contents_url: String::from("https://localhost:8080/ipfs/QmeJKgZ638x2pfFVVaZAB9XjgLCSHh3e6qjMSCmSXKcuUq"),
                    custom: json_obj!({
                        "sensitiveProps": {
                            "senseProp1": "XYZ",
                            "senseProp2": "456"
                        },
                        "propNum": 5,
                        "propStr": "ABC",
                        "propBool": true
                    }),
                }),
                contents: None,
            });
        }

        #[test]
        fn it_deserialize_non_fungible_token_info_only_contents() {
            let json = r#"{"contents":{"data":"This is the contents of non-fungible token #1"}}"#;
            let non_fungible_token_info: NonFungibleTokenInfo = serde_json::from_str(json).unwrap();

            assert_eq!(non_fungible_token_info, NonFungibleTokenInfo {
                asset_id: None,
                metadata: None,
                contents: Some(NonFungibleTokenContents {
                    data: String::from("This is the contents of non-fungible token #1"),
                }),
            });
        }

        #[test]
        fn it_deserialize_retrieve_non_fungible_token_result_single_data_chunk() {
            let json = r#"{"nonFungibleToken":{"assetId":"a5sCytXhvRCCGZ7PhQ6o","metadata":{"name":"TestNFToken_1","description":"First non-fungible token issued for test","contentsEncrypted":true,"contentsURL":"https://localhost:8080/ipfs/QmeJKgZ638x2pfFVVaZAB9XjgLCSHh3e6qjMSCmSXKcuUq","custom":{"sensitiveProps":{"senseProp1":"XYZ","senseProp2":"456"},"propNum":5,"propStr":"ABC","propBool":true}},"contents":{"data":"This is the contents of non-fungible token #1"}}}"#;
            let retrieve_non_fungible_token_result: RetrieveNonFungibleTokenResult = serde_json::from_str(json).unwrap();

            assert_eq!(retrieve_non_fungible_token_result, RetrieveNonFungibleTokenResult {
                continuation_token: None,
                token_retrieval_id: None,
                non_fungible_token: Some(NonFungibleTokenInfo {
                    asset_id: Some(String::from("a5sCytXhvRCCGZ7PhQ6o")),
                    metadata: Some(NonFungibleTokenMetadata {
                        name: String::from("TestNFToken_1"),
                        description: Some(String::from("First non-fungible token issued for test")),
                        contents_encrypted: true,
                        contents_url: String::from("https://localhost:8080/ipfs/QmeJKgZ638x2pfFVVaZAB9XjgLCSHh3e6qjMSCmSXKcuUq"),
                        custom: json_obj!({
                            "sensitiveProps": {
                                "senseProp1": "XYZ",
                                "senseProp2": "456"
                            },
                            "propNum": 5,
                            "propStr": "ABC",
                            "propBool": true
                        }),
                    }),
                    contents: Some(NonFungibleTokenContents {
                        data: String::from("This is the contents of non-fungible token #1"),
                    }),
                }),
            });
        }

        #[test]
        fn it_deserialize_retrieve_non_fungible_token_result_multiple_data_chunk() {
            let json = r#"{"continuationToken":"eXxdwcoXm3dJBF7Ej759","nonFungibleToken":{"assetId":"ahfTzqgWAXnMR6Z57mcp","metadata":{"name":"TestNFToken_2","description":"Second non-fungible token issued for test","contentsEncrypted":true,"contentsURL":"https://localhost:8080/ipfs/Qmabg5V61n4hjcUjpGWzznLWmnJ7NEiTcgD8Xohb9kc5Je"},"contents":{"data":"This is the beginning of the contents of non-fungible token #2..."}}}"#;
            let retrieve_non_fungible_token_result: RetrieveNonFungibleTokenResult = serde_json::from_str(json).unwrap();

            assert_eq!(retrieve_non_fungible_token_result, RetrieveNonFungibleTokenResult {
                continuation_token: Some(String::from("eXxdwcoXm3dJBF7Ej759")),
                token_retrieval_id: None,
                non_fungible_token: Some(NonFungibleTokenInfo {
                    asset_id: Some(String::from("ahfTzqgWAXnMR6Z57mcp")),
                    metadata: Some(NonFungibleTokenMetadata {
                        name: String::from("TestNFToken_2"),
                        description: Some(String::from("Second non-fungible token issued for test")),
                        contents_encrypted: true,
                        contents_url: String::from("https://localhost:8080/ipfs/Qmabg5V61n4hjcUjpGWzznLWmnJ7NEiTcgD8Xohb9kc5Je"),
                        custom: None,
                    }),
                    contents: Some(NonFungibleTokenContents {
                        data: String::from("This is the beginning of the contents of non-fungible token #2..."),
                    }),
                }),
            });
        }

        #[test]
        fn it_deserialize_retrieve_non_fungible_token_result_async() {
            let json = r#"{"tokenRetrievalId":"rGEcL2HhoarCupvbkrv9"}"#;
            let retrieve_non_fungible_token_result: RetrieveNonFungibleTokenResult = serde_json::from_str(json).unwrap();

            assert_eq!(retrieve_non_fungible_token_result, RetrieveNonFungibleTokenResult {
                continuation_token: None,
                token_retrieval_id: Some(String::from("rGEcL2HhoarCupvbkrv9")),
                non_fungible_token: None,
            });
        }

        #[test]
        fn it_deserialize_retrieve_non_fungible_token_response() {
            let json = r#"{"status":"success","data":{"nonFungibleToken":{"assetId":"a5sCytXhvRCCGZ7PhQ6o","metadata":{"name":"TestNFToken_1","description":"First non-fungible token issued for test","contentsEncrypted":true,"contentsURL":"https://localhost:8080/ipfs/QmeJKgZ638x2pfFVVaZAB9XjgLCSHh3e6qjMSCmSXKcuUq","custom":{"sensitiveProps":{"senseProp1":"XYZ","senseProp2":"456"},"propNum":5,"propStr":"ABC","propBool":true}},"contents":{"data":"This is the contents of non-fungible token #1"}}}}"#;
            let retrieve_non_fungible_token_response: RetrieveNonFungibleTokenResponse = serde_json::from_str(json).unwrap();

            assert_eq!(retrieve_non_fungible_token_response, RetrieveNonFungibleTokenResponse {
                status: String::from("success"),
                data: RetrieveNonFungibleTokenResult {
                    continuation_token: None,
                    token_retrieval_id: None,
                    non_fungible_token: Some(NonFungibleTokenInfo {
                        asset_id: Some(String::from("a5sCytXhvRCCGZ7PhQ6o")),
                        metadata: Some(NonFungibleTokenMetadata {
                            name: String::from("TestNFToken_1"),
                            description: Some(String::from("First non-fungible token issued for test")),
                            contents_encrypted: true,
                            contents_url: String::from("https://localhost:8080/ipfs/QmeJKgZ638x2pfFVVaZAB9XjgLCSHh3e6qjMSCmSXKcuUq"),
                            custom: json_obj!({
                            "sensitiveProps": {
                                "senseProp1": "XYZ",
                                "senseProp2": "456"
                            },
                            "propNum": 5,
                            "propStr": "ABC",
                            "propBool": true
                        }),
                        }),
                        contents: Some(NonFungibleTokenContents {
                            data: String::from("This is the contents of non-fungible token #1"),
                        }),
                    }),
                }
            });
        }

        #[test]
        fn it_deserialize_nf_token_retrieval_process_error() {
            let json = r#"{"code":500,"message":"Internal server error"}"#;

            let nf_token_retrieval_process_error: NFTokenRetrievalProcessError = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_retrieval_process_error, NFTokenRetrievalProcessError {
                code: 500,
                message: String::from("Internal server error"),
            });
        }

        #[test]
        fn it_deserialize_nf_token_retrieval_process_progress_not_finished() {
            let json = r#"{"bytesRetrieved":0,"done":false}"#;

            let nf_token_retrieval_process_progress: NFTokenRetrievalProcessProgress = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_retrieval_process_progress, NFTokenRetrievalProcessProgress {
                bytes_retrieved: 0,
                done: false,
                success: None,
                error: None,
                finish_date: None,
            });
        }

        #[test]
        fn it_deserialize_nf_token_retrieval_process_progress_finished_error() {
            let json = r#"{"bytesRetrieved":512,"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2022-11-05T12:01:47.483"}"#;

            let nf_token_retrieval_process_progress: NFTokenRetrievalProcessProgress = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_retrieval_process_progress, NFTokenRetrievalProcessProgress {
                bytes_retrieved: 512,
                done: true,
                success: Some(false),
                error: Some(NFTokenRetrievalProcessError {
                    code: 500,
                    message: String::from("Internal server error"),
                }),
                finish_date: Some("2022-11-05T12:01:47.483".into()),
            });
        }

        #[test]
        fn it_deserialize_nf_token_retrieval_process_progress_finished_success() {
            let json = r#"{"bytesRetrieved":1024,"done":true,"success":true,"finishDate":"2022-11-05T12:06:32.405"}"#;

            let nf_token_retrieval_process_progress: NFTokenRetrievalProcessProgress = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_retrieval_process_progress, NFTokenRetrievalProcessProgress {
                bytes_retrieved: 1024,
                done: true,
                success: Some(true),
                error: None,
                finish_date: Some("2022-11-05T12:06:32.405".into()),
            });
        }

        #[test]
        fn it_deserialize_retrieve_nf_token_retrieval_progress_result_no_opts() {
            let json = r#"{"progress":{"bytesRetrieved":512,"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2022-11-05T12:01:47.483"}}"#;

            let nf_token_retrieval_progress_result: RetrieveNFTokenRetrievalProgressResult = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_retrieval_progress_result, RetrieveNFTokenRetrievalProgressResult {
                progress: NFTokenRetrievalProcessProgress {
                    bytes_retrieved: 512,
                    done: true,
                    success: Some(false),
                    error: Some(NFTokenRetrievalProcessError {
                        code: 500,
                        message: String::from("Internal server error"),
                    }),
                    finish_date: Some("2022-11-05T12:01:47.483".into()),
                },
                continuation_token: None,
            });
        }

        #[test]
        fn it_deserialize_retrieve_nf_token_retrieval_progress_result_all_opts() {
            let json = r#"{"progress":{"bytesRetrieved":1024,"done":true,"success":true,"finishDate":"2022-11-05T12:06:32.405"},"continuationToken":"eLNuL4M46n3BD57GNEuy"}"#;

            let nf_token_retrieval_progress_result: RetrieveNFTokenRetrievalProgressResult = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_retrieval_progress_result, RetrieveNFTokenRetrievalProgressResult {
                progress: NFTokenRetrievalProcessProgress {
                    bytes_retrieved: 1024,
                    done: true,
                    success: Some(true),
                    error: None,
                    finish_date: Some("2022-11-05T12:06:32.405".into()),
                },
                continuation_token: Some(String::from("eLNuL4M46n3BD57GNEuy")),
            });
        }

        #[test]
        fn it_deserialize_retrieve_nf_token_retrieval_progress_response() {
            let json = r#"{"status":"success","data":{"progress":{"bytesRetrieved":1024,"done":true,"success":true,"finishDate":"2022-11-05T12:06:32.405"},"continuationToken":"eLNuL4M46n3BD57GNEuy"}}"#;
            let nf_token_retrieval_progress_response: RetrieveNFTokenRetrievalProgressResponse = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_retrieval_progress_response, RetrieveNFTokenRetrievalProgressResponse {
                status: String::from("success"),
                data: RetrieveNFTokenRetrievalProgressResult {
                    progress: NFTokenRetrievalProcessProgress {
                        bytes_retrieved: 1024,
                        done: true,
                        success: Some(true),
                        error: None,
                        finish_date: Some("2022-11-05T12:06:32.405".into()),
                    },
                    continuation_token: Some(String::from("eLNuL4M46n3BD57GNEuy")),
                },
            });
        }

        #[test]
        fn it_deserialize_transfer_non_fungible_token_result() {
            let json = r#"{"success":true}"#;
            let transfer_non_fungible_token_result: TransferNonFungibleTokenResult = serde_json::from_str(json).unwrap();

            assert_eq!(transfer_non_fungible_token_result, TransferNonFungibleTokenResult {
                token_transfer_id: None,
                success: Some(true),
            });
        }

        #[test]
        fn it_deserialize_transfer_non_fungible_token_result_async() {
            let json = r#"{"tokenTransferId":"xuYnPMKQSBXi28wRaZpN"}"#;
            let transfer_non_fungible_token_result: TransferNonFungibleTokenResult = serde_json::from_str(json).unwrap();

            assert_eq!(transfer_non_fungible_token_result, TransferNonFungibleTokenResult {
                token_transfer_id: Some(String::from("xuYnPMKQSBXi28wRaZpN")),
                success: None,
            });
        }

        #[test]
        fn it_serialize_transfer_non_fungible_token_request() {
            let transfer_non_fungible_token_request = TransferNonFungibleTokenRequest {
                receiving_device: DeviceId {
                    id: String::from("drc3XdxNtzoucpw9xiRp"),
                    is_prod_unique_id: Some(false)
                },
                async_: None,
            };

            let json = serde_json::to_string(&transfer_non_fungible_token_request).unwrap();

            assert_eq!(json, r#"{"receivingDevice":{"id":"drc3XdxNtzoucpw9xiRp","isProdUniqueId":false}}"#);
        }

        #[test]
        fn it_serialize_transfer_non_fungible_token_request_async() {
            let transfer_non_fungible_token_request = TransferNonFungibleTokenRequest {
                receiving_device: DeviceId {
                    id: String::from("drc3XdxNtzoucpw9xiRp"),
                    is_prod_unique_id: Some(false)
                },
                async_: Some(true),
            };

            let json = serde_json::to_string(&transfer_non_fungible_token_request).unwrap();

            assert_eq!(json, r#"{"receivingDevice":{"id":"drc3XdxNtzoucpw9xiRp","isProdUniqueId":false},"async":true}"#);
        }

        #[test]
        fn it_deserialize_transfer_non_fungible_token_response() {
            let json = r#"{"status":"success","data":{"success":true}}"#;
            let transfer_non_fungible_token_response: TransferNonFungibleTokenResponse = serde_json::from_str(json).unwrap();

            assert_eq!(transfer_non_fungible_token_response, TransferNonFungibleTokenResponse {
                status: String::from("success"),
                data: TransferNonFungibleTokenResult {
                    token_transfer_id: None,
                    success: Some(true),
                },
            });
        }

        #[test]
        fn it_deserialize_nf_token_data_manipulation_progress_no_opts() {
            let json = r#"{"bytesRead":1234}"#;

            let nf_token_data_manipulation_progress: NFTokenDataManipulationProgress = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_data_manipulation_progress, NFTokenDataManipulationProgress {
                bytes_read: 1234,
                bytes_written: None,
            });
        }

        #[test]
        fn it_deserialize_nf_token_data_manipulation_progress_all_opts() {
            let json = r#"{"bytesRead":1234,"bytesWritten":1024}"#;

            let nf_token_data_manipulation_progress: NFTokenDataManipulationProgress = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_data_manipulation_progress, NFTokenDataManipulationProgress {
                bytes_read: 1234,
                bytes_written: Some(1024),
            });
        }

        #[test]
        fn it_deserialize_nf_token_transfer_process_error() {
            let json = r#"{"code":500,"message":"Internal server error"}"#;

            let nf_token_transfer_process_error: NFTokenTransferProcessError = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_transfer_process_error, NFTokenTransferProcessError {
                code: 500,
                message: String::from("Internal server error"),
            });
        }

        #[test]
        fn it_deserialize_nf_token_transfer_process_progress_not_finished() {
            let json = r#"{"dataManipulation":{"bytesRead":0},"done":false}"#;

            let nf_token_transfer_process_progress: NFTokenTransferProcessProgress = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_transfer_process_progress, NFTokenTransferProcessProgress {
                data_manipulation: NFTokenDataManipulationProgress {
                    bytes_read: 0,
                    bytes_written: None
                },
                done: false,
                success: None,
                error: None,
                finish_date: None,
            });
        }

        #[test]
        fn it_deserialize_nf_token_transfer_process_progress_finished_error() {
            let json = r#"{"dataManipulation":{"bytesRead":512},"done":true,"success":false,"error":{"code":500,"message":"Internal server error"},"finishDate":"2022-11-07T09:05:52.972"}"#;

            let nf_token_transfer_process_progress: NFTokenTransferProcessProgress = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_transfer_process_progress, NFTokenTransferProcessProgress {
                data_manipulation: NFTokenDataManipulationProgress {
                    bytes_read: 512,
                    bytes_written: None
                },
                done: true,
                success: Some(false),
                error: Some(NFTokenRetrievalProcessError {
                    code: 500,
                    message: String::from("Internal server error"),
                }),
                finish_date: Some("2022-11-07T09:05:52.972".into()),
            });
        }

        #[test]
        fn it_deserialize_nf_token_transfer_process_progress_finished_success() {
            let json = r#"{"dataManipulation":{"bytesRead":1234,"bytesWritten":1024},"done":true,"success":true,"finishDate":"2022-11-07T10:09:57.384"}"#;

            let nf_token_transfer_process_progress: NFTokenTransferProcessProgress = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_transfer_process_progress, NFTokenTransferProcessProgress {
                data_manipulation: NFTokenDataManipulationProgress {
                    bytes_read: 1234,
                    bytes_written: Some(1024),
                },
                done: true,
                success: Some(true),
                error: None,
                finish_date: Some("2022-11-07T10:09:57.384".into()),
            });
        }

        #[test]
        fn it_deserialize_retrieve_nf_token_transfer_progress_result_no_opts() {
            let json = r#"{"progress":{"dataManipulation":{"bytesRead":1234,"bytesWritten":1024},"done":true,"success":true,"finishDate":"2022-11-07T10:09:57.384"}}"#;

            let nf_token_transfer_progress_result: RetrieveNFTokenTransferProgressResult = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_transfer_progress_result, RetrieveNFTokenTransferProgressResult {
                progress: NFTokenTransferProcessProgress {
                    data_manipulation: NFTokenDataManipulationProgress {
                        bytes_read: 1234,
                        bytes_written: Some(1024),
                    },
                    done: true,
                    success: Some(true),
                    error: None,
                    finish_date: Some("2022-11-07T10:09:57.384".into()),
                },
            });
        }

        #[test]
        fn it_deserialize_retrieve_nf_token_transfer_progress_response() {
            let json = r#"{"status":"success","data":{"progress":{"dataManipulation":{"bytesRead":1234,"bytesWritten":1024},"done":true,"success":true,"finishDate":"2022-11-07T10:09:57.384"}}}"#;
            let nf_token_transfer_progress_response: RetrieveNFTokenTransferProgressResponse = serde_json::from_str(json).unwrap();

            assert_eq!(nf_token_transfer_progress_response, RetrieveNFTokenTransferProgressResponse {
                status: String::from("success"),
                data: RetrieveNFTokenTransferProgressResult {
                    progress: NFTokenTransferProcessProgress {
                        data_manipulation: NFTokenDataManipulationProgress {
                            bytes_read: 1234,
                            bytes_written: Some(1024),
                        },
                        done: true,
                        success: Some(true),
                        error: None,
                        finish_date: Some("2022-11-07T10:09:57.384".into()),
                    },
                },
            });
        }
    }
}