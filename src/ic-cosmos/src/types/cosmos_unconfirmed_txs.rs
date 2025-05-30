use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Represents the response from the /num_unconfirmed_txs endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct NumUnconfirmedTransactionsResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The unconfirmed transactions result
    pub result: NumUnconfirmedTransactionsResult,
}

/// Represents the unconfirmed transactions result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct NumUnconfirmedTransactionsResult {
    /// The number of unconfirmed transactions
    pub n_txs: String,
    /// The total number of transactions
    pub total: String,
    /// The total size in bytes
    pub total_bytes: String,
}
