use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::cosmos_common::{BlockHeader, BlockID};

/// Represents the response from the /commit endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct CommitResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The commit result
    pub result: CommitResult,
}

/// Represents the commit result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct CommitResult {
    /// The signed header
    pub signed_header: SignedHeader,
    /// Whether this commit is canonical
    pub canonical: bool,
}

/// Represents a signed header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct SignedHeader {
    /// The block header
    pub header: BlockHeader,
    /// The commit
    pub commit: Commit,
}

/// Represents a commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Commit {
    /// The commit height
    pub height: String,
    /// The commit round
    pub round: i32,
    /// The block ID
    pub block_id: BlockID,
    /// The commit signatures
    pub signatures: Vec<CommitSignature>,
}

/// Represents a commit signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct CommitSignature {
    /// The block ID flag
    pub block_id_flag: i32,
    /// The validator address
    pub validator_address: String,
    /// The timestamp
    pub timestamp: String,
    /// The signature
    pub signature: Option<String>,
}
