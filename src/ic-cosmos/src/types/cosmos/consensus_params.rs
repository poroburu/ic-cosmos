use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::cosmos::common::ConsensusParams;

/// Represents the response from the /consensus_params endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ConsensusParamsResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The consensus parameters result
    pub result: ConsensusParamsResult,
}

/// Represents the consensus parameters result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ConsensusParamsResult {
    /// The block height
    pub block_height: String,
    /// The consensus parameters
    pub consensus_params: ConsensusParams,
}
