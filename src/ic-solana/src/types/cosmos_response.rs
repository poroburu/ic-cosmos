use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Represents the ABCI info response from a Cosmos node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct RpcAbciInfo {
    /// The ABCI response containing node information
    pub response: AbciResponse,
}

/// The ABCI response containing detailed node information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct AbciResponse {
    /// The application data, typically a JSON string containing application-specific information
    pub data: String,
    /// The version of the ABCI implementation
    pub version: String,
    /// The version of the application (optional)
    #[serde(rename = "app_version")]
    pub app_version: Option<String>,
    /// The height of the last committed block
    #[serde(rename = "last_block_height")]
    pub last_block_height: String,
    /// The application hash of the last committed block
    #[serde(rename = "last_block_app_hash")]
    pub last_block_app_hash: String,
}

/// Represents the health check response from a Cosmos node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct RpcHealthResponse {
    /// The status of the node's health check
    pub status: String,
}

/// Represents the status response from a Cosmos node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct RpcStatusResponse {
    /// The node information
    pub node_info: NodeInfo,
    /// The synchronization information
    pub sync_info: SyncInfo,
    /// The validator information
    pub validator_info: ValidatorInfo,
}

/// Represents node information in the status response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct NodeInfo {
    /// The protocol version information
    pub protocol_version: ProtocolVersion,
    /// The node's ID
    pub id: String,
    /// The address the node is listening on
    pub listen_addr: String,
    /// The network identifier
    pub network: String,
    /// The node's version
    pub version: String,
    /// The node's channels
    pub channels: String,
    /// The node's moniker
    pub moniker: String,
    /// Additional node information
    pub other: OtherInfo,
}

/// Represents protocol version information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ProtocolVersion {
    /// The P2P protocol version
    pub p2p: String,
    /// The block protocol version
    pub block: String,
    /// The application protocol version
    pub app: String,
}

/// Represents additional node information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct OtherInfo {
    /// The transaction index
    #[serde(rename = "tx_index")]
    pub tx_index: String,
    /// The RPC address
    #[serde(rename = "rpc_address")]
    pub rpc_address: String,
}

/// Represents synchronization information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct SyncInfo {
    /// The latest block hash
    #[serde(rename = "latest_block_hash")]
    pub latest_block_hash: String,
    /// The latest application hash
    #[serde(rename = "latest_app_hash")]
    pub latest_app_hash: String,
    /// The latest block height
    #[serde(rename = "latest_block_height")]
    pub latest_block_height: String,
    /// The latest block time
    #[serde(rename = "latest_block_time")]
    pub latest_block_time: String,
    /// The earliest block hash
    #[serde(rename = "earliest_block_hash")]
    pub earliest_block_hash: String,
    /// The earliest application hash
    #[serde(rename = "earliest_app_hash")]
    pub earliest_app_hash: String,
    /// The earliest block height
    #[serde(rename = "earliest_block_height")]
    pub earliest_block_height: String,
    /// The earliest block time
    #[serde(rename = "earliest_block_time")]
    pub earliest_block_time: String,
    /// Whether the node is catching up
    #[serde(rename = "catching_up")]
    pub catching_up: bool,
}

/// Represents validator information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorInfo {
    /// The validator's address
    pub address: String,
    /// The validator's public key
    pub pub_key: PubKey,
    /// The validator's voting power
    #[serde(rename = "voting_power")]
    pub voting_power: String,
}

/// Represents a public key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct PubKey {
    /// The type of the public key
    #[serde(rename = "type")]
    pub type_field: String,
    /// The value of the public key
    pub value: String,
}
