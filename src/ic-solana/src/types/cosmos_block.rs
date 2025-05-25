use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::cosmos_common::{BlockHeader, BlockID, Evidence};

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
