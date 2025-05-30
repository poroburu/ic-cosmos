use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::cosmos_common::BlockHeader;

/// Represents the response from the /header endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct HeaderResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The header result
    pub result: HeaderResult,
}

/// Represents the header result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct HeaderResult {
    /// The block header
    pub header: BlockHeader,
}
