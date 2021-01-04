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

/// Data returned from a successful call to *List Permission Events* API method.
pub type ListPermissionEventsResult = HashMap<PermissionEvent, String>;
/// Data returned from a successful call to *Check Effective Permission Right* API method.
pub type CheckEffectivePermissionRightResult = HashMap<String, PermissionRight>;
/// Data returned from a successful call to *List Notification Events* API method.
pub type ListNotificationEventsResult = HashMap<NotificationEvent, String>;

/// Identifies a given virtual device.
#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
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
    /// Default value: **`Encoding::UTF8`***
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
    /// Default value: **`Encoding::UTF8`***
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
    /// **Note**: not returned for outbound sent messages sent with read confirmation not enabled.
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

/// Information about an asset issuance event.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetIssuanceEventEntry {
    /// The issued amount of the asset.
    pub amount: f64,
    /// The virtual device to which the issued amount was assigned.
    pub holding_device: DeviceInfo,
    /// Date and time when the asset was issued.
    pub date: UtcDateTime,
}

/// Information about an asset holder.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetHolderEntry {
    /// The virtual device that holds an amount of the asset.
    pub holder: DeviceInfo,
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
    fn it_deserialize_asset_issuance_event_entry() {
        let json = r#"{"amount":123,"holdingDevice":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"date":"2020-12-23T10:51:45.935Z"}"#;

        let asset_issuance_event_entry: AssetIssuanceEventEntry = serde_json::from_str(json).unwrap();

        assert_eq!(asset_issuance_event_entry, AssetIssuanceEventEntry {
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
        let json = r#"{"issuanceEvents":[{"amount":123,"holdingDevice":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"date":"2020-12-23T10:51:45.935Z"},{"amount":150,"holdingDevice":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"},"date":"2020-12-23T11:17:23.731Z"}],"hasMore":false}"#;

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
                    date: "2020-12-23T10:51:45.935Z".into(),
                },
                AssetIssuanceEventEntry {
                    amount: 150.0,
                    holding_device: DeviceInfo {
                        device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                        name: None,
                        prod_unique_id: None,
                    },
                    date: "2020-12-23T11:17:23.731Z".into(),
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
        let json = r#"{"status":"success","data":{"issuanceEvents":[{"amount":123,"holdingDevice":{"deviceId":"drc3XdxNtzoucpw9xiRp"},"date":"2020-12-23T10:51:45.935Z"},{"amount":150,"holdingDevice":{"deviceId":"d8YpQ7jgPBJEkBrnvp58"},"date":"2020-12-23T11:17:23.731Z"}],"hasMore":false}}"#;

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
                        date: "2020-12-23T10:51:45.935Z".into(),
                    },
                    AssetIssuanceEventEntry {
                        amount: 150.0,
                        holding_device: DeviceInfo {
                            device_id: String::from("d8YpQ7jgPBJEkBrnvp58"),
                            name: None,
                            prod_unique_id: None,
                        },
                        date: "2020-12-23T11:17:23.731Z".into(),
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
}