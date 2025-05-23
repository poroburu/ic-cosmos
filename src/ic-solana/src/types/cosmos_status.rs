use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Represents the complete status response from a Cosmos node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct StatusResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The status result containing node information
    pub result: Status,
}

/// The main status information containing node, sync, and validator details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Status {
    /// Information about the node
    pub node_info: NodeInfo,
    /// Information about the node's synchronization status
    pub sync_info: SyncInfo,
    /// Information about the validator
    pub validator_info: ValidatorInfo,
}

/// Represents detailed information about a node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct NodeInfo {
    /// Protocol version information
    pub protocol_version: ProtocolVersion,
    /// Node's unique identifier
    pub id: String,
    /// Address the node is listening on
    pub listen_addr: String,
    /// Network identifier
    pub network: String,
    /// Node's software version
    pub version: String,
    /// Node's communication channels
    pub channels: String,
    /// Node's moniker (human-readable name)
    pub moniker: String,
    /// Additional node information
    pub other: OtherInfo,
}

/// Represents protocol version information for different components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ProtocolVersion {
    /// P2P protocol version
    pub p2p: String,
    /// Block protocol version
    pub block: String,
    /// Application protocol version
    pub app: String,
}

/// Represents additional node information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct OtherInfo {
    /// Transaction index status
    #[serde(rename = "tx_index")]
    pub tx_index: String,
    /// RPC address
    #[serde(rename = "rpc_address")]
    pub rpc_address: String,
}

/// Represents synchronization information for the node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct SyncInfo {
    /// Hash of the latest block
    #[serde(rename = "latest_block_hash")]
    pub latest_block_hash: String,
    /// Application hash of the latest block
    #[serde(rename = "latest_app_hash")]
    pub latest_app_hash: String,
    /// Height of the latest block
    #[serde(rename = "latest_block_height")]
    pub latest_block_height: String,
    /// Timestamp of the latest block
    #[serde(rename = "latest_block_time")]
    pub latest_block_time: String,
    /// Hash of the earliest block
    #[serde(rename = "earliest_block_hash")]
    pub earliest_block_hash: String,
    /// Application hash of the earliest block
    #[serde(rename = "earliest_app_hash")]
    pub earliest_app_hash: String,
    /// Height of the earliest block
    #[serde(rename = "earliest_block_height")]
    pub earliest_block_height: String,
    /// Timestamp of the earliest block
    #[serde(rename = "earliest_block_time")]
    pub earliest_block_time: String,
    /// Whether the node is currently catching up
    #[serde(rename = "catching_up")]
    pub catching_up: bool,
}

/// Represents information about a validator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorInfo {
    /// Validator's address
    pub address: String,
    /// Validator's public key
    pub pub_key: PubKey,
    /// Validator's voting power
    #[serde(rename = "voting_power")]
    pub voting_power: String,
}

/// Represents a public key with its type and value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct PubKey {
    /// Type of the public key
    #[serde(rename = "type")]
    pub type_field: String,
    /// Base64 encoded public key value
    pub value: String,
}
