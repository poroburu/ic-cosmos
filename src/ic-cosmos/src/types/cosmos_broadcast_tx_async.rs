use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Represents the response from the /broadcast_tx_async endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BroadcastTxAsyncResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The error message if any
    pub error: String,
    /// The broadcast transaction result
    pub result: BroadcastTxResult,
}

/// Represents the broadcast transaction result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BroadcastTxResult {
    /// The result code
    pub code: i32,
    /// The result data
    pub data: String,
    /// The result log
    pub log: String,
    /// The codespace
    pub codespace: String,
    /// The transaction hash
    pub hash: String,
}
