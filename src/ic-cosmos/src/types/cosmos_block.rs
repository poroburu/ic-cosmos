use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::cosmos_common::{BlockHeader, BlockID, ConsensusParams, Event, Evidence};

/// Represents the block response from a Cosmos node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The block result
    pub result: BlockComplete,
}

/// Represents a complete block with its ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockComplete {
    /// The block ID
    pub block_id: BlockID,
    /// The block data
    pub block: Block,
}

/// Represents a block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Block {
    /// The block header
    pub header: BlockHeader,
    /// The block data
    pub data: HashMap<String, Vec<String>>,
    /// The block evidence
    pub evidence: HashMap<String, Vec<Evidence>>,
    /// The last commit
    pub last_commit: Option<LastCommit>,
}

/// Represents the last commit information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct LastCommit {
    /// The commit height
    pub height: String,
    /// The commit round
    pub round: i32,
    /// The block ID
    pub block_id: BlockID,
    /// The commit signatures
    pub signatures: Vec<Signatures>,
}

/// Represents the signatures of the last commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Signatures {
    /// The block ID flag
    pub block_id_flag: i32,
    /// The validator address
    pub validator_address: String,
    /// The timestamp
    pub timestamp: String,
    /// The signature
    pub signature: Option<String>,
}

/// Represents the response from the /block_results endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockResultsResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The block results
    pub result: BlockResults,
}

/// Represents the block results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockResults {
    /// The block height
    pub height: String,
    /// The transaction results
    pub txs_results: Option<Vec<TxResult>>,
    /// The finalize block events
    pub finalize_block_events: Option<Vec<BlockEvent>>,
    /// The validator updates
    pub validator_updates: Option<Vec<ValidatorUpdate>>,
    /// The consensus parameter updates
    pub consensus_param_updates: Option<ConsensusParams>,
}

/// Represents a transaction result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct TxResult {
    /// The result code
    pub code: i32,
    /// The result data
    pub data: String,
    /// The result log
    pub log: String,
    /// Additional info
    pub info: String,
    /// Gas wanted
    pub gas_wanted: String,
    /// Gas used
    pub gas_used: String,
    /// Events
    pub events: Option<Vec<BlockEvent>>,
    /// Codespace
    pub codespace: String,
}

/// Represents a block event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockEvent {
    /// Event type
    pub r#type: String,
    /// Event attributes
    pub attributes: Vec<Event>,
}

/// Represents a validator update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorUpdate {
    /// Validator public key
    pub pub_key: ValidatorPubKey,
    /// Validator voting power
    pub power: String,
}

/// Represents a validator public key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorPubKey {
    /// Key type
    pub r#type: String,
    /// Key value
    pub value: String,
}
