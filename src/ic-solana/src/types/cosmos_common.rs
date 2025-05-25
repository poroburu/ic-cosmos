use candid::CandidType;
use serde::{Deserialize, Serialize};

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

/// Represents an event in the Cosmos system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Event {
    /// The event key
    pub key: String,
    /// The event value
    pub value: String,
    /// Whether the event is indexed
    pub index: bool,
}

/// Represents a block ID with its hash and parts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockID {
    /// The block hash
    pub hash: String,
    /// The block parts information
    pub parts: BlockParts,
}

/// Represents the parts of a block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockParts {
    /// Total number of parts
    pub total: i32,
    /// Hash of the parts
    pub hash: String,
}

/// Represents a validator with its public key and voting power.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Validator {
    /// The validator's public key
    pub pub_key: PubKey,
    /// The validator's voting power
    pub voting_power: i64,
    /// The validator's address
    pub address: String,
}

/// Represents a validator with priority information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorPriority {
    /// The validator's address
    pub address: String,
    /// The validator's public key
    pub pub_key: PubKey,
    /// The validator's voting power
    pub voting_power: String,
    /// The validator's proposer priority
    pub proposer_priority: String,
}

/// Represents consensus parameters for the blockchain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ConsensusParams {
    /// Block parameters
    pub block: BlockParams,
    /// Evidence parameters
    pub evidence: EvidenceParams,
    /// Validator parameters
    pub validator: ValidatorParams,
}

/// Represents block-specific consensus parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockParams {
    /// Maximum block size in bytes
    pub max_bytes: String,
    /// Maximum gas per block
    pub max_gas: String,
    /// Time interval between blocks in milliseconds
    pub time_iota_ms: String,
}

/// Represents evidence-specific consensus parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct EvidenceParams {
    /// Maximum age of evidence in blocks
    pub max_age: String,
}

/// Represents validator-specific consensus parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorParams {
    /// List of allowed public key types
    pub pub_key_types: Vec<String>,
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
